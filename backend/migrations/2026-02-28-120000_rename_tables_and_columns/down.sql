-- Restore indexes
ALTER INDEX idx_scan_targets_ip_port             RENAME TO idx_ips_ip_port;
ALTER INDEX idx_player_count_snapshots_server_id RENAME TO idx_data_server_id_id;
ALTER INDEX idx_servers_is_online                RENAME TO idx_servers_was_online;
ALTER INDEX idx_servers_is_crashed               RENAME TO idx_servers_crashed;
ALTER INDEX idx_servers_is_spoofable             RENAME TO idx_servers_spoofable;
ALTER INDEX idx_servers_is_checked               RENAME TO idx_servers_checked;
ALTER INDEX idx_servers_is_online_mode           RENAME TO idx_servers_license;

-- Restore columns on player_count_snapshots
ALTER TABLE player_count_snapshots RENAME COLUMN recorded_at    TO timestamp;
ALTER TABLE player_count_snapshots RENAME COLUMN players_max    TO max;
ALTER TABLE player_count_snapshots RENAME COLUMN players_online TO online;

-- Restore columns on servers
ALTER TABLE servers RENAME COLUMN updated_at     TO updated;
ALTER TABLE servers RENAME COLUMN created_at     TO created;
ALTER TABLE servers RENAME COLUMN is_online      TO was_online;
ALTER TABLE servers RENAME COLUMN is_crashed     TO crashed;
ALTER TABLE servers RENAME COLUMN is_spoofable   TO spoofable;
ALTER TABLE servers RENAME COLUMN is_checked     TO checked;
ALTER TABLE servers RENAME COLUMN is_online_mode TO license;

-- Restore tables
ALTER TABLE scan_targets           RENAME TO ips;
ALTER TABLE ping_requests          RENAME TO server_ping;
ALTER TABLE player_count_snapshots RENAME TO data;
