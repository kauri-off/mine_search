use std::sync::Arc;

use crate::{database::DatabaseWrapper, events::ServerEvents, registry::WorkerRegistry};

/// Resolved watchtower HTTP API settings, present only when both URL and token
/// are configured. Drives the manual "update stack" action.
pub struct WatchtowerConfig {
    pub url: String,
    pub token: String,
}

/// Shared state for both gRPC services.
pub struct AppState {
    pub db: Arc<DatabaseWrapper>,
    pub registry: Arc<WorkerRegistry>,
    pub events: Arc<ServerEvents>,
    pub watchtower: Option<WatchtowerConfig>,
}
