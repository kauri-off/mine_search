-- servers: lookup by IP (fetch_server_info)
CREATE INDEX idx_servers_ip ON servers (ip);

-- servers: boolean filter columns used in fetch_server_list
CREATE INDEX idx_servers_license ON servers (license);
CREATE INDEX idx_servers_checked ON servers (checked);
CREATE INDEX idx_servers_spoofable ON servers (spoofable);
CREATE INDEX idx_servers_crashed ON servers (crashed);
CREATE INDEX idx_servers_was_online ON servers (was_online);
CREATE INDEX idx_servers_unique_players ON servers (unique_players);

-- data: foreign key + ordering (fetch_server_data, fetch_server_info, fetch_server_list)
CREATE INDEX idx_data_server_id_id ON data (server_id, id DESC);

-- ips: lookup by ip+port to avoid duplicates
CREATE INDEX idx_ips_ip_port ON ips (ip, port);