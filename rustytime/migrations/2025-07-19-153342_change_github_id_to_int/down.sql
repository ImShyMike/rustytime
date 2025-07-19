-- Revert the 'github_id' column type change in the 'users' table
ALTER TABLE users
    ALTER COLUMN github_id TYPE TEXT USING github_id::TEXT;