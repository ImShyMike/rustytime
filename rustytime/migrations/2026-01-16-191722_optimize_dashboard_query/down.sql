-- Drop the optimized function
DROP FUNCTION IF EXISTS calculate_dashboard_stats_by_range(INT, TIMESTAMPTZ, INT, INT);

-- Drop the covering index
DROP INDEX IF EXISTS idx_heartbeats_user_time_cover;
