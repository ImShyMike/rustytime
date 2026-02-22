CREATE INDEX IF NOT EXISTS idx_heartbeats_project ON heartbeats(user_id, project, time DESC) WHERE project IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_heartbeats_language ON heartbeats(user_id, language, time DESC) WHERE language IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_heartbeats_editor ON heartbeats(user_id, editor, time DESC) WHERE editor IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_heartbeats_operating_system ON heartbeats(user_id, operating_system, time DESC) WHERE operating_system IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_heartbeats_project_user ON heartbeats(time, project, user_id) WHERE project IS NOT NULL;

CREATE TABLE heartbeats_new (
    id                 BIGSERIAL,
    time               TIMESTAMPTZ      NOT NULL DEFAULT now(),
    created_at         TIMESTAMPTZ      NOT NULL DEFAULT now(),
    user_id            INTEGER          NOT NULL,
    entity             TEXT             NOT NULL,
    type               TEXT             NOT NULL,
    ip_address         INET             NOT NULL,
    project            TEXT,
    branch             TEXT,
    language           TEXT,
    category           TEXT,
    is_write           BOOLEAN          DEFAULT FALSE,
    editor             TEXT,
    operating_system   TEXT,
    machine            TEXT,
    user_agent         TEXT             NOT NULL DEFAULT '',
    lines              INTEGER,
    project_root_count INTEGER,
    dependencies       TEXT[],
    line_additions     INTEGER,
    line_deletions     INTEGER,
    lineno             INTEGER,
    cursorpos          INTEGER,
    source_type        SMALLINT,
    project_id         INTEGER,
    PRIMARY KEY (user_id, time)
);

SELECT create_hypertable(
    'heartbeats_new',
    'time',
    chunk_time_interval => INTERVAL '1 day',
    if_not_exists => TRUE
);

ALTER TABLE heartbeats_new SET (
    timescaledb.compress           = true,
    timescaledb.compress_segmentby = 'user_id',
    timescaledb.compress_orderby   = 'time DESC'
);

INSERT INTO heartbeats_new (id, time, created_at, user_id, entity, type, ip_address, project, branch,
    language, category, is_write, editor, operating_system, machine, user_agent, lines,
    project_root_count, dependencies, line_additions, line_deletions, lineno, cursorpos,
    source_type, project_id)
SELECT id, time, created_at, user_id, entity, type, ip_address, project, branch,
    language, category, is_write, editor, operating_system, machine, user_agent, lines,
    project_root_count, dependencies, line_additions, line_deletions, lineno, cursorpos,
    source_type, project_id
FROM heartbeats;

SELECT setval(
    pg_get_serial_sequence('heartbeats_new', 'id'),
    (SELECT MAX(id) FROM heartbeats_new)
);

ALTER TABLE heartbeats RENAME TO heartbeats_old;
ALTER TABLE heartbeats_new RENAME TO heartbeats;

DROP INDEX IF EXISTS idx_heartbeats_user_time;
CREATE INDEX idx_heartbeats_user_time ON heartbeats (user_id, time DESC);

SELECT add_compression_policy('heartbeats', INTERVAL '7 days');

DROP TABLE heartbeats_old CASCADE;