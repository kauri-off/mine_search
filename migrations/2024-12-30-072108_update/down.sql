-- This file should undo anything in `up.sql`

ALTER TABLE server RENAME COLUMN ip TO addr;