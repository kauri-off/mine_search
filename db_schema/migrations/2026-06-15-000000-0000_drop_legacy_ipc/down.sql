CREATE TABLE scan_targets (
    id SERIAL PRIMARY KEY,
    ip TEXT NOT NULL,
    port INTEGER NOT NULL,
    quick BOOLEAN NOT NULL
);
CREATE INDEX idx_scan_targets_ip_port ON scan_targets (ip, port);

CREATE TABLE ping_requests (
    id SERIAL PRIMARY KEY,
    server_id INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    with_connection BOOLEAN NOT NULL DEFAULT FALSE
);
