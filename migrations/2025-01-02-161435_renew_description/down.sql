-- This file should undo anything in `up.sql`

ALTER TABLE servers DROP COLUMN description;

ALTER TABLE servers ADD COLUMN description TEXT;