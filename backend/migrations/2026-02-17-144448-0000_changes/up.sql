ALTER TABLE servers DROP COLUMN white_list;
ALTER TABLE servers ADD COLUMN disconnect_reason JSONB;
ALTER TABLE servers RENAME COLUMN auth_me TO spoofable;