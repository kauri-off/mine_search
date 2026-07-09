CREATE TYPE join_status AS ENUM ('undetermined', 'spoofable', 'whitelist', 'password', 'modded');

ALTER TABLE servers ADD COLUMN join_status join_status NOT NULL DEFAULT 'undetermined';

UPDATE servers SET join_status = 'spoofable' WHERE is_spoofable IS TRUE;

DROP INDEX idx_servers_is_spoofable;
ALTER TABLE servers DROP COLUMN is_spoofable;

CREATE INDEX idx_servers_join_status ON servers (join_status);
