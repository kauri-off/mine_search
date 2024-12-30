-- This file should undo anything in `up.sql`

CREATE TABLE server_old (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    addr TEXT NOT NULL, -- Поле addr возвращается
    online INTEGER NOT NULL,
    max INTEGER NOT NULL,
    version_name TEXT NOT NULL,
    protocol INTEGER NOT NULL,
    license BOOLEAN NOT NULL,
    white_list BOOLEAN
);

PRAGMA foreign_keys = OFF;

INSERT INTO server_old (id, addr, online, max, version_name, protocol, license, white_list)
SELECT id, ip, online, max, version_name, protocol, license, white_list
FROM server;

DROP TABLE server;

ALTER TABLE server_old RENAME TO server;

PRAGMA foreign_keys = ON;