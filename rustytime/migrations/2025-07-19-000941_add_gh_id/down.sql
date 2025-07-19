-- Remove 'github_id' column to users table
ALTER TABLE users
    DROP COLUMN IF EXISTS github_id;