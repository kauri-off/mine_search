-- Your SQL goes here

CREATE TABLE server_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    ip TEXT NOT NULL UNIQUE, -- Переименованное поле с ограничением UNIQUE
    online INTEGER NOT NULL,
    max INTEGER NOT NULL,
    version_name TEXT NOT NULL,
    protocol INTEGER NOT NULL,
    license BOOLEAN NOT NULL,
    white_list BOOLEAN
);

PRAGMA foreign_keys = OFF;

INSERT INTO server_new (id, ip, online, max, version_name, protocol, license, white_list)
SELECT id, addr, online, max, version_name, protocol, license, white_list
FROM server;

DROP TABLE server;

ALTER TABLE server_new RENAME TO server;

PRAGMA foreign_keys = ON;
