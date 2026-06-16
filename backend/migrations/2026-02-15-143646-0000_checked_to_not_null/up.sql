UPDATE servers
SET checked = FALSE
WHERE checked IS NULL;

ALTER TABLE servers
ALTER COLUMN checked SET NOT NULL;
