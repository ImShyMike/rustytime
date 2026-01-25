-- Create a function to get project stats with aliases
CREATE OR REPLACE FUNCTION calculate_project_stats_with_aliases(
    p_user_id INT,
    p_timeout_seconds INT,
    p_limit INT DEFAULT 10
) RETURNS TABLE (
    name TEXT,
    total_seconds BIGINT
) AS $$
    SELECT
        p.name as name,
        pt.total_seconds
    FROM (
        SELECT
            resolved_project_id,
            COALESCE(SUM(diff), 0)::bigint as total_seconds
        FROM (
            SELECT
                resolved_project_id,
                CASE
                    WHEN prev_time IS NULL THEN 0
                    ELSE LEAST(EXTRACT(EPOCH FROM (time - prev_time)), p_timeout_seconds)
                END as diff
            FROM (
                SELECT
                    h.time,
                    COALESCE(pa.alias_to, h.project_id) as resolved_project_id,
                    LAG(h.time) OVER (
                        PARTITION BY COALESCE(pa.alias_to, h.project_id)
                        ORDER BY h.time
                    ) as prev_time
                FROM heartbeats h
                LEFT JOIN project_aliases pa
                    ON pa.user_id = h.user_id
                    AND pa.project_id = h.project_id
                WHERE h.user_id = p_user_id
                    AND h.project_id IS NOT NULL
            ) resolved_projects
        ) capped_diffs
        GROUP BY resolved_project_id
    ) pt
    JOIN projects p ON p.id = pt.resolved_project_id
    ORDER BY pt.total_seconds DESC
    LIMIT p_limit;
$$ LANGUAGE SQL STABLE;

-- Create a function to get field stats (language, editor, os, etc.)
CREATE OR REPLACE FUNCTION calculate_field_stats(
    p_user_id INT,
    p_field_name TEXT,
    p_timeout_seconds INT,
    p_limit INT DEFAULT 10
) RETURNS TABLE (
    name TEXT,
    total_seconds BIGINT
) AS $$
BEGIN
    RETURN QUERY EXECUTE format(
        'SELECT
            %I as name,
            COALESCE(SUM(diff), 0)::bigint as total_seconds
        FROM (
            SELECT
                %I,
                CASE
                    WHEN LAG(time) OVER (PARTITION BY %I ORDER BY time) IS NULL THEN 0
                    ELSE LEAST(EXTRACT(EPOCH FROM (time - LAG(time) OVER (PARTITION BY %I ORDER BY time))), $2)
                END as diff
            FROM heartbeats
            WHERE %I IS NOT NULL
            AND user_id = $1
            AND time IS NOT NULL
        ) capped_diffs
        GROUP BY %I
        ORDER BY total_seconds DESC
        LIMIT $3',
        p_field_name, p_field_name, p_field_name, p_field_name, p_field_name, p_field_name
    ) USING p_user_id, p_timeout_seconds, p_limit;
END;
$$ LANGUAGE plpgsql STABLE;
