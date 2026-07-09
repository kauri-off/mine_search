DROP INDEX idx_servers_join_status;

ALTER TABLE servers ADD COLUMN is_spoofable BOOLEAN;
UPDATE servers SET is_spoofable = TRUE WHERE join_status = 'spoofable';

ALTER TABLE servers DROP COLUMN join_status;
DROP TYPE join_status;

CREATE INDEX idx_servers_is_spoofable ON servers (is_spoofable);
