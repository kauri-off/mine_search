-- This file should undo anything in `up.sql`
ALTER TABLE servers DROP COLUMN checked;
ALTER TABLE servers DROP COLUMN auth_me;
ALTER TABLE servers DROP COLUMN crashed;
