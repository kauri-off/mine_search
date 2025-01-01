-- This file should undo anything in `up.sql`

ALTER TABLE players
ALTER COLUMN last_seen DROP DEFAULT;
