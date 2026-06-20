-- Idempotency ledger for worker scan results. The worker assigns a stable
-- `result_id` (UUID) to every ScanResult and keeps it in a durable outbox until
-- the backend acks it. Because delivery is at-least-once (the worker replays
-- unacked results on reconnect / on a periodic sweep), the backend records each
-- processed id here and skips re-applying a result it has already persisted —
-- otherwise replays would create duplicate, append-only player_count_snapshots.
CREATE TABLE processed_results (
    result_id TEXT PRIMARY KEY,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Supports the periodic prune of rows older than the worker's replay horizon.
CREATE INDEX idx_processed_results_processed_at ON processed_results (processed_at);
