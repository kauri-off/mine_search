-- Postgres cannot drop a value from an enum, so rebuild the type without 'broken'.
ALTER TABLE servers ALTER COLUMN join_status DROP DEFAULT;
DROP INDEX idx_servers_join_status;

ALTER TYPE join_status RENAME TO join_status_old;
CREATE TYPE join_status AS ENUM ('undetermined', 'spoofable', 'whitelist', 'password', 'modded');

UPDATE servers SET join_status = 'undetermined' WHERE join_status = 'broken';
ALTER TABLE servers
  ALTER COLUMN join_status TYPE join_status USING join_status::text::join_status;

ALTER TABLE servers ALTER COLUMN join_status SET DEFAULT 'undetermined';
DROP TYPE join_status_old;

CREATE INDEX idx_servers_join_status ON servers (join_status);
