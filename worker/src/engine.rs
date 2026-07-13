//! Scanning engine: runs the random-search threads and the periodic update cycle,
//! reacting live to config changes pushed through a watch channel. The gRPC session
//! loop drives it via `set_config`/`scan`/`ping`, streams its discoveries through a
//! [`GrpcSink`], and pulls re-probe targets through a [`GrpcTargetSource`].

use std::{
    net::IpAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use rand::{SeedableRng, rngs::SysRng};
use rand_chacha::ChaCha8Rng;
use tokio::{
    sync::{Notify, Semaphore, watch},
    task::JoinSet,
    time::timeout,
};
use tokio_stream::StreamExt;
use tracing::{debug, error, info};
use worker::generate_random_ip;

use crate::{
    grpc_backend::{GrpcSink, GrpcTargetSource},
    report::{ScanReport, check_server, probe},
};

/// Live-tunable subset of the worker config (mirrors `[worker]` and the gRPC
/// `WorkerConfig` message). Pushed through a watch channel so the engine can
/// react to changes without a restart.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeConfig {
    pub threads: i32,
    pub search_module: bool,
    pub update_module: bool,
    /// Whether the update cycle re-probes with a login connection. Not consumed by
    /// the engine directly (the backend stamps `with_connection` onto each update
    /// target from the registered config), but tracked here so a UI change to this
    /// knob is detected as a config change — and thus persisted to the worker's
    /// config file — instead of being silently dropped.
    pub update_with_connection: bool,
    pub update_interval_secs: u32,
    pub update_concurrency: u32,
    /// Which existing servers the update cycle re-probes (sent to the backend).
    pub update_filter: proto::worker::ServerFilter,
    /// Acceptance filter applied to freshly discovered servers before reporting.
    pub search_filter: proto::worker::ServerFilter,
}

impl From<&crate::config::WorkerConfig> for RuntimeConfig {
    fn from(c: &crate::config::WorkerConfig) -> Self {
        Self {
            threads: c.threads,
            search_module: c.search_module,
            update_module: c.update_module,
            update_with_connection: c.update_with_connection,
            update_interval_secs: c.update_interval_secs,
            update_concurrency: c.update_concurrency,
            update_filter: crate::grpc_backend::filter_to_proto(&c.update_filter),
            search_filter: crate::grpc_backend::filter_to_proto(&c.search_filter),
        }
    }
}

/// Whether a freshly discovered server passes the search-module acceptance
/// filter. Only fields observable at discovery time are checked: online-mode
/// (`licensed`, from the login handshake that discovery always performs),
/// `requires_mods`, and `has_players`. An unset filter field accepts anything.
fn accept_discovery(report: &ScanReport, f: &proto::worker::ServerFilter) -> bool {
    if let Some(want) = f.licensed {
        let is_online_mode = report.extra.as_ref().is_some_and(|e| e.is_online_mode);
        if is_online_mode != want {
            return false;
        }
    }
    if let Some(want) = f.requires_mods {
        if report.requires_mods != want {
            return false;
        }
    }
    if let Some(want) = f.has_players {
        if (report.players_online > 0) != want {
            return false;
        }
    }
    true
}

/// One server the worker should re-probe during an update cycle.
#[derive(Debug, Clone)]
pub struct UpdateTarget {
    pub ip: String,
    pub port: u16,
    pub with_connection: bool,
}

/// One frame of the update-target stream: either the leading total (count of
/// servers this cycle will re-probe) or a single target.
#[derive(Debug, Clone)]
pub enum UpdateTargetItem {
    Total(u64),
    Target(UpdateTarget),
}

const SEARCH_PORT: u16 = 25565;
const DEFAULT_UPDATE_INTERVAL_SECS: u64 = 600;
const DEFAULT_UPDATE_CONCURRENCY: usize = 50;
/// Wait before retrying a cycle whose target fetch failed (e.g. the backend link
/// was momentarily down, as can happen on a cold start when the update loop
/// spins up before the session). Much shorter than a normal interval so updates
/// begin promptly once the link is up, instead of stalling for a full interval.
const RETRY_INTERVAL_SECS: u64 = 10;
/// Per-server budget for a full probe (status + optional login handshake).
/// The underlying socket reads have no timeout of their own, so this is the
/// only thing bounding a server that accepts the TCP connection but stalls.
const PROBE_TIMEOUT: Duration = Duration::from_secs(10);

pub struct Engine {
    pub sink: GrpcSink,
    pub targets: GrpcTargetSource,
    /// Process start, used for the heartbeat's uptime. Lives with the engine (not
    /// the session) so uptime reflects the worker, not the current connection.
    pub started: Instant,
    cfg_tx: watch::Sender<RuntimeConfig>,
    /// Effective pause signal the search threads observe: `manual_paused ||
    /// updater_paused`. Recomputed via [`Engine::refresh_pause`] whenever either
    /// source changes, so the update cycle can pause/resume search without
    /// clobbering an operator's manual pause.
    pause_tx: watch::Sender<bool>,
    /// Operator-requested pause (Control::Pause/ResumeSearch).
    manual_paused: AtomicBool,
    /// Search paused because an update cycle is running.
    updater_paused: AtomicBool,
    /// Fired to start an update cycle immediately, cutting short the interval wait.
    trigger_update: Notify,
    /// Fired to interrupt the running update cycle.
    abort_update: Notify,
    pub servers_found: AtomicU64,
    pub ips_scanned: AtomicU64,
    pub updating: AtomicBool,
    pub update_done: AtomicU64,
    pub update_total: AtomicU64,
    pub last_update_unix: AtomicI64,
}

#[allow(dead_code)] // some methods are only driven by the gRPC command loop
impl Engine {
    pub fn new(sink: GrpcSink, targets: GrpcTargetSource, cfg: RuntimeConfig) -> Arc<Self> {
        let (cfg_tx, _) = watch::channel(cfg);
        let (pause_tx, _) = watch::channel(false);
        Arc::new(Self {
            sink,
            targets,
            started: Instant::now(),
            cfg_tx,
            pause_tx,
            manual_paused: AtomicBool::new(false),
            updater_paused: AtomicBool::new(false),
            trigger_update: Notify::new(),
            abort_update: Notify::new(),
            servers_found: AtomicU64::new(0),
            ips_scanned: AtomicU64::new(0),
            updating: AtomicBool::new(false),
            update_done: AtomicU64::new(0),
            update_total: AtomicU64::new(0),
            last_update_unix: AtomicI64::new(0),
        })
    }

    pub fn config(&self) -> RuntimeConfig {
        self.cfg_tx.borrow().clone()
    }

    pub fn set_config(&self, cfg: RuntimeConfig) {
        // Skip identical config: the backend re-pushes its pinned config on every
        // (re)connect, and pushing it through the watch would needlessly rebuild
        // the search pool ("reconfiguring") even when nothing changed.
        if *self.cfg_tx.borrow() == cfg {
            return;
        }
        info!("applying config: {:?}", cfg);
        let _ = self.cfg_tx.send(cfg);
    }

    pub fn searching(&self) -> bool {
        self.config().search_module && !*self.pause_tx.borrow()
    }

    /// Manually pause/resume the search threads (Control commands). Independent
    /// of the updater's pause, so an update cycle finishing won't resume search
    /// that the operator paused.
    pub fn set_paused(&self, paused: bool) {
        self.manual_paused.store(paused, Ordering::Relaxed);
        self.refresh_pause();
    }

    /// Recomputes the effective pause (`manual || updater`) and publishes it to
    /// the search threads.
    fn refresh_pause(&self) {
        let effective = self.manual_paused.load(Ordering::Relaxed)
            || self.updater_paused.load(Ordering::Relaxed);
        let _ = self.pause_tx.send(effective);
    }

    /// Pause/resume search on behalf of the update cycle (leaves a manual pause intact).
    fn set_updater_paused(&self, paused: bool) {
        self.updater_paused.store(paused, Ordering::Relaxed);
        self.refresh_pause();
    }

    /// Start an update cycle now, cutting short the inter-cycle interval wait.
    pub fn trigger_update(&self) {
        self.trigger_update.notify_one();
    }

    /// Interrupt the running update cycle (in-flight probes finish).
    pub fn abort_update(&self) {
        self.abort_update.notify_one();
    }

    /// Launches the search supervisor and the update loop, returning their task
    /// handles so the caller can abort them on shutdown/reconnect (otherwise the
    /// tasks keep the `Engine` alive forever, leaking the whole search pool).
    pub fn start(self: &Arc<Self>) -> Vec<tokio::task::JoinHandle<()>> {
        vec![
            tokio::spawn(search_supervisor(self.clone())),
            tokio::spawn(update_loop(self.clone())),
        ]
    }

    /// On-demand scan (discovery semantics).
    ///
    /// Bounded by [`PROBE_TIMEOUT`] like [`Engine::ping`]: a server that accepts
    /// the TCP connection but never replies must not leak the task forever (the
    /// underlying socket reads have no timeout of their own).
    pub async fn scan(&self, ip: String, port: u16) {
        self.ips_scanned.fetch_add(1, Ordering::Relaxed);
        if let Ok(Ok(report)) = timeout(PROBE_TIMEOUT, probe(&ip, port, None, true, true)).await {
            self.servers_found.fetch_add(1, Ordering::Relaxed);
            self.sink.discovered(report).await;
        }
    }

    /// On-demand ping / update-cycle probe (update semantics).
    ///
    /// Bounded by a per-server [`PROBE_TIMEOUT`]: an unresponsive server (TCP
    /// accepts but never replies) must not block its update slot until the OS
    /// connection timeout fires. A timeout is treated as offline.
    pub async fn ping(&self, ip: String, port: u16, with_connection: bool) {
        match timeout(
            PROBE_TIMEOUT,
            probe(&ip, port, None, with_connection, false),
        )
        .await
        {
            Ok(Ok(report)) => self.sink.updated(report).await,
            Ok(Err(_)) | Err(_) => self.sink.offline(&ip).await,
        }
    }
}

async fn search_supervisor(engine: Arc<Engine>) {
    let mut cfg_rx = engine.cfg_tx.subscribe();
    loop {
        let cfg = cfg_rx.borrow_and_update().clone();

        if cfg.search_module && cfg.threads > 0 {
            let mut set = JoinSet::new();
            for _ in 0..cfg.threads {
                set.spawn(search_thread(engine.clone(), engine.pause_tx.subscribe()));
            }
            info!("search: {} threads running", cfg.threads);

            // Block until the config changes, then rebuild the pool.
            if cfg_rx.changed().await.is_err() {
                set.abort_all();
                return;
            }
            set.abort_all();
            while set.join_next().await.is_some() {}
            info!("search: reconfiguring");
        } else if cfg_rx.changed().await.is_err() {
            return;
        }
    }
}

async fn search_thread(engine: Arc<Engine>, mut pause_rx: watch::Receiver<bool>) {
    let mut rng =
        ChaCha8Rng::try_from_rng(&mut SysRng).expect("Failed to seed RNG from system entropy");

    loop {
        if *pause_rx.borrow() {
            if pause_rx.changed().await.is_err() {
                return;
            }
            continue;
        }

        let ip = IpAddr::V4(generate_random_ip(&mut rng)).to_string();
        engine.ips_scanned.fetch_add(1, Ordering::Relaxed);

        if let Ok(stream) = check_server(&ip, SEARCH_PORT).await {
            debug!("Potential server found at {}:{}", ip, SEARCH_PORT);
            let engine = engine.clone();
            let _ = timeout(PROBE_TIMEOUT, async move {
                match probe(&ip, SEARCH_PORT, Some(stream), true, true).await {
                    Ok(report) => {
                        // Drop discoveries that don't match the search-module
                        // acceptance filter before counting or reporting them.
                        if !accept_discovery(&report, &engine.config().search_filter) {
                            debug!("Discovery {}:{} filtered out", ip, SEARCH_PORT);
                            return;
                        }
                        engine.servers_found.fetch_add(1, Ordering::Relaxed);
                        info!(
                            target: "server_found",
                            ip = %report.ip,
                            port = report.port,
                            version = %report.version_name,
                            online = report.players_online,
                            max = report.players_max,
                            "New server detected"
                        );
                        engine.sink.discovered(report).await;
                    }
                    Err(e) => debug!("Failed to process {}:{} | {}", ip, SEARCH_PORT, e),
                }
            })
            .await;
        }
    }
}

/// Blocks while the update module is disabled, returning only when a cycle should
/// run anyway: either a manual trigger fired (an operator forcing a one-off
/// update), or the module was just switched on. Returns `false` when the config
/// channel closes (the engine is shutting down), so the caller can exit.
///
/// This is the fix for "a manual update does nothing while the update module is
/// off": the old loop, when disabled, slept in a fixed 5s cycle and never
/// observed `trigger_update`, so `CONTROL_TRIGGER_UPDATE` was ignored.
async fn park_while_disabled(
    trigger: &Notify,
    cfg_rx: &mut watch::Receiver<RuntimeConfig>,
) -> bool {
    loop {
        tokio::select! {
            _ = trigger.notified() => return true,
            res = cfg_rx.changed() => {
                if res.is_err() {
                    return false;
                }
                // Any config change wakes us; run a cycle only if the module was
                // actually turned on, otherwise keep parking (e.g. an unrelated
                // change to `threads` must not start an update).
                if cfg_rx.borrow().update_module {
                    return true;
                }
            }
        }
    }
}

async fn update_loop(engine: Arc<Engine>) {
    let mut cfg_rx = engine.cfg_tx.subscribe();
    // Brief settle so the first cycle on a cold start doesn't race the backend
    // session coming up: the engine (and this loop) is spawned before `run`
    // establishes the link, so an immediate fetch would otherwise fail.
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Run a cycle immediately the first time the module is enabled (matches the
    // old "enabled → starts scanning shortly" behavior). While disabled we wait
    // for an explicit trigger before the first run.
    let mut run_now = engine.config().update_module;
    // Whether the previous cycle managed to fetch targets. A failed fetch retries
    // after `RETRY_INTERVAL_SECS` rather than a full update interval.
    let mut last_fetched = true;

    loop {
        if !run_now {
            let cfg = engine.config();
            if cfg.update_module {
                // Enabled: run on the inter-cycle interval, an early manual
                // trigger, or bail out to re-evaluate on any config change (so a
                // disable takes effect promptly and an interval change is picked
                // up without waiting out the old interval).
                let interval = if !last_fetched {
                    RETRY_INTERVAL_SECS
                } else if cfg.update_interval_secs == 0 {
                    DEFAULT_UPDATE_INTERVAL_SECS
                } else {
                    cfg.update_interval_secs as u64
                };
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(interval)) => {}
                    _ = engine.trigger_update.notified() => {
                        info!(target: "updater", "Update triggered manually");
                    }
                    res = cfg_rx.changed() => {
                        if res.is_err() {
                            return;
                        }
                        // Config changed: re-evaluate from the top without running
                        // a cycle (the module may have just been disabled).
                        continue;
                    }
                }
            } else {
                // Disabled: only a manual trigger (or the module being enabled)
                // starts a cycle.
                if !park_while_disabled(&engine.trigger_update, &mut cfg_rx).await {
                    return;
                }
            }
        }
        run_now = false;

        last_fetched = run_update_cycle(&engine).await;
    }
}

/// Runs a single update cycle: pauses search if it is active, streams re-probe
/// targets from the backend and probes them through a bounded pool, then resumes
/// search. Interruptible via [`Engine::abort_update`]. Returns whether the target
/// stream was successfully opened (a failed fetch shortens the next wait).
async fn run_update_cycle(engine: &Arc<Engine>) -> bool {
    let cfg = engine.config();

    if cfg.search_module {
        info!(target: "updater", "Pausing search");
        engine.set_updater_paused(true);
        tokio::time::sleep(Duration::from_secs(20)).await;
    }

    engine.updating.store(true, Ordering::Relaxed);
    info!(target: "updater", "Starting update cycle");

    engine.update_total.store(0, Ordering::Relaxed);
    engine.update_done.store(0, Ordering::Relaxed);
    let concurrency = if cfg.update_concurrency == 0 {
        DEFAULT_UPDATE_CONCURRENCY
    } else {
        cfg.update_concurrency as usize
    };

    let fetched = match engine
        .targets
        .update_targets(cfg.update_filter.clone())
        .await
    {
        Ok(stream) => {
            // Consume the target stream through a fixed-size pool. We acquire a
            // permit *before* pulling the next target, so at most `concurrency`
            // probes are in flight and at most `concurrency` tasks are ever
            // alive — no matter how many servers the backend streams (the old
            // code spawned one task per target up front). Reserved/invalid
            // addresses are dropped so a compromised backend can't aim the
            // worker at internal hosts. `update_total` is set once from the
            // stream's leading frame (the backend's row count) so progress has a
            // fixed denominator; skipped addresses still count as done so
            // `update_done` reaches `update_total` at the end of the cycle.
            let dispatch_engine = engine.clone();
            let dispatcher = tokio::spawn(async move {
                let semaphore = Arc::new(Semaphore::new(concurrency));
                let mut set = JoinSet::new();
                tokio::pin!(stream);
                while let Some(item) = stream.next().await {
                    let t = match item {
                        Ok(UpdateTargetItem::Total(n)) => {
                            dispatch_engine.update_total.store(n, Ordering::Relaxed);
                            continue;
                        }
                        Ok(UpdateTargetItem::Target(t)) => t,
                        Err(e) => {
                            error!(target: "updater", "target stream error: {}", e);
                            break;
                        }
                    };
                    if !worker::is_probeable_ip(&t.ip) {
                        dispatch_engine.update_done.fetch_add(1, Ordering::Relaxed);
                        continue;
                    }
                    let Ok(permit) = semaphore.clone().acquire_owned().await else {
                        break;
                    };
                    let engine = dispatch_engine.clone();
                    set.spawn(async move {
                        let _permit = permit;
                        engine.ping(t.ip, t.port, t.with_connection).await;
                        engine.update_done.fetch_add(1, Ordering::Relaxed);
                    });
                }
                while set.join_next().await.is_some() {}
            });

            let abort = dispatcher.abort_handle();
            tokio::select! {
                _ = async { let _ = dispatcher.await; } => {}
                _ = engine.abort_update.notified() => {
                    abort.abort();
                    info!(target: "updater", "Update cycle aborted");
                }
            }
            true
        }
        Err(e) => {
            error!(target: "updater", "Failed to fetch update targets: {}", e);
            false
        }
    };

    engine.updating.store(false, Ordering::Relaxed);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    engine.last_update_unix.store(now, Ordering::Relaxed);
    info!(target: "updater", "Update cycle finished");

    if cfg.search_module {
        engine.set_updater_paused(false);
        info!(target: "updater", "Resuming search");
    }

    fetched
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{Duration, timeout};

    fn rc(update_module: bool, threads: i32) -> RuntimeConfig {
        RuntimeConfig {
            threads,
            search_module: false,
            update_module,
            update_with_connection: false,
            update_interval_secs: 600,
            update_concurrency: 50,
            update_filter: proto::worker::ServerFilter::default(),
            search_filter: proto::worker::ServerFilter::default(),
        }
    }

    // The core of the "manual update does nothing while the update module is off"
    // bug: a trigger fired while disabled must wake the parked updater so a cycle
    // runs. Before the fix, the disabled branch never observed `trigger_update`.
    #[tokio::test]
    async fn park_returns_on_manual_trigger_when_disabled() {
        let trigger = Notify::new();
        let (_tx, rx) = watch::channel(rc(false, 0));
        let mut rx = rx;
        trigger.notify_one();
        let woke = timeout(Duration::from_millis(500), park_while_disabled(&trigger, &mut rx))
            .await
            .expect("park should return promptly after a manual trigger");
        assert!(woke, "a manual trigger should warrant a cycle");
    }

    // Enabling the module while parked must also start a cycle, matching the old
    // "enabled → starts scanning shortly" behavior.
    #[tokio::test]
    async fn park_returns_when_module_enabled() {
        let trigger = Notify::new();
        let (tx, rx) = watch::channel(rc(false, 0));
        let mut rx = rx;
        tx.send(rc(true, 0)).unwrap();
        let woke = timeout(Duration::from_millis(500), park_while_disabled(&trigger, &mut rx))
            .await
            .expect("park should return once the module is enabled");
        assert!(woke);
    }

    // An unrelated config change (module still off) must NOT start a cycle: only a
    // trigger or an actual enable does.
    #[tokio::test]
    async fn park_ignores_unrelated_change_while_disabled() {
        let trigger = Notify::new();
        let (tx, rx) = watch::channel(rc(false, 0));
        let mut rx = rx;

        // A change that leaves the module disabled must keep us parked.
        tx.send(rc(false, 8)).unwrap();
        let still_parked =
            timeout(Duration::from_millis(150), park_while_disabled(&trigger, &mut rx)).await;
        assert!(
            still_parked.is_err(),
            "an unrelated change must not start an update cycle"
        );

        // A subsequent trigger must then wake it.
        trigger.notify_one();
        let woke = timeout(Duration::from_millis(500), park_while_disabled(&trigger, &mut rx))
            .await
            .expect("park should return after the trigger");
        assert!(woke);
    }

    // When the engine shuts down the config sender is dropped; the parked updater
    // must observe the closed channel and exit instead of spinning.
    #[tokio::test]
    async fn park_returns_false_when_config_channel_closes() {
        let trigger = Notify::new();
        let (tx, rx) = watch::channel(rc(false, 0));
        let mut rx = rx;
        drop(tx);
        let woke = timeout(Duration::from_millis(500), park_while_disabled(&trigger, &mut rx))
            .await
            .expect("park should return when the channel closes");
        assert!(!woke, "a closed channel means shut down, not run a cycle");
    }
}
