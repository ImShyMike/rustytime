-- Drop the functions
DROP FUNCTION IF EXISTS calculate_user_duration(INT, TIMESTAMPTZ, TIMESTAMPTZ, TEXT, TEXT, TEXT, TEXT, INT);
DROP FUNCTION IF EXISTS calculate_all_user_durations(TIMESTAMPTZ, TIMESTAMPTZ, INT);
DROP FUNCTION IF EXISTS calculate_field_stats(INT, TEXT, INT, INT);
DROP FUNCTION IF EXISTS calculate_project_stats_with_aliases(INT, INT, INT);
DROP FUNCTION IF EXISTS calculate_project_time_with_aliases(INT, INT, INT);
