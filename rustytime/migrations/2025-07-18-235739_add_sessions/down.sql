-- Drop the trigger and function
DROP TRIGGER IF EXISTS update_sessions_updated_at ON sessions;
DROP FUNCTION IF EXISTS update_updated_at_column();

-- Drop the sessions table
DROP TABLE IF EXISTS sessions;
