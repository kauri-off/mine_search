//! Transport-agnostic scanning engine: runs the random-search threads and the
//! periodic update cycle against a [`Sink`]/[`TargetSource`], reacting live to
//! config changes pushed through a watch channel. The gRPC session loop drives
//! it via `set_config`/`scan`/`ping`; in diesel mode it just runs the static
//! config from `config.toml`.

use std::{
    net::IpAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};

use rand::{SeedableRng, rngs::SysRng};
use rand_chacha::ChaCha8Rng;
use tokio::{
    sync::{Semaphore, watch},
    task::JoinSet,
    time::timeout,
};
use tracing::{debug, error, info};
use worker::generate_random_ip;

use crate::{
    report::{check_server, probe},
    sink::{RuntimeConfig, Sink, TargetSource},
};

const SEARCH_PORT: u16 = 25565;
const UPDATE_INTERVAL_SECS: u64 = 600;
const UPDATE_CONCURRENCY: usize = 50;

pub struct Engine {
    pub sink: Arc<dyn Sink>,
    pub targets: Arc<dyn TargetSource>,
    cfg_tx: watch::Sender<RuntimeConfig>,
    pause_tx: watch::Sender<bool>,
    pub servers_found: AtomicU64,
    pub ips_scanned: AtomicU64,
    pub updating: AtomicBool,
}

#[allow(dead_code)] // some methods are only driven by the gRPC command loop
impl Engine {
    pub fn new(
        sink: Arc<dyn Sink>,
        targets: Arc<dyn TargetSource>,
        cfg: RuntimeConfig,
    ) -> Arc<Self> {
        let (cfg_tx, _) = watch::channel(cfg);
        let (pause_tx, _) = watch::channel(false);
        Arc::new(Self {
            sink,
            targets,
            cfg_tx,
            pause_tx,
            servers_found: AtomicU64::new(0),
            ips_scanned: AtomicU64::new(0),
            updating: AtomicBool::new(false),
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

    /// Manually pause/resume the search threads (Control commands).
    pub fn set_paused(&self, paused: bool) {
        let _ = self.pause_tx.send(paused);
    }

    /// Launches the search supervisor and the update loop.
    pub fn start(self: &Arc<Self>) {
        tokio::spawn(search_supervisor(self.clone()));
        tokio::spawn(update_loop(self.clone()));
    }

    /// On-demand scan (discovery semantics).
    pub async fn scan(&self, ip: String, port: u16) {
        self.ips_scanned.fetch_add(1, Ordering::Relaxed);
        if let Ok(report) = probe(&ip, port, None, true, true).await {
            self.servers_found.fetch_add(1, Ordering::Relaxed);
            self.sink.discovered(report).await;
        }
    }

    /// On-demand ping / update-cycle probe (update semantics).
    pub async fn ping(&self, ip: String, port: u16, with_connection: bool) {
        match probe(&ip, port, None, with_connection, false).await {
            Ok(report) => self.sink.updated(report).await,
            Err(_) => self.sink.offline(&ip).await,
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
            let _ = timeout(Duration::from_secs(10), async move {
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
            let _ = engine.pause_tx.send(true);
            tokio::time::sleep(Duration::from_secs(20)).await;
        }

        engine.updating.store(true, Ordering::Relaxed);
        info!(target: "updater", "Starting update cycle");

        match engine
            .targets
            .update_targets(
                cfg.only_update_spoofable,
                cfg.only_update_cracked,
                cfg.update_with_connection,
            )
            .await
        {
            Ok(targets) => {
                let semaphore = Arc::new(Semaphore::new(UPDATE_CONCURRENCY));
                let mut handles = Vec::new();
                for t in targets {
                    let permit = semaphore.clone().acquire_owned();
                    let engine = engine.clone();
                    handles.push(tokio::spawn(async move {
                        let _permit = permit.await;
                        engine.ping(t.ip, t.port, t.with_connection).await;
                    }));
                }
                for h in handles {
                    let _ = h.await;
                }
            }
            Err(e) => error!(target: "updater", "Failed to fetch update targets: {}", e),
        }

        engine.updating.store(false, Ordering::Relaxed);
        info!(target: "updater", "Update cycle finished");

        if cfg.search_module {
            let _ = engine.pause_tx.send(false);
            info!(target: "updater", "Resuming search");
        }

        tokio::time::sleep(Duration::from_secs(UPDATE_INTERVAL_SECS)).await;
    }
}
