UPDATE servers
SET crashed = FALSE
WHERE crashed IS NULL;

ALTER TABLE servers
ALTER COLUMN crashed SET NOT NULL;
