//! In-memory registry of connected workers. Each worker that opens a `Session`
//! stream gets a [`WorkerHandle`] holding its latest config/metrics and an
//! outbound command channel. The frontend's worker-management RPCs read and
//! mutate this registry; `dispatch_*` routes scan/ping work to a live worker.

use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

use chrono::Utc;
use proto::{
    api::{WorkerInfo, WorkerList},
    worker::{PingTask, ScanTask, ServerCommand, WorkerConfig, WorkerMetrics, server_command},
};
use tokio::sync::{RwLock, mpsc};
use tonic::Status;

pub struct WorkerHandle {
    pub name: Option<String>,
    pub version: String,
    pub online: bool,
    pub last_seen: i64,
    pub config: WorkerConfig,
    pub metrics: Option<WorkerMetrics>,
    pub cmd_tx: mpsc::Sender<Result<ServerCommand, Status>>,
}

#[derive(Default)]
pub struct WorkerRegistry {
    workers: RwLock<HashMap<String, WorkerHandle>>,
    /// Config the operator wants applied, retained across reconnects so a worker
    /// that drops and comes back is re-tuned to its last requested settings.
    desired: RwLock<HashMap<String, WorkerConfig>>,
    rr: AtomicUsize,
}

fn now() -> i64 {
    Utc::now().timestamp()
}

impl WorkerRegistry {
    /// Registers (or replaces) a worker. Returns the config the worker should
    /// actually run: its own reported config, unless the operator previously
    /// pinned a different one via the UI.
    pub async fn register(
        &self,
        id: String,
        name: Option<String>,
        version: String,
        reported: WorkerConfig,
        cmd_tx: mpsc::Sender<Result<ServerCommand, Status>>,
    ) -> WorkerConfig {
        let effective = self
            .desired
            .read()
            .await
            .get(&id)
            .cloned()
            .unwrap_or(reported);

        let mut workers = self.workers.write().await;
        workers.insert(
            id,
            WorkerHandle {
                name,
                version,
                online: true,
                last_seen: now(),
                config: effective.clone(),
                metrics: None,
                cmd_tx,
            },
        );
        effective
    }

    pub async fn mark_offline(&self, id: &str) {
        if let Some(h) = self.workers.write().await.get_mut(id) {
            h.online = false;
            h.last_seen = now();
        }
    }

    pub async fn heartbeat(&self, id: &str, metrics: WorkerMetrics) {
        if let Some(h) = self.workers.write().await.get_mut(id) {
            h.metrics = Some(metrics);
            h.last_seen = now();
            h.online = true;
        }
    }

    /// Records the operator's desired config and pushes it to the worker if it
    /// is currently connected. Returns an error if the worker is unknown.
    pub async fn set_config(&self, id: &str, config: WorkerConfig) -> Result<(), Status> {
        self.desired
            .write()
            .await
            .insert(id.to_string(), config.clone());

        let tx = {
            let mut workers = self.workers.write().await;
            let handle = workers
                .get_mut(id)
                .ok_or_else(|| Status::not_found("unknown worker"))?;
            handle.config = config.clone();
            handle.cmd_tx.clone()
        };

        tx.send(Ok(ServerCommand {
            cmd: Some(server_command::Cmd::SetConfig(config)),
        }))
        .await
        .map_err(|_| Status::unavailable("worker is offline"))
    }

    pub async fn list(&self) -> WorkerList {
        let workers = self.workers.read().await;
        WorkerList {
            workers: workers.iter().map(|(id, h)| h.to_info(id)).collect(),
        }
    }

    pub async fn get(&self, id: &str) -> Result<WorkerInfo, Status> {
        self.workers
            .read()
            .await
            .get(id)
            .map(|h| h.to_info(id))
            .ok_or_else(|| Status::not_found("unknown worker"))
    }

    /// Sends a command to a single live worker, chosen round-robin. Errors when
    /// no worker is currently connected.
    async fn dispatch(&self, cmd: server_command::Cmd) -> Result<(), Status> {
        let tx = {
            let workers = self.workers.read().await;
            let online: Vec<&mpsc::Sender<Result<ServerCommand, Status>>> = workers
                .values()
                .filter(|h| h.online)
                .map(|h| &h.cmd_tx)
                .collect();
            if online.is_empty() {
                return Err(Status::unavailable("no workers connected"));
            }
            let idx = self.rr.fetch_add(1, Ordering::Relaxed) % online.len();
            online[idx].clone()
        };

        tx.send(Ok(ServerCommand { cmd: Some(cmd) }))
            .await
            .map_err(|_| Status::unavailable("worker disconnected"))
    }

    /// Sends a command to a specific worker by id. Errors when the worker is
    /// unknown or currently offline.
    async fn dispatch_to(&self, worker_id: &str, cmd: server_command::Cmd) -> Result<(), Status> {
        let tx = {
            let workers = self.workers.read().await;
            let handle = workers
                .get(worker_id)
                .ok_or_else(|| Status::not_found("unknown worker"))?;
            if !handle.online {
                return Err(Status::unavailable("worker offline"));
            }
            handle.cmd_tx.clone()
        };

        tx.send(Ok(ServerCommand { cmd: Some(cmd) }))
            .await
            .map_err(|_| Status::unavailable("worker disconnected"))
    }

    pub async fn dispatch_ping(
        &self,
        worker_id: &str,
        ip: String,
        port: i32,
        with_connection: bool,
    ) -> Result<(), Status> {
        self.dispatch_to(
            worker_id,
            server_command::Cmd::Ping(PingTask {
                ip,
                port,
                with_connection,
            }),
        )
        .await
    }

    pub async fn dispatch_scan(&self, ip: String, port: i32) -> Result<(), Status> {
        self.dispatch(server_command::Cmd::Scan(ScanTask { ip, port }))
            .await
    }

    /// Sends a parameterless control command (pause/resume search, abort/trigger
    /// update) to a specific worker. `control` is the `worker.Control` enum value.
    pub async fn send_control(&self, worker_id: &str, control: i32) -> Result<(), Status> {
        self.dispatch_to(worker_id, server_command::Cmd::Control(control))
            .await
    }
}

impl WorkerHandle {
    fn to_info(&self, id: &str) -> WorkerInfo {
        WorkerInfo {
            worker_id: id.to_string(),
            name: self.name.clone(),
            version: self.version.clone(),
            online: self.online,
            last_seen_unix: self.last_seen,
            config: Some(self.config.clone()),
            metrics: self.metrics.clone(),
        }
    }
}
