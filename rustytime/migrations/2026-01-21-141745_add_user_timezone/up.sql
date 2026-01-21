ALTER TABLE users ADD COLUMN timezone VARCHAR(50) NOT NULL DEFAULT 'UTC';

CREATE INDEX idx_users_timezone ON users(timezone);
