-- Remove 'email' from users table
ALTER TABLE users
    DROP COLUMN IF EXISTS email;