-- 0. Decompress all compressed chunks
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

-- 1. Disable compression temporarily
ALTER TABLE heartbeats SET (timescaledb.compress = false);

-- 2. Change column type from INTEGER -> BIGINT
ALTER TABLE heartbeats ALTER COLUMN id TYPE BIGINT;

-- 3. Ensure the sequence exists and is attached
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_class WHERE relname = 'heartbeats_id_seq') THEN
        CREATE SEQUENCE heartbeats_id_seq OWNED BY heartbeats.id;
    END IF;
END
$$;

ALTER TABLE heartbeats ALTER COLUMN id SET DEFAULT nextval('heartbeats_id_seq');

-- 4. Re-enable compression
ALTER TABLE heartbeats SET (timescaledb.compress = true);
