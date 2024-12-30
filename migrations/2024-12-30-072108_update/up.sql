-- Your SQL goes here

ALTER TABLE server RENAME COLUMN addr TO ip;

CREATE UNIQUE INDEX IF NOT EXISTS ux_ip ON server(ip);