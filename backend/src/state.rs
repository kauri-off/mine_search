use std::sync::Arc;

use crate::{database::DatabaseWrapper, events::ServerEvents, registry::WorkerRegistry};

/// Shared state for both gRPC services.
pub struct AppState {
    pub db: Arc<DatabaseWrapper>,
    pub registry: Arc<WorkerRegistry>,
    pub events: Arc<ServerEvents>,
}
