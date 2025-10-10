-- Disable compression
ALTER TABLE heartbeats SET (timescaledb.compress = false);

-- Drop default
ALTER TABLE heartbeats ALTER COLUMN id DROP DEFAULT;

-- Revert to INTEGER
ALTER TABLE heartbeats ALTER COLUMN id TYPE INTEGER;

-- Drop sequence
DROP SEQUENCE IF EXISTS heartbeats_id_seq;

-- Re-enable compression
ALTER TABLE heartbeats SET (timescaledb.compress = true);
