-- Change type to int for github_id in users table
ALTER TABLE users
    ALTER COLUMN github_id TYPE INTEGER USING github_id::INTEGER;