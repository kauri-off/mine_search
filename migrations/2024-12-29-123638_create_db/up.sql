-- Your SQL goes here

-- Таблица для данных о статусе
CREATE TABLE server (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    addr TEXT NOT NULL,
    online INTEGER NOT NULL,
    max INTEGER NOT NULL,
    version_name TEXT NOT NULL,
    protocol INTEGER NOT NULL,
    license BOOLEAN NOT NULL,
    white_list BOOLEAN
);

-- Таблица для игроков
CREATE TABLE players (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    uuid TEXT NOT NULL,
    name TEXT NOT NULL,
    last_seen TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    server_id INTEGER NOT NULL,
    FOREIGN KEY (server_id) REFERENCES server (id)
);
