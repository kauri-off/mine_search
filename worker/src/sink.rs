//! Abstractions that decouple the scanning engine from the transport. The
//! `grpc` feature implements these by streaming to / pulling from the backend;
//! the `diesel` feature implements them with direct database access.

use async_trait::async_trait;

use crate::report::ScanReport;

/// Where scan results go.
#[async_trait]
pub trait Sink: Send + Sync {
    /// A server found via the search/scan path (upsert-by-ip).
    async fn discovered(&self, report: ScanReport);
    /// A server re-probed during an update cycle / on-demand ping (full update).
    async fn updated(&self, report: ScanReport);
    /// A server that failed re-probing: mark offline.
    async fn offline(&self, ip: &str);
}

/// One server the worker should re-probe during an update cycle.
#[derive(Debug, Clone)]
pub struct UpdateTarget {
    pub ip: String,
    pub port: u16,
    pub with_connection: bool,
}

/// Supplies the set of servers to re-probe. In diesel mode this reads the DB; in
/// gRPC mode it asks the backend.
#[async_trait]
pub trait TargetSource: Send + Sync {
    async fn update_targets(
        &self,
        only_spoofable: bool,
        only_cracked: bool,
        with_connection: bool,
    ) -> anyhow::Result<Vec<UpdateTarget>>;
}

/// Live-tunable subset of the worker config (mirrors `[worker]` and the gRPC
/// `WorkerConfig` message). Pushed through a watch channel so the engine can
/// react to changes without a restart.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub threads: i32,
    pub search_module: bool,
    pub update_module: bool,
    pub update_with_connection: bool,
    pub only_update_spoofable: bool,
    pub only_update_cracked: bool,
}

impl From<&crate::config::WorkerConfig> for RuntimeConfig {
    fn from(c: &crate::config::WorkerConfig) -> Self {
        Self {
            threads: c.threads,
            search_module: c.search_module,
            update_module: c.update_module,
            update_with_connection: c.update_with_connection,
            only_update_spoofable: c.only_update_spoofable,
            only_update_cracked: c.only_update_cracked,
        }
    }
}
