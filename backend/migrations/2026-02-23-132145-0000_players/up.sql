ALTER TABLE data DROP COLUMN players;
ALTER TABLE servers DROP COLUMN unique_players;

CREATE TYPE player_status AS ENUM ('none', 'regular', 'admin');

CREATE TABLE players (
    id          SERIAL PRIMARY KEY,
    server_id   INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    name        VARCHAR NOT NULL,
    status      player_status NOT NULL DEFAULT 'none',
    CONSTRAINT players_server_id_name_unique UNIQUE (server_id, name)
);