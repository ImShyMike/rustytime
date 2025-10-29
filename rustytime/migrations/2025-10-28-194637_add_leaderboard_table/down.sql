-- Disable automatic updated_at management for leaderboards
SELECT diesel_manage_updated_at('leaderboards', 'disable');

-- Drop indexes
DROP INDEX IF EXISTS idx_leaderboards_unique;
DROP INDEX IF EXISTS idx_leaderboards_user;
DROP INDEX IF EXISTS idx_leaderboards_period;

-- Drop the table
DROP TABLE IF EXISTS leaderboards;
