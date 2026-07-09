DROP INDEX IF EXISTS idx_servers_motd_trgm;
DROP INDEX IF EXISTS idx_servers_version_name_trgm;
DROP INDEX IF EXISTS idx_servers_ip_trgm;
ALTER TABLE servers DROP COLUMN IF EXISTS motd;
-- Leave the pg_trgm extension in place: other objects may depend on it and
-- dropping an extension another migration/table relies on would fail.
