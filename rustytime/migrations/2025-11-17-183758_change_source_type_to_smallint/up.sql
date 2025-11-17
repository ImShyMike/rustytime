-- Decompress all compressed chunks
DO $$
DECLARE
    r RECORD;
BEGIN
    FOR r IN
        SELECT
            c.schema_name AS chunk_schema,
            c.table_name  AS chunk_name
        FROM _timescaledb_catalog.hypertable h
        JOIN _timescaledb_catalog.chunk c ON c.hypertable_id = h.id
        WHERE h.table_name = 'heartbeats'
          AND c.compressed_chunk_id IS NOT NULL
    LOOP
        EXECUTE format('SELECT decompress_chunk(''%I.%I'')', r.chunk_schema, r.chunk_name);
    END LOOP;
END
$$;

-- Disable compression temporarily
ALTER TABLE heartbeats SET (timescaledb.compress = false);

-- Change source_type from string to smallint
ALTER TABLE heartbeats ALTER COLUMN source_type TYPE SMALLINT USING 
    CASE 
        WHEN source_type = 'direct_entry' THEN 0 
        ELSE 1 
    END;

-- Re-enable compression
ALTER TABLE heartbeats SET (timescaledb.compress = true);
