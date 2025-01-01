-- Your SQL goes here
CREATE TABLE servers (
    id SERIAL PRIMARY KEY,
    ip TEXT NOT NULL UNIQUE,
    online INTEGER NOT NULL,
    max INTEGER NOT NULL,
    version_name TEXT NOT NULL,
    protocol INTEGER NOT NULL,
    license BOOLEAN NOT NULL,
    white_list BOOLEAN,
    description TEXT
);

CREATE TABLE players (
    id SERIAL PRIMARY KEY,
    uuid TEXT NOT NULL,
    name TEXT NOT NULL,
    last_seen TIMESTAMP NOT NULL,
    server_id INTEGER,
    FOREIGN KEY (server_id) REFERENCES servers(id),
    UNIQUE (name, server_id)
);
