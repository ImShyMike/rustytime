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
  api_key     UUID          NOT NULL UNIQUE DEFAULT gen_random_uuid(),
  is_admin    BOOLEAN       NOT NULL DEFAULT FALSE,
  is_banned   BOOLEAN       NOT NULL DEFAULT FALSE,
  created_at  TIMESTAMPTZ   NOT NULL DEFAULT now(),
  updated_at  TIMESTAMPTZ   NOT NULL DEFAULT now()
);

-- Auto manage updated_at column
SELECT diesel_manage_updated_at('users');

-- Create sessions table
CREATE TABLE sessions (
    id                  UUID        PRIMARY KEY  DEFAULT gen_random_uuid(),
    user_id             INTEGER     NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    github_user_id      BIGINT      NOT NULL,
    github_access_token TEXT        NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at          TIMESTAMPTZ NOT NULL DEFAULT (now() + interval '7 days'),
    CONSTRAINT sessions_expires_at_future CHECK (expires_at > created_at)
);

-- Create indexes for session lookups
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_github_user_id ON sessions(github_user_id);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);

-- Auto manage updated_at column
SELECT diesel_manage_updated_at('sessions');

-- Create heartbeats table
CREATE TABLE IF NOT EXISTS heartbeats (
  id                SERIAL NOT NULL,
  time              TIMESTAMPTZ NOT NULL DEFAULT now(),
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
  source_type       TEXT,
  PRIMARY KEY (user_id, time)
);

-- Transform to hypertable
SELECT create_hypertable(
  'heartbeats',
  'time',
  chunk_time_interval => INTERVAL '1 day',
  if_not_exists      => TRUE
);

-- Enable compression for performance
ALTER TABLE heartbeats
  SET (
    timescaledb.compress                = true,
    timescaledb.compress_segmentby      = 'user_id',
    timescaledb.compress_orderby        = 'time DESC'
  );

-- Add a compression policy
SELECT add_compression_policy('heartbeats', INTERVAL '7 days');

-- Primary time-series index
CREATE INDEX idx_heartbeats_user_time ON heartbeats (user_id, time DESC);

-- For common queries
CREATE INDEX idx_heartbeats_project ON heartbeats(user_id, project, time DESC) WHERE project IS NOT NULL;
CREATE INDEX idx_heartbeats_language ON heartbeats(user_id, language, time DESC) WHERE language IS NOT NULL;
CREATE INDEX idx_heartbeats_editor ON heartbeats(user_id, editor, time DESC) WHERE editor IS NOT NULL;
CREATE INDEX idx_heartbeats_operating_system ON heartbeats(user_id, operating_system, time DESC) WHERE operating_system IS NOT NULL;
CREATE INDEX idx_heartbeats_project_user ON heartbeats(time, project, user_id) WHERE project IS NOT NULL;

-- Remove expired sessions older than 30 days
CREATE OR REPLACE FUNCTION cleanup_expired_sessions() RETURNS void AS $$
BEGIN
  DELETE FROM sessions WHERE expires_at < now() - INTERVAL '30 days';
END;
$$ LANGUAGE plpgsql;

-- Schedule session cleanup to run daily
SELECT add_job('cleanup_expired_sessions', '1 day');

-- For global recent activity (last 24h, last hour, etc...)
CREATE INDEX idx_heartbeats_time_desc ON heartbeats(time DESC);

-- For daily activity analysis (simple time index)
CREATE INDEX idx_heartbeats_time_user ON heartbeats(time, user_id);

-- For LAG queries and window functions
CREATE INDEX idx_heartbeats_window_operations ON heartbeats(user_id, time) 
INCLUDE (project, language, editor, operating_system, entity, type);