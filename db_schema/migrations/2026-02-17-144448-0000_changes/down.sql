ALTER TABLE servers ADD COLUMN white_list BOOLEAN;
ALTER TABLE servers RENAME COLUMN spoofable TO auth_me;
ALTER TABLE servers DROP COLUMN disconnect_reason;