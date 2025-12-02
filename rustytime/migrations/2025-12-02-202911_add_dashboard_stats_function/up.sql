-- Aggregate dashboard statistics in a single function call
CREATE OR REPLACE FUNCTION calculate_dashboard_stats(
	p_user_id INT,
	p_timeout_seconds INT,
	p_limit INT DEFAULT 10
) RETURNS TABLE (
	metric_type TEXT,
	name TEXT,
	total_seconds BIGINT,
	total_time BIGINT
) AS $$
DECLARE
	v_total BIGINT;
BEGIN
	SELECT COALESCE(
		calculate_user_duration(
			p_user_id,
			NULL,
			NULL,
			NULL,
			NULL,
			NULL,
			NULL,
			p_timeout_seconds
		),
		0
	)
	INTO v_total;

	RETURN QUERY
	SELECT 'project'::TEXT, stats.name, stats.total_seconds, v_total
	FROM calculate_project_stats_with_aliases(p_user_id, p_timeout_seconds, p_limit) AS stats(name, total_seconds);

	RETURN QUERY
	SELECT 'editor'::TEXT, stats.name, stats.total_seconds, v_total
	FROM calculate_field_stats(p_user_id, 'editor', p_timeout_seconds, p_limit) AS stats(name, total_seconds);

	RETURN QUERY
	SELECT 'operating_system'::TEXT, stats.name, stats.total_seconds, v_total
	FROM calculate_field_stats(p_user_id, 'operating_system', p_timeout_seconds, p_limit) AS stats(name, total_seconds);

	RETURN QUERY
	SELECT 'language'::TEXT, stats.name, stats.total_seconds, v_total
	FROM calculate_field_stats(p_user_id, 'language', p_timeout_seconds, p_limit) AS stats(name, total_seconds);

	RETURN QUERY
	SELECT 'total_time'::TEXT, NULL::TEXT, v_total, v_total;
END;
$$ LANGUAGE plpgsql STABLE;
