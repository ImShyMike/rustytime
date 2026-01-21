ALTER TABLE users DROP COLUMN timezone;

DROP INDEX IF EXISTS idx_users_timezone;
