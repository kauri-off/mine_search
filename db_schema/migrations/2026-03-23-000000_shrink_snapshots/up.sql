UPDATE player_count_snapshots
SET players_online = GREATEST(LEAST(players_online, 32767), -32768),
    players_max    = GREATEST(LEAST(players_max,    32767), -32768)
WHERE players_online NOT BETWEEN -32768 AND 32767
   OR players_max    NOT BETWEEN -32768 AND 32767;

ALTER TABLE player_count_snapshots
    ALTER COLUMN players_online TYPE int2,
    ALTER COLUMN players_max    TYPE int2;

ALTER TABLE player_count_snapshots DROP COLUMN id CASCADE;

ALTER TABLE player_count_snapshots ADD PRIMARY KEY (server_id, recorded_at);
