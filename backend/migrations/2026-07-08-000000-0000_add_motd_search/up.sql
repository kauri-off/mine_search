-- Free-text server search used to scan the whole table in-process, deserializing
-- each row's `description` JSONB to flatten its MOTD. Make search indexable:
--   1. pg_trgm for substring (`ILIKE %needle%`) matching backed by a GIN index.
--   2. a plaintext `motd` column populated at write time (the JSONB chat tree
--      can't be filtered directly), so MOTD is queryable like ip/version_name.
--
-- Existing rows get motd = '' and become MOTD-searchable the next time they are
-- re-probed (an update cycle rewrites the column). ip/version_name search works
-- immediately via their trigram indexes.
CREATE EXTENSION IF NOT EXISTS pg_trgm;

ALTER TABLE servers ADD COLUMN motd text NOT NULL DEFAULT '';

CREATE INDEX idx_servers_ip_trgm ON servers USING gin (ip gin_trgm_ops);
CREATE INDEX idx_servers_version_name_trgm ON servers USING gin (version_name gin_trgm_ops);
CREATE INDEX idx_servers_motd_trgm ON servers USING gin (motd gin_trgm_ops);
