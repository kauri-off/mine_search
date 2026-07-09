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
    time::{Duration, SystemTime, UNIX_EPOCH},
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
    report::{check_server, probe},
};

/// Live-tunable subset of the worker config (mirrors `[worker]` and the gRPC
/// `WorkerConfig` message). Pushed through a watch channel so the engine can
/// react to changes without a restart.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub threads: i32,
    pub search_module: bool,
    pub update_module: bool,
    pub only_update_spoofable: bool,
    pub only_update_cracked: bool,
    pub update_interval_secs: u32,
    pub update_concurrency: u32,
}

impl From<&crate::config::WorkerConfig> for RuntimeConfig {
    fn from(c: &crate::config::WorkerConfig) -> Self {
        Self {
            threads: c.threads,
            search_module: c.search_module,
            update_module: c.update_module,
            only_update_spoofable: c.only_update_spoofable,
            only_update_cracked: c.only_update_cracked,
            update_interval_secs: c.update_interval_secs,
            update_concurrency: c.update_concurrency,
        }
    }
}

/// One server the worker should re-probe during an update cycle.
#[derive(Debug, Clone)]
pub struct UpdateTarget {
    pub ip: String,
    pub port: u16,
    pub with_connection: bool,
}

const SEARCH_PORT: u16 = 25565;
const DEFAULT_UPDATE_INTERVAL_SECS: u64 = 600;
const DEFAULT_UPDATE_CONCURRENCY: usize = 50;
/// Per-server budget for a full probe (status + optional login handshake).
/// The underlying socket reads have no timeout of their own, so this is the
/// only thing bounding a server that accepts the TCP connection but stalls.
const PROBE_TIMEOUT: Duration = Duration::from_secs(10);

pub struct Engine {
    pub sink: GrpcSink,
    pub targets: GrpcTargetSource,
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
        match timeout(PROBE_TIMEOUT, probe(&ip, port, None, with_connection, false)).await {
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

async fn update_loop(engine: Arc<Engine>) {
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let cfg = engine.config();
        if !cfg.update_module {
            tokio::time::sleep(Duration::from_secs(5)).await;
            continue;
        }

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

        match engine
            .targets
            .update_targets(cfg.only_update_spoofable, cfg.only_update_cracked)
            .await
        {
            Ok(stream) => {
                // Consume the target stream through a fixed-size pool. We acquire a
                // permit *before* pulling the next target, so at most `concurrency`
                // probes are in flight and at most `concurrency` tasks are ever
                // alive — no matter how many servers the backend streams (the old
                // code spawned one task per target up front). Reserved/invalid
                // addresses are dropped so a compromised backend can't aim the
                // worker at internal hosts. `update_total` now counts probeable
                // targets as they arrive rather than being known up front.
                let dispatch_engine = engine.clone();
                let dispatcher = tokio::spawn(async move {
                    let semaphore = Arc::new(Semaphore::new(concurrency));
                    let mut set = JoinSet::new();
                    tokio::pin!(stream);
                    while let Some(item) = stream.next().await {
                        let t = match item {
                            Ok(t) => t,
                            Err(e) => {
                                error!(target: "updater", "target stream error: {}", e);
                                break;
                            }
                        };
                        if !worker::is_probeable_ip(&t.ip) {
                            continue;
                        }
                        dispatch_engine.update_total.fetch_add(1, Ordering::Relaxed);
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
            }
            Err(e) => error!(target: "updater", "Failed to fetch update targets: {}", e),
        }

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

        let interval = if cfg.update_interval_secs == 0 {
            DEFAULT_UPDATE_INTERVAL_SECS
        } else {
            cfg.update_interval_secs as u64
        };
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(interval)) => {}
            _ = engine.trigger_update.notified() => {
                info!(target: "updater", "Update triggered manually");
            }
        }
    }
}
