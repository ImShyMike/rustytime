-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Enable pgcrypto extension
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Create users table
CREATE TABLE users (
  id          SERIAL       PRIMARY KEY,
  email       VARCHAR(254) UNIQUE NOT NULL,
  name        VARCHAR(100),
  avatar_url  VARCHAR(200),
  created_at  TIMESTAMPTZ  NOT NULL DEFAULT now(),
  api_key     UUID         NOT NULL UNIQUE DEFAULT gen_random_uuid()
);

-- Create heartbeats table
CREATE TABLE IF NOT EXISTS heartbeats (
  id                SERIAL         NOT NULL,
  created_at        TIMESTAMPTZ    NOT NULL DEFAULT now(),
  user_id           INTEGER        NOT NULL
                       REFERENCES users(id) ON DELETE CASCADE,
  entity            VARCHAR(512)   NOT NULL,
  type              VARCHAR(50)    NOT NULL,
  ip_address        INET           NOT NULL,
  project           VARCHAR(100),
  branch            VARCHAR(100),
  language          VARCHAR(50),
  category          VARCHAR(50),
  is_write          BOOLEAN        DEFAULT FALSE,
  editor            VARCHAR(50),
  operating_system  VARCHAR(100),
  machine           VARCHAR(100),
  user_agent        VARCHAR(255)   NOT NULL DEFAULT '',
  lines             INTEGER,
  project_root_count INTEGER,
  dependencies      TEXT[],
  line_additions    INTEGER,
  line_deletions    INTEGER,
  lineno            INTEGER,
  cursorpos         INTEGER,
  PRIMARY KEY (user_id, created_at)
);

-- Transform to hypertable
SELECT create_hypertable(
  'heartbeats',
  'created_at',
  chunk_time_interval => INTERVAL '1 day',
  if_not_exists      => TRUE
);

-- Enable compression for performance
ALTER TABLE heartbeats
  SET (
    timescaledb.compress            = true,
    timescaledb.compress_segmentby  = 'user_id',
    timescaledb.compress_orderby    = 'created_at'
  );

-- Add a compression policy (compress chunks older than 1 day)
SELECT add_compression_policy('heartbeats', INTERVAL '1 day');

-- Recent activity index
CREATE INDEX ON heartbeats (user_id, created_at DESC);
