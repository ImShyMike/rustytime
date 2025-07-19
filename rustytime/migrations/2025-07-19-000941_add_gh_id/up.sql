-- Add 'github_id' column to users table
ALTER TABLE users
    ADD COLUMN IF NOT EXISTS github_id TEXT UNIQUE NOT NULL;