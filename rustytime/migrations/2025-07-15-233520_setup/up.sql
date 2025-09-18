-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Enable pgcrypto extension
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Create users table
CREATE TABLE users (
  id          SERIAL        PRIMARY KEY,
  github_id   BIGINT        NOT NULL UNIQUE,
  name        VARCHAR(100)  NOT NULL,
  avatar_url  VARCHAR(200)  NOT NULL,
  created_at  TIMESTAMPTZ   NOT NULL DEFAULT now(),
  api_key     UUID          NOT NULL UNIQUE DEFAULT gen_random_uuid(),
  is_admin    BOOLEAN       NOT NULL DEFAULT FALSE,
  is_banned   BOOLEAN       NOT NULL DEFAULT FALSE,
  updated_at  TIMESTAMPTZ   NOT NULL DEFAULT now()
);

-- Auto manage updated_at column
SELECT diesel_manage_updated_at('users');

-- Create sessions table
CREATE TABLE sessions (
    id UUID             PRIMARY KEY  DEFAULT gen_random_uuid(),
    user_id             INTEGER      NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    github_access_token TEXT         NOT NULL,
    github_user_id      BIGINT       NOT NULL,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    expires_at          TIMESTAMPTZ  NOT NULL DEFAULT (now() + interval '7 days')
);

-- Auto manage updated_at column
SELECT diesel_manage_updated_at('sessions');

-- Create index for efficient lookups
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_github_user_id ON sessions(github_user_id);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);

-- Create heartbeats table
CREATE TABLE IF NOT EXISTS heartbeats (
  id                SERIAL NOT NULL,
  time              BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM now()),
  created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
  user_id           INTEGER NOT NULL REFERENCES users(id),
  entity            TEXT NOT NULL,
  type              TEXT NOT NULL,
  ip_address        INET NOT NULL,
  project           TEXT,
  branch            TEXT,
  language          TEXT,
  category          TEXT,
  is_write          BOOLEAN DEFAULT FALSE,
  editor            TEXT,
  operating_system  TEXT,
  machine           TEXT,
  user_agent        TEXT NOT NULL DEFAULT '',
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
