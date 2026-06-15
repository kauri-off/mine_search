-- The backend↔worker IPC used to flow through these polling tables. With the
-- gRPC control plane (workers stream results and receive commands directly),
-- they are no longer used and are dropped.
DROP TABLE IF EXISTS ping_requests;
DROP TABLE IF EXISTS scan_targets;
