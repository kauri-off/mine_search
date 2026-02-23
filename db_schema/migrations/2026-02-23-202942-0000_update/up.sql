CREATE TABLE server_ping (
    id          SERIAL PRIMARY KEY,
    server_id   INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE
);