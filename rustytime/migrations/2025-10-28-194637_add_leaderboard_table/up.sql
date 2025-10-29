-- Create leaderboard table
CREATE TABLE leaderboards (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    period_type VARCHAR(20) NOT NULL, -- 'daily', 'weekly', 'all_time'
    period_date DATE NOT NULL, -- for daily/weekly: the date of the period; for all_time: '1970-01-01'
    total_seconds BIGINT NOT NULL DEFAULT 0,
    rank INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_leaderboards_period ON leaderboards(period_type, period_date);
CREATE INDEX idx_leaderboards_user ON leaderboards(user_id);
CREATE UNIQUE INDEX idx_leaderboards_unique ON leaderboards(user_id, period_type, period_date);

-- Auto manage updated_at column
SELECT diesel_manage_updated_at ('leaderboards');
