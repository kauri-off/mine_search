//! `grpc` feature (default ON): the worker dials the backend, registers, streams
//! scan results and heartbeats, and applies commands (ping/scan/config/control)
//! pushed back over the same bidirectional `Session` stream.

use std::{
    path::{Path, PathBuf},
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};

use anyhow::anyhow;
use proto::worker::{
    Heartbeat, Register, ScanResult, ServerExtra, ServerReport, WorkerConfig as PbConfig,
    WorkerMessage, WorkerMetrics, scan_result, server_command,
    worker_control_client::WorkerControlClient, worker_message,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{
    Request, Status,
    service::{Interceptor, interceptor::InterceptedService},
    transport::{Channel, ClientTlsConfig, Certificate},
};
use tracing::{error, info, warn};

use crate::{
    config::WorkerConfig,
    engine::{Engine, RuntimeConfig, UpdateTarget},
    report::ScanReport,
};

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

fn config_to_proto(c: &WorkerConfig) -> PbConfig {
    PbConfig {
        threads: c.threads,
        search_module: c.search_module,
        update_module: c.update_module,
        update_with_connection: c.update_with_connection,
        only_update_spoofable: c.only_update_spoofable,
        only_update_cracked: c.only_update_cracked,
        update_interval_secs: c.update_interval_secs,
        update_concurrency: c.update_concurrency,
    }
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
    worker["only_update_spoofable"] = toml_edit::value(c.only_update_spoofable);
    worker["only_update_cracked"] = toml_edit::value(c.only_update_cracked);
    worker["update_interval_secs"] = toml_edit::value(c.update_interval_secs as i64);
    worker["update_concurrency"] = toml_edit::value(c.update_concurrency as i64);

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
            warn!("could not parse {} to persist worker id: {e}", path.display());
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

fn proto_to_runtime(c: PbConfig) -> RuntimeConfig {
    RuntimeConfig {
        threads: c.threads,
        search_module: c.search_module,
        update_module: c.update_module,
        update_with_connection: c.update_with_connection,
        only_update_spoofable: c.only_update_spoofable,
        only_update_cracked: c.only_update_cracked,
        update_interval_secs: c.update_interval_secs,
        update_concurrency: c.update_concurrency,
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

/// Aborts the per-session engine tasks (search supervisor, update loop,
/// heartbeat) when `run` returns by any path. Aborting the supervisor drops its
/// local `JoinSet`, which in turn aborts all search threads; without this the
/// tasks keep an `Arc<Engine>` alive and every reconnect leaks another pool.
struct AbortOnDrop(Vec<tokio::task::JoinHandle<()>>);

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        for h in &self.0 {
            h.abort();
        }
    }
}

/// Streams the engine's scan outcomes to the backend over the session channel.
pub struct GrpcSink {
    tx: mpsc::Sender<WorkerMessage>,
}

impl GrpcSink {
    async fn send(&self, outcome: scan_result::Outcome) {
        let msg = WorkerMessage {
            kind: Some(worker_message::Kind::Result(ScanResult {
                outcome: Some(outcome),
            })),
        };
        if self.tx.send(msg).await.is_err() {
            warn!("session channel closed; dropping scan result");
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
        self.send(scan_result::Outcome::Offline(proto::worker::ServerOffline {
            ip: ip.to_string(),
        }))
        .await;
    }
}

/// Asks the backend which servers to re-probe during an update cycle.
pub struct GrpcTargetSource {
    client: Client,
    worker_id: String,
}

impl GrpcTargetSource {
    pub async fn update_targets(
        &self,
        only_spoofable: bool,
        only_cracked: bool,
        _with_connection: bool,
    ) -> anyhow::Result<Vec<UpdateTarget>> {
        let mut client = self.client.clone();
        let resp = client
            .fetch_update_targets(proto::worker::FetchUpdateTargetsRequest {
                worker_id: self.worker_id.clone(),
                only_spoofable,
                only_cracked,
            })
            .await?
            .into_inner();

        Ok(resp
            .targets
            .into_iter()
            .map(|t| UpdateTarget {
                ip: t.ip,
                port: t.port as u16,
                with_connection: t.with_connection,
            })
            .collect())
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

/// Connects, registers, and runs the session until the stream closes or a
/// shutdown command arrives. Returns `Ok` on a clean shutdown; the caller may
/// retry on `Err`.
pub async fn run(
    cfg: WorkerConfig,
    worker_id: String,
    config_path: PathBuf,
) -> anyhow::Result<()> {
    let backend_url = cfg
        .backend_url
        .clone()
        .ok_or_else(|| anyhow!("[worker].backend_url is required in gRPC mode"))?;

    info!("connecting to backend at {backend_url}");
    let channel = build_channel(&backend_url, &cfg).await?;
    let interceptor = AuthInterceptor {
        token: cfg.token.clone(),
    };
    let client = WorkerControlClient::with_interceptor(channel, interceptor);

    let (msg_tx, msg_rx) = mpsc::channel::<WorkerMessage>(256);

    let sink = GrpcSink { tx: msg_tx.clone() };
    let targets = GrpcTargetSource {
        client: client.clone(),
        worker_id: worker_id.clone(),
    };
    let engine = Engine::new(sink, targets, RuntimeConfig::from(&cfg));

    // Register first so it is the first message the backend sees.
    msg_tx
        .send(WorkerMessage {
            kind: Some(worker_message::Kind::Register(Register {
                worker_id: worker_id.clone(),
                name: cfg.name.clone(),
                config: Some(config_to_proto(&cfg)),
                version: env!("CARGO_PKG_VERSION").to_string(),
            })),
        })
        .await?;

    let mut session_client = client.clone();
    let mut inbound = session_client
        .session(ReceiverStream::new(msg_rx))
        .await?
        .into_inner();

    let mut handles = engine.start();
    handles.push(tokio::spawn(heartbeat(engine.clone(), msg_tx.clone())));
    // Tears the tasks down on every exit path below (clean shutdown, `?`, stream close).
    let _guard = AbortOnDrop(handles);
    info!(worker = %worker_id, "session established");

    while let Some(cmd) = inbound.message().await? {
        match cmd.cmd {
            Some(server_command::Cmd::SetConfig(c)) => {
                persist_config(&config_path, &c);
                engine.set_config(proto_to_runtime(c));
            }
            Some(server_command::Cmd::SetName(s)) => {
                persist_name(&config_path, &s.name);
            }
            Some(server_command::Cmd::Ping(p)) => {
                let engine = engine.clone();
                tokio::spawn(async move {
                    engine.ping(p.ip, p.port as u16, p.with_connection).await;
                });
            }
            Some(server_command::Cmd::Scan(s)) => {
                let engine = engine.clone();
                tokio::spawn(async move {
                    engine.scan(s.ip, s.port as u16).await;
                });
            }
            Some(server_command::Cmd::Control(ctrl)) => {
                match proto::worker::Control::try_from(ctrl).unwrap_or(proto::worker::Control::Unspecified) {
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

async fn heartbeat(engine: Arc<Engine>, tx: mpsc::Sender<WorkerMessage>) {
    let start = Instant::now();
    let mut prev_scanned = 0u64;
    let mut prev_update_done = 0u64;
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
            uptime_secs: start.elapsed().as_secs(),
            searching,
            updating: engine.updating.load(Ordering::Relaxed),
            active_threads: if searching { cfg.threads.max(0) as u32 } else { 0 },
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
