-- Rename tables
ALTER TABLE data RENAME TO player_count_snapshots;
ALTER TABLE server_ping RENAME TO ping_requests;
ALTER TABLE ips RENAME TO scan_targets;

-- Rename columns on servers
ALTER TABLE servers RENAME COLUMN license    TO is_online_mode;
ALTER TABLE servers RENAME COLUMN checked    TO is_checked;
ALTER TABLE servers RENAME COLUMN spoofable  TO is_spoofable;
ALTER TABLE servers RENAME COLUMN crashed    TO is_crashed;
ALTER TABLE servers RENAME COLUMN was_online TO is_online;
ALTER TABLE servers RENAME COLUMN created    TO created_at;
ALTER TABLE servers RENAME COLUMN updated    TO updated_at;

-- Rename columns on player_count_snapshots (was data)
ALTER TABLE player_count_snapshots RENAME COLUMN online     TO players_online;
ALTER TABLE player_count_snapshots RENAME COLUMN max        TO players_max;
ALTER TABLE player_count_snapshots RENAME COLUMN timestamp  TO recorded_at;

-- Rename indexes
ALTER INDEX idx_servers_license    RENAME TO idx_servers_is_online_mode;
ALTER INDEX idx_servers_checked    RENAME TO idx_servers_is_checked;
ALTER INDEX idx_servers_spoofable  RENAME TO idx_servers_is_spoofable;
ALTER INDEX idx_servers_crashed    RENAME TO idx_servers_is_crashed;
ALTER INDEX idx_servers_was_online RENAME TO idx_servers_is_online;
ALTER INDEX idx_data_server_id_id  RENAME TO idx_player_count_snapshots_server_id;
ALTER INDEX idx_ips_ip_port        RENAME TO idx_scan_targets_ip_port;
