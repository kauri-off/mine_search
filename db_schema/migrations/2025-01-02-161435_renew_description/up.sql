-- Your SQL goes here

ALTER TABLE servers DROP COLUMN description;

ALTER TABLE servers ADD COLUMN description JSONB DEFAULT '{}' NOT NULL;
