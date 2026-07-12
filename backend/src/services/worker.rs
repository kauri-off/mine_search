//! WorkerControl gRPC service: the worker-facing control plane. Workers dial in
//! and open the `Session` stream; the backend persists their results and pushes
//! commands. `FetchUpdateTargets` serves the per-cycle re-probe list.

use std::{pin::Pin, sync::Arc};

use futures::Stream;
use proto::worker::{
    Ack, FetchUpdateTargetsRequest, ScanResult, ServerCommand, UpdateTarget, WorkerMessage,
    scan_result, server_command, worker_control_server::WorkerControl, worker_message,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};

use crate::{auth, persistence, server_filters::ServerFilters, state::AppState};

/// Capacity of the per-session result queue feeding the writer task. Sized to
/// absorb short DB hiccups; on overflow results are dropped and replayed from
/// the worker's outbox.
const WRITER_QUEUE: usize = 1024;

pub struct WorkerService {
    pub state: Arc<AppState>,
}

/// Rows per keyset page when streaming update targets. Big enough to amortise
/// round-trips, small enough that each page is a bounded gRPC message.
const UPDATE_TARGET_BATCH: i64 = 1000;

#[tonic::async_trait]
impl WorkerControl for WorkerService {
    type SessionStream = Pin<Box<dyn Stream<Item = Result<ServerCommand, Status>> + Send>>;
    type FetchUpdateTargetsStream =
        Pin<Box<dyn Stream<Item = Result<UpdateTarget, Status>> + Send>>;

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

            // Persistence runs on a dedicated writer task fed by this channel, so
            // a slow/unreachable DB never blocks the read loop below (which also
            // processes heartbeats — a stall there would make the worker look
            // offline). Results that don't fit are dropped; the worker's outbox
            // replays anything left un-acked.
            let (result_tx, result_rx) = mpsc::channel::<ScanResult>(WRITER_QUEUE);
            let writer = tokio::spawn(writer_task(state.clone(), cmd_tx.clone(), result_rx));

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
                            if let Err(mpsc::error::TrySendError::Full(_)) =
                                result_tx.try_send(result)
                            {
                                tracing::warn!(
                                    "db writer queue full; dropping scan result (worker will replay)"
                                );
                            }
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

            // Close the queue so the writer task drains and exits.
            drop(result_tx);
            let _ = writer.await;

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
    ) -> Result<Response<Self::FetchUpdateTargetsStream>, Status> {
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

        // Which existing servers this cycle re-probes. Absent filter = re-probe all.
        let filters: ServerFilters = req.filter.as_ref().map(ServerFilters::from).unwrap_or_default();

        // Page through the servers table and stream one target per row. Each
        // batch re-acquires (and releases) a pooled connection, so a slow worker
        // draining the stream never pins a connection for the whole cycle.
        let (tx, rx) = mpsc::channel::<Result<UpdateTarget, Status>>(256);
        let state = self.state.clone();
        tokio::spawn(async move {
            let mut after_id: Option<i32> = None;
            loop {
                let rows = match persistence::fetch_update_targets_batch(
                    &state.db,
                    &filters,
                    after_id,
                    UPDATE_TARGET_BATCH,
                )
                .await
                {
                    Ok(rows) => rows,
                    Err(e) => {
                        let _ = tx
                            .send(Err(Status::internal(format!("db error: {e}"))))
                            .await;
                        return;
                    }
                };

                let got = rows.len();
                for row in rows {
                    after_id = Some(row.id);
                    let target = UpdateTarget {
                        ip: row.ip,
                        port: row.port,
                        with_connection,
                    };
                    if tx.send(Ok(target)).await.is_err() {
                        return; // worker dropped the stream
                    }
                }

                if (got as i64) < UPDATE_TARGET_BATCH {
                    break;
                }
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }
}

/// Drains the per-session result queue, persisting each result and acking it
/// back to the worker once durable. Persist failures are logged and left
/// un-acked, so the worker replays them later. Exits when the queue is closed
/// (the session ended).
async fn writer_task(
    state: Arc<AppState>,
    cmd_tx: mpsc::Sender<Result<ServerCommand, Status>>,
    mut result_rx: mpsc::Receiver<ScanResult>,
) {
    while let Some(result) = result_rx.recv().await {
        let result_id = result.result_id.clone();
        let outcome = match result.outcome {
            Some(scan_result::Outcome::Discovered(s)) => {
                persistence::persist_discovered(&state.db, s, &result_id).await
            }
            Some(scan_result::Outcome::Updated(s)) => {
                persistence::persist_updated(&state.db, s, &result_id).await
            }
            Some(scan_result::Outcome::Offline(o)) => {
                persistence::persist_offline(&state.db, &o.ip, &result_id).await
            }
            None => Ok(None),
        };
        match outcome {
            Ok(maybe_id) => {
                if let Some(server_id) = maybe_id {
                    state.events.notify(server_id);
                }
                // Ack even on `None` (replay / vanished server): the result is
                // durably accounted for, so the worker should drop it.
                if !result_id.is_empty() {
                    let _ = cmd_tx
                        .send(Ok(ServerCommand {
                            cmd: Some(server_command::Cmd::Ack(Ack { result_id })),
                        }))
                        .await;
                }
            }
            Err(e) => tracing::error!("failed to persist scan result: {e}"),
        }
    }
}
