-- Your SQL goes here
ALTER TABLE servers ADD COLUMN checked BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE servers ADD COLUMN auth_me BOOLEAN DEFAULT NULL;
ALTER TABLE servers ADD COLUMN crashed BOOLEAN NOT NULL DEFAULT false;
