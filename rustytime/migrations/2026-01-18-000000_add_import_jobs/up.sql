-- Create import_jobs table for tracking background import jobs
CREATE TABLE import_jobs (
    id BIGSERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL DEFAULT 'running', -- 'running', 'completed', 'failed'
    imported_count BIGINT,
    processed_count BIGINT,
    request_count INTEGER,
    start_date TEXT,
    time_taken DOUBLE PRECISION,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_import_jobs_user ON import_jobs(user_id);
CREATE INDEX idx_import_jobs_status ON import_jobs(status);
CREATE INDEX idx_import_jobs_user_created ON import_jobs(user_id, created_at DESC);

-- Auto manage updated_at column
SELECT diesel_manage_updated_at('import_jobs');
