//! WorkerControl gRPC service: the worker-facing control plane. Workers dial in
//! and open the `Session` stream; the backend persists their results and pushes
//! commands. `FetchUpdateTargets` serves the per-cycle re-probe list.

use std::{pin::Pin, sync::Arc};

use futures::Stream;
use proto::worker::{
    FetchUpdateTargetsRequest, ScanResult, ServerCommand, UpdateTargets, WorkerMessage,
    scan_result, server_command, worker_control_server::WorkerControl, worker_message,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};

use crate::{auth, persistence, state::AppState};

pub struct WorkerService {
    pub state: Arc<AppState>,
}

#[tonic::async_trait]
impl WorkerControl for WorkerService {
    type SessionStream = Pin<Box<dyn Stream<Item = Result<ServerCommand, Status>> + Send>>;

    async fn session(
        &self,
        request: Request<Streaming<WorkerMessage>>,
    ) -> Result<Response<Self::SessionStream>, Status> {
        auth::require_worker_token(&request)?;

        let mut inbound = request.into_inner();
        let (cmd_tx, cmd_rx) = mpsc::channel::<Result<ServerCommand, Status>>(128);
        let state = self.state.clone();

        tokio::spawn(async move {
            let mut worker_id: Option<String> = None;

            loop {
                match inbound.message().await {
                    Ok(Some(msg)) => match msg.kind {
                        Some(worker_message::Kind::Register(reg)) => {
                            let id = reg.worker_id.clone();
                            let config = reg.config.unwrap_or_default();
                            let effective = state
                                .registry
                                .register(id.clone(), reg.name, reg.version, config, cmd_tx.clone())
                                .await;
                            worker_id = Some(id.clone());
                            tracing::info!(worker = %id, "worker registered");
                            // Echo the effective config so a worker re-tuned by the
                            // operator while it was offline picks the change back up.
                            let _ = cmd_tx
                                .send(Ok(ServerCommand {
                                    cmd: Some(server_command::Cmd::SetConfig(effective)),
                                }))
                                .await;
                        }
                        Some(worker_message::Kind::Heartbeat(hb)) => {
                            if let (Some(id), Some(metrics)) = (worker_id.as_ref(), hb.metrics) {
                                state.registry.heartbeat(id, metrics).await;
                            }
                        }
                        Some(worker_message::Kind::Result(result)) => {
                            handle_result(&state, result).await;
                        }
                        None => {}
                    },
                    Ok(None) => break,
                    Err(e) => {
                        tracing::debug!("worker stream error: {e}");
                        break;
                    }
                }
            }

            if let Some(id) = worker_id {
                tracing::info!(worker = %id, "worker disconnected");
                state.registry.mark_offline(&id).await;
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(cmd_rx))))
    }

    async fn fetch_update_targets(
        &self,
        request: Request<FetchUpdateTargetsRequest>,
    ) -> Result<Response<UpdateTargets>, Status> {
        auth::require_worker_token(&request)?;
        let req = request.into_inner();

        let with_connection = self
            .state
            .registry
            .get(&req.worker_id)
            .await
            .ok()
            .and_then(|w| w.config)
            .map(|c| c.update_with_connection)
            .unwrap_or(false);

        let targets = persistence::fetch_update_targets(
            &self.state.db,
            req.only_spoofable,
            req.only_cracked,
            with_connection,
        )
        .await
        .map_err(|e| Status::internal(format!("db error: {e}")))?;

        Ok(Response::new(UpdateTargets { targets }))
    }
}

async fn handle_result(state: &AppState, result: ScanResult) {
    let outcome = match result.outcome {
        Some(scan_result::Outcome::Discovered(s)) => persistence::persist_discovered(&state.db, s).await,
        Some(scan_result::Outcome::Updated(s)) => persistence::persist_updated(&state.db, s).await,
        Some(scan_result::Outcome::Offline(o)) => persistence::persist_offline(&state.db, &o.ip).await,
        None => Ok(None),
    };
    match outcome {
        Ok(Some(server_id)) => state.events.notify(server_id),
        Ok(None) => {}
        Err(e) => tracing::error!("failed to persist scan result: {e}"),
    }
}
