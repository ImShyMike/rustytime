-- Add 'user_agent' column to heartbeat table
ALTER TABLE heartbeats
    ADD COLUMN IF NOT EXISTS user_agent VARCHAR(255) NOT NULL DEFAULT '';