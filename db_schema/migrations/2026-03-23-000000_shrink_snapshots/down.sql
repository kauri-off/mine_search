ALTER TABLE player_count_snapshots DROP CONSTRAINT player_count_snapshots_pkey;

ALTER TABLE player_count_snapshots ADD COLUMN id bigserial;
UPDATE player_count_snapshots SET id = nextval(pg_get_serial_sequence('player_count_snapshots', 'id'));
ALTER TABLE player_count_snapshots ADD PRIMARY KEY (id);

ALTER TABLE player_count_snapshots
    ALTER COLUMN players_online TYPE int4,
    ALTER COLUMN players_max    TYPE int4;
