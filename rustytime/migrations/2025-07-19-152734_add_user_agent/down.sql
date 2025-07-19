-- Remove the 'user_agent' column from the heartbeat table
ALTER TABLE heartbeats
    DROP COLUMN user_agent;