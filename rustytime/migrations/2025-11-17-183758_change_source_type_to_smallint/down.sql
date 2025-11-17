-- Disable compression
ALTER TABLE heartbeats SET (timescaledb.compress = false);

-- Revert source_type from integer back to string
ALTER TABLE heartbeats ALTER COLUMN source_type TYPE VARCHAR USING 
    CASE 
        WHEN source_type = 0 THEN 'direct_entry'
        ELSE ''
    END;

-- Re-enable compression
ALTER TABLE heartbeats SET (timescaledb.compress = true);
