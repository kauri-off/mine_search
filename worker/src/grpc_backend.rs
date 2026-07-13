//! `grpc` feature (default ON): the worker dials the backend, registers, streams
//! scan results and heartbeats, and applies commands (ping/scan/config/control)
//! pushed back over the same bidirectional `Session` stream.

use std::{
    path::Path,
    sync::{Arc, Mutex, atomic::Ordering},
    time::Duration,
};

use anyhow::anyhow;
use proto::worker::{
    Heartbeat, Register, ScanResult, ServerExtra, ServerFilter as PbFilter, ServerReport,
    WorkerConfig as PbConfig, WorkerMessage, WorkerMetrics, scan_result, server_command,
    worker_control_client::WorkerControlClient, worker_message,
};
use tokio::{sync::mpsc, task::JoinSet};
use tokio_stream::{StreamExt, wrappers::ReceiverStream};
use tonic::{
    Request, Status,
    service::{Interceptor, interceptor::InterceptedService},
    transport::{Certificate, Channel, ClientTlsConfig},
};
use tracing::{debug, error, info, warn};

use crate::{
    config::WorkerConfig,
    engine::{Engine, RuntimeConfig, UpdateTarget, UpdateTargetItem},
    outbox::Outbox,
    report::ScanReport,
};

/// How often the replay sweep runs, and how long a result may go un-acked before
/// it is re-sent. Covers the "link up but backend can't persist" case.
const SWEEP_INTERVAL: Duration = Duration::from_secs(30);
const RESEND_AFTER: Duration = Duration::from_secs(60);

#[derive(Clone)]
pub struct AuthInterceptor {
    token: Option<String>,
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, Status> {
        if let Some(t) = &self.token {
            let value = format!("Bearer {t}")
                .parse()
                .map_err(|_| Status::internal("invalid worker token"))?;
            req.metadata_mut().insert("authorization", value);
        }
        Ok(req)
    }
}

type Client = WorkerControlClient<InterceptedService<Channel, AuthInterceptor>>;

/// Converts a config-file [`crate::config::ServerFilter`] into its proto form.
/// `pub(crate)` so [`crate::engine::RuntimeConfig`] can reuse it.
pub(crate) fn filter_to_proto(f: &crate::config::ServerFilter) -> PbFilter {
    PbFilter {
        online: f.online,
        licensed: f.licensed,
        checked: f.checked,
        crashed: f.crashed,
        requires_mods: f.requires_mods,
        has_players: f.has_players,
        has_none_players: f.has_none_players,
        join_status: f.join_status.clone(),
        query: f.query.clone(),
    }
}

fn config_to_proto(c: &WorkerConfig) -> PbConfig {
    PbConfig {
        threads: c.threads,
        search_module: c.search_module,
        update_module: c.update_module,
        update_with_connection: c.update_with_connection,
        update_interval_secs: c.update_interval_secs,
        update_concurrency: c.update_concurrency,
        update_filter: Some(filter_to_proto(&c.update_filter)),
        search_filter: Some(filter_to_proto(&c.search_filter)),
    }
}

/// Renders a proto [`PbFilter`] into a `toml_edit` table, emitting only the set
/// fields so an unset filter stays absent from the file.
fn filter_to_toml(f: &Option<PbFilter>) -> toml_edit::Table {
    let mut t = toml_edit::Table::new();
    if let Some(f) = f {
        if let Some(v) = f.online {
            t["online"] = toml_edit::value(v);
        }
        if let Some(v) = f.licensed {
            t["licensed"] = toml_edit::value(v);
        }
        if let Some(v) = f.checked {
            t["checked"] = toml_edit::value(v);
        }
        if let Some(v) = f.crashed {
            t["crashed"] = toml_edit::value(v);
        }
        if let Some(v) = f.requires_mods {
            t["requires_mods"] = toml_edit::value(v);
        }
        if let Some(v) = f.has_players {
            t["has_players"] = toml_edit::value(v);
        }
        if let Some(v) = f.has_none_players {
            t["has_none_players"] = toml_edit::value(v);
        }
        if let Some(v) = &f.join_status {
            t["join_status"] = toml_edit::value(v.as_str());
        }
        if let Some(v) = &f.query {
            t["query"] = toml_edit::value(v.as_str());
        }
    }
    t
}

/// Surgically rewrites the live-tunable `[worker]` keys in the worker's config
/// file so a UI-driven retune survives restarts. Uses `toml_edit` to preserve
/// comments and the non-tunable connection fields (`backend_url`, `token`, …).
/// Best-effort: a failure is logged, never fatal (mirrors the `worker_id` write).
fn persist_config(path: &Path, c: &PbConfig) {
    let existing = std::fs::read_to_string(path).unwrap_or_default();
    let mut doc = match existing.parse::<toml_edit::DocumentMut>() {
        Ok(doc) => doc,
        Err(e) => {
            warn!("could not parse {} to persist config: {e}", path.display());
            return;
        }
    };

    let worker = doc["worker"].or_insert(toml_edit::table());
    worker["threads"] = toml_edit::value(c.threads as i64);
    worker["search_module"] = toml_edit::value(c.search_module);
    worker["update_module"] = toml_edit::value(c.update_module);
    worker["update_with_connection"] = toml_edit::value(c.update_with_connection);
    worker["update_interval_secs"] = toml_edit::value(c.update_interval_secs as i64);
    worker["update_concurrency"] = toml_edit::value(c.update_concurrency as i64);
    // Replace the whole subtable each time so clearing a filter drops its key.
    worker["update_filter"] = toml_edit::Item::Table(filter_to_toml(&c.update_filter));
    worker["search_filter"] = toml_edit::Item::Table(filter_to_toml(&c.search_filter));

    if let Err(e) = std::fs::write(path, doc.to_string()) {
        warn!("could not persist config to {}: {e}", path.display());
    } else {
        info!("persisted updated config to {}", path.display());
    }
}

/// Writes the generated `[worker].id` key into the worker's config file so the
/// identity (and thus operator-pinned config) survives restarts. Best-effort,
/// like [`persist_config`].
pub fn persist_id(path: &Path, id: &str) {
    let existing = std::fs::read_to_string(path).unwrap_or_default();
    let mut doc = match existing.parse::<toml_edit::DocumentMut>() {
        Ok(doc) => doc,
        Err(e) => {
            warn!(
                "could not parse {} to persist worker id: {e}",
                path.display()
            );
            return;
        }
    };

    let worker = doc["worker"].or_insert(toml_edit::table());
    worker["id"] = toml_edit::value(id);

    if let Err(e) = std::fs::write(path, doc.to_string()) {
        warn!("could not persist worker id to {}: {e}", path.display());
    } else {
        info!("persisted worker id to {}", path.display());
    }
}

/// Rewrites the `[worker].name` key in the worker's config file so a UI-driven
/// rename survives restarts. `None` clears the key. Best-effort, like
/// [`persist_config`].
fn persist_name(path: &Path, name: &Option<String>) {
    let existing = std::fs::read_to_string(path).unwrap_or_default();
    let mut doc = match existing.parse::<toml_edit::DocumentMut>() {
        Ok(doc) => doc,
        Err(e) => {
            warn!("could not parse {} to persist name: {e}", path.display());
            return;
        }
    };

    let worker = doc["worker"].or_insert(toml_edit::table());
    match name {
        Some(n) => worker["name"] = toml_edit::value(n.as_str()),
        None => {
            if let Some(t) = worker.as_table_mut() {
                t.remove("name");
            }
        }
    }

    if let Err(e) = std::fs::write(path, doc.to_string()) {
        warn!("could not persist name to {}: {e}", path.display());
    } else {
        info!("persisted updated name to {}", path.display());
    }
}

/// Upper bounds on backend-supplied config. A malicious or buggy `SetConfig`
/// with a huge `threads`/`update_concurrency` would otherwise spawn that many
/// tasks and OOM the worker, so both are clamped here where backend config
/// enters the runtime.
const MAX_THREADS: i32 = 10000;
const MAX_UPDATE_CONCURRENCY: u32 = 10000;

fn proto_to_runtime(c: &PbConfig) -> RuntimeConfig {
    RuntimeConfig {
        threads: c.threads.clamp(0, MAX_THREADS),
        search_module: c.search_module,
        update_module: c.update_module,
        update_with_connection: c.update_with_connection,
        update_interval_secs: c.update_interval_secs,
        update_concurrency: c.update_concurrency.min(MAX_UPDATE_CONCURRENCY),
        update_filter: c.update_filter.clone().unwrap_or_default(),
        search_filter: c.search_filter.clone().unwrap_or_default(),
    }
}

fn report_to_proto(report: ScanReport) -> ServerReport {
    ServerReport {
        ip: report.ip,
        port: report.port,
        version_name: report.version_name,
        protocol: report.protocol,
        description_json: report.description.to_string(),
        players_online: report.players_online,
        players_max: report.players_max,
        player_names: report.player_names,
        requires_mods: report.requires_mods,
        favicon: report.favicon,
        ping: report.ping,
        extra: report.extra.map(|e| ServerExtra {
            is_online_mode: e.is_online_mode,
            disconnect_reason_json: e.disconnect_reason.map(|v| v.to_string()),
        }),
    }
}

/// Shared, swappable handle to the *current* gRPC session's transport.
///
/// The [`Engine`] and its tasks (search pool, update loop) are long-lived — they
/// are created once and outlive any individual connection. A single session, by
/// contrast, owns an outbound channel and an RPC client that die when the link
/// drops. This handle bridges the two: on each (re)connect `run` [`set`](Self::set)s
/// the new session's sender/client, and [`clear`](Self::clear)s them when it ends.
/// The engine therefore keeps its state (update-cycle timer, search pool, counters)
/// across reconnects instead of being torn down and rebuilt every time.
#[derive(Clone, Default)]
pub struct SessionLink {
    tx: Arc<Mutex<Option<mpsc::Sender<WorkerMessage>>>>,
    client: Arc<Mutex<Option<Client>>>,
}

impl SessionLink {
    fn set(&self, tx: mpsc::Sender<WorkerMessage>, client: Client) {
        *self.tx.lock().unwrap() = Some(tx);
        *self.client.lock().unwrap() = Some(client);
    }

    fn clear(&self) {
        *self.tx.lock().unwrap() = None;
        *self.client.lock().unwrap() = None;
    }

    fn sender(&self) -> Option<mpsc::Sender<WorkerMessage>> {
        self.tx.lock().unwrap().clone()
    }

    fn client(&self) -> Option<Client> {
        self.client.lock().unwrap().clone()
    }
}

/// Aborts the per-session tasks (heartbeat, replay sweep) and clears the shared
/// [`SessionLink`] when `run` returns by any path, so a stale sender/client is
/// never handed to the long-lived engine after the link drops.
struct SessionGuard {
    handles: Vec<tokio::task::JoinHandle<()>>,
    link: SessionLink,
}

impl Drop for SessionGuard {
    fn drop(&mut self) {
        for h in &self.handles {
            h.abort();
        }
        self.link.clear();
    }
}

/// Streams the engine's scan outcomes to the backend over the current session's
/// channel, persisting each to the durable [`Outbox`] first so nothing is lost
/// if the link is down.
pub struct GrpcSink {
    link: SessionLink,
    outbox: Arc<Outbox>,
}

impl GrpcSink {
    async fn send(&self, outcome: scan_result::Outcome) {
        let result_id = uuid::Uuid::new_v4().to_string();
        let msg = WorkerMessage {
            kind: Some(worker_message::Kind::Result(ScanResult {
                outcome: Some(outcome),
                result_id: result_id.clone(),
            })),
        };
        // Persist before sending: if there is no live session (or the result is
        // never acked), the periodic sweep / reconnect replay re-sends it.
        self.outbox.add(&result_id, &msg).await;
        let delivered = match self.link.sender() {
            Some(tx) => tx.send(msg).await.is_ok(),
            None => false,
        };
        if !delivered {
            // Expected while disconnected or during a teardown: the result is safe
            // in the outbox and the next session replays it. A handled condition,
            // not a fault, so it stays at debug level.
            debug!("no active session; scan result retained in outbox for replay");
        }
    }

    /// A server found via the search/scan path (upsert-by-ip).
    pub async fn discovered(&self, report: ScanReport) {
        self.send(scan_result::Outcome::Discovered(report_to_proto(report)))
            .await;
    }
    /// A server re-probed during an update cycle / on-demand ping (full update).
    pub async fn updated(&self, report: ScanReport) {
        self.send(scan_result::Outcome::Updated(report_to_proto(report)))
            .await;
    }
    /// A server that failed re-probing: mark offline.
    pub async fn offline(&self, ip: &str) {
        self.send(scan_result::Outcome::Offline(
            proto::worker::ServerOffline { ip: ip.to_string() },
        ))
        .await;
    }
}

/// Asks the backend which servers to re-probe during an update cycle. Pulls the
/// current session's client from the shared [`SessionLink`] so update cycles keep
/// working across reconnects (and fail gracefully while disconnected).
pub struct GrpcTargetSource {
    link: SessionLink,
    worker_id: String,
}

impl GrpcTargetSource {
    /// Opens the server-streaming `FetchUpdateTargets` RPC and yields the leading
    /// total followed by one target at a time. The backend stamps each target's
    /// `with_connection` from this worker's registered config, so the caller no
    /// longer supplies it. Streaming means neither side has to buffer the whole
    /// `servers` table.
    pub async fn update_targets(
        &self,
        filter: PbFilter,
    ) -> anyhow::Result<
        impl tokio_stream::Stream<Item = anyhow::Result<UpdateTargetItem>> + Send + use<>,
    > {
        use proto::worker::fetch_update_targets_response::Kind;
        let mut client = self
            .link
            .client()
            .ok_or_else(|| anyhow!("no active backend session"))?;
        let stream = client
            .fetch_update_targets(proto::worker::FetchUpdateTargetsRequest {
                worker_id: self.worker_id.clone(),
                filter: Some(filter),
            })
            .await?
            .into_inner();

        Ok(stream.map(|res| {
            let msg = res.map_err(anyhow::Error::from)?;
            match msg.kind {
                Some(Kind::Total(n)) => Ok(UpdateTargetItem::Total(n)),
                Some(Kind::Target(t)) => Ok(UpdateTargetItem::Target(UpdateTarget {
                    ip: t.ip,
                    port: t.port as u16,
                    with_connection: t.with_connection,
                })),
                None => Err(anyhow!("empty FetchUpdateTargets frame")),
            }
        }))
    }
}

async fn build_channel(url: &str, cfg: &WorkerConfig) -> anyhow::Result<Channel> {
    let mut endpoint = Channel::from_shared(url.to_string())?;
    if url.starts_with("https") {
        let mut tls = ClientTlsConfig::new().with_webpki_roots();
        if let Some(ca) = &cfg.tls_ca {
            tls = tls.ca_certificate(Certificate::from_pem(std::fs::read(ca)?));
        }
        endpoint = endpoint.tls_config(tls)?;
    }
    if cfg.insecure.unwrap_or(false) {
        warn!("[worker].insecure is set but certificate verification cannot be disabled; ignoring");
    }
    Ok(endpoint.connect().await?)
}

/// Builds the long-lived engine and the [`SessionLink`] its sink/target source
/// share. Called once at startup; the engine's search pool and update loop then
/// run for the whole process, surviving every reconnect. `run` swaps each new
/// session's transport into the returned link.
pub fn build_engine(
    cfg: &WorkerConfig,
    worker_id: &str,
    outbox: Arc<Outbox>,
) -> (Arc<Engine>, SessionLink) {
    let link = SessionLink::default();
    let sink = GrpcSink {
        link: link.clone(),
        outbox,
    };
    let targets = GrpcTargetSource {
        link: link.clone(),
        worker_id: worker_id.to_string(),
    };
    let engine = Engine::new(sink, targets, RuntimeConfig::from(cfg));
    // Start the search supervisor and update loop once. Dropping the handles
    // detaches the tasks: they must outlive any single session and are only
    // stopped when the process exits.
    let _ = engine.start();
    (engine, link)
}

/// Connects and registers a session for the already-running `engine`, publishes
/// the session's transport into `link`, and pumps commands/heartbeats until the
/// stream closes or a shutdown command arrives. Returns `Ok` on a clean shutdown;
/// the caller may retry on `Err`. The engine itself is untouched, so its state
/// (update-cycle timer, search pool, counters) carries across reconnects.
pub async fn run(
    cfg: &WorkerConfig,
    worker_id: &str,
    config_path: &Path,
    outbox: &Arc<Outbox>,
    engine: &Arc<Engine>,
    link: &SessionLink,
) -> anyhow::Result<()> {
    let backend_url = cfg
        .backend_url
        .clone()
        .ok_or_else(|| anyhow!("[worker].backend_url is required in gRPC mode"))?;

    info!("connecting to backend at {backend_url}");
    let channel = build_channel(&backend_url, cfg).await?;
    let interceptor = AuthInterceptor {
        token: cfg.token.clone(),
    };
    let client = WorkerControlClient::with_interceptor(channel, interceptor);

    let (msg_tx, msg_rx) = mpsc::channel::<WorkerMessage>(256);

    // Register first so it is the first message the backend sees. Sent before the
    // link is published and the heartbeat is spawned so nothing can jump ahead of
    // it in the channel buffer.
    msg_tx
        .send(WorkerMessage {
            kind: Some(worker_message::Kind::Register(Register {
                worker_id: worker_id.to_string(),
                name: cfg.name.clone(),
                config: Some(config_to_proto(cfg)),
                version: env!("CARGO_PKG_VERSION").to_string(),
            })),
        })
        .await?;

    let mut session_client = client.clone();
    let mut inbound = session_client
        .session(ReceiverStream::new(msg_rx))
        .await?
        .into_inner();

    // Session is up: publish its transport to the long-lived engine (its
    // sink/target source now route through this connection) and start the
    // per-session tasks. The guard clears the link and aborts these tasks on
    // every exit path below.
    link.set(msg_tx.clone(), client.clone());
    let _guard = SessionGuard {
        handles: vec![
            tokio::spawn(heartbeat(engine.clone(), msg_tx.clone())),
            tokio::spawn(replay_sweep(outbox.clone(), msg_tx.clone())),
        ],
        link: link.clone(),
    };

    info!(worker = %worker_id, "session established");

    // Replay every result still awaiting an ack from a previous session.
    for msg in outbox.collect(None).await {
        if msg_tx.send(msg).await.is_err() {
            return Err(anyhow!("session closed before outbox replay completed"));
        }
    }

    // On-demand ping/scan commands run as detached tasks. Track them in a
    // JoinSet so they are aborted when the session ends (the JoinSet drops with
    // `run`) instead of surviving reconnects; drain finished ones each iteration
    // so completed handles don't accumulate over a long-lived session.
    let mut cmd_tasks: JoinSet<()> = JoinSet::new();

    loop {
        let cmd = tokio::select! {
            // The backend dropped our outbound (request) half while keeping the
            // inbound half open: the session is half-dead — scan results can no
            // longer be delivered and only accumulate in the outbox. Tear down now
            // so `main` reconnects and re-establishes a working link, instead of
            // spinning (and flooding logs) until the inbound half also closes.
            _ = msg_tx.closed() => {
                return Err(anyhow!("session outbound channel closed by backend"));
            }
            msg = inbound.message() => match msg? {
                Some(cmd) => cmd,
                None => break,
            },
        };
        while cmd_tasks.try_join_next().is_some() {}
        match cmd.cmd {
            Some(server_command::Cmd::SetConfig(c)) => {
                // Only persist/apply when the runtime config actually changed, so a
                // reconnect that re-pushes identical config doesn't rewrite the file
                // or churn the search pool.
                let rc = proto_to_runtime(&c);
                if engine.config() != rc {
                    persist_config(config_path, &c);
                    engine.set_config(rc);
                }
            }
            Some(server_command::Cmd::SetName(s)) => {
                persist_name(config_path, &s.name);
            }
            Some(server_command::Cmd::Ack(ack)) => {
                outbox.ack(&ack.result_id).await;
            }
            Some(server_command::Cmd::Ping(p)) => {
                if !worker::is_probeable_ip(&p.ip) {
                    warn!("ignoring ping to non-probeable address {}", p.ip);
                    continue;
                }
                let engine = engine.clone();
                cmd_tasks.spawn(async move {
                    engine.ping(p.ip, p.port as u16, p.with_connection).await;
                });
            }
            Some(server_command::Cmd::Scan(s)) => {
                if !worker::is_probeable_ip(&s.ip) {
                    warn!("ignoring scan of non-probeable address {}", s.ip);
                    continue;
                }
                let engine = engine.clone();
                cmd_tasks.spawn(async move {
                    engine.scan(s.ip, s.port as u16).await;
                });
            }
            Some(server_command::Cmd::Control(ctrl)) => {
                match proto::worker::Control::try_from(ctrl)
                    .unwrap_or(proto::worker::Control::Unspecified)
                {
                    proto::worker::Control::PauseSearch => engine.set_paused(true),
                    proto::worker::Control::ResumeSearch => engine.set_paused(false),
                    proto::worker::Control::AbortUpdate => engine.abort_update(),
                    proto::worker::Control::TriggerUpdate => engine.trigger_update(),
                    proto::worker::Control::Shutdown => {
                        info!("shutdown command received");
                        return Ok(());
                    }
                    proto::worker::Control::Unspecified => {}
                }
            }
            None => {}
        }
    }

    Err(anyhow!("session stream closed by backend"))
}

/// Periodically re-sends outbox results that have gone too long without an ack,
/// so a result is eventually delivered even if the link stays up while the
/// backend cannot persist. Exits (via the channel error) when the session ends.
async fn replay_sweep(outbox: Arc<Outbox>, tx: mpsc::Sender<WorkerMessage>) {
    let mut interval = tokio::time::interval(SWEEP_INTERVAL);
    interval.tick().await; // skip the immediate first tick
    loop {
        interval.tick().await;
        for msg in outbox.collect(Some(RESEND_AFTER)).await {
            if tx.send(msg).await.is_err() {
                return;
            }
        }
    }
}

async fn heartbeat(engine: Arc<Engine>, tx: mpsc::Sender<WorkerMessage>) {
    // Counters are cumulative across the engine's whole life (it outlives the
    // session), so seed the rate baselines from their current values — otherwise
    // the first heartbeat of a reconnected session would report a huge spike.
    let mut prev_scanned = engine.ips_scanned.load(Ordering::Relaxed);
    let mut prev_update_done = engine.update_done.load(Ordering::Relaxed);
    let period = Duration::from_secs(5);
    let mut interval = tokio::time::interval(period);

    loop {
        interval.tick().await;
        let scanned = engine.ips_scanned.load(Ordering::Relaxed);
        let rate = scanned.saturating_sub(prev_scanned) as f64 / period.as_secs_f64();
        prev_scanned = scanned;

        let update_done = engine.update_done.load(Ordering::Relaxed);
        let update_rate =
            update_done.saturating_sub(prev_update_done) as f64 / period.as_secs_f64();
        prev_update_done = update_done;

        let cfg = engine.config();
        let searching = engine.searching();
        let metrics = WorkerMetrics {
            servers_found: engine.servers_found.load(Ordering::Relaxed),
            ips_scanned: scanned,
            scan_rate: rate,
            uptime_secs: engine.started.elapsed().as_secs(),
            searching,
            updating: engine.updating.load(Ordering::Relaxed),
            active_threads: if searching {
                cfg.threads.max(0) as u32
            } else {
                0
            },
            update_done,
            update_total: engine.update_total.load(Ordering::Relaxed),
            update_rate,
            last_update_unix: engine.last_update_unix.load(Ordering::Relaxed),
        };

        let msg = WorkerMessage {
            kind: Some(worker_message::Kind::Heartbeat(Heartbeat {
                metrics: Some(metrics),
            })),
        };
        if tx.send(msg).await.is_err() {
            error!("heartbeat channel closed");
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ServerFilter as FileFilter, WorkerConfig as FileConfig};

    fn file_config() -> FileConfig {
        FileConfig {
            threads: 42,
            search_module: true,
            update_module: true,
            update_with_connection: true,
            update_interval_secs: 300,
            update_concurrency: 25,
            update_filter: FileFilter {
                licensed: Some(true),
                ..FileFilter::default()
            },
            search_filter: FileFilter::default(),
            log_level: None,
            backend_url: Some("http://backend:50051".into()),
            token: Some("secret".into()),
            id: Some("worker-1".into()),
            name: Some("Alpha".into()),
            tls_ca: None,
            insecure: None,
        }
    }

    // Guards the "update_with_connection change is silently dropped" bug: this
    // knob must be part of the runtime config so a UI toggle of only this field is
    // detected as a change (and thus persisted + re-registered), instead of the
    // `engine.config() != rc` guard treating it as a no-op.
    #[test]
    fn proto_to_runtime_tracks_update_with_connection() {
        let mut a = config_to_proto(&file_config());
        a.update_with_connection = false;
        let mut b = a.clone();
        b.update_with_connection = true;
        assert_ne!(
            proto_to_runtime(&a),
            proto_to_runtime(&b),
            "toggling update_with_connection must change the runtime config"
        );
    }

    // The backend re-pushes the worker's own config on every (re)connect. That
    // echo must compare equal to the config the engine already built at startup,
    // otherwise every reconnect would needlessly churn the search pool. This holds
    // only if every proto field is mirrored in RuntimeConfig on both paths.
    #[test]
    fn register_echo_is_a_noop_for_unchanged_config() {
        let cfg = file_config();
        let from_file = RuntimeConfig::from(&cfg);
        let via_proto = proto_to_runtime(&config_to_proto(&cfg));
        assert_eq!(
            from_file, via_proto,
            "startup config and the register echo must be identical"
        );
    }
}
