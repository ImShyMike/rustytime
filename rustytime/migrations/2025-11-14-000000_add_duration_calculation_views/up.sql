-- Create a function to calculate user duration with filters
CREATE OR REPLACE FUNCTION calculate_user_duration(
    p_user_id INT,
    p_start_date TIMESTAMPTZ,
    p_end_date TIMESTAMPTZ,
    p_project TEXT,
    p_language TEXT,
    p_entity TEXT,
    p_type TEXT,
    p_timeout_seconds INT
) RETURNS BIGINT AS $$
    SELECT COALESCE(SUM(diff), 0)::bigint
    FROM (
        SELECT CASE
            WHEN LAG(time) OVER (ORDER BY time) IS NULL THEN 0
            ELSE LEAST(EXTRACT(EPOCH FROM (time - LAG(time) OVER (ORDER BY time))), p_timeout_seconds)
        END as diff
        FROM heartbeats
        WHERE (p_user_id IS NULL OR user_id = p_user_id)
          AND (p_start_date IS NULL OR time >= p_start_date)
          AND (p_end_date IS NULL OR time <= p_end_date)
          AND (p_project IS NULL OR project = p_project)
          AND (p_language IS NULL OR language = p_language)
          AND (p_entity IS NULL OR entity = p_entity)
          AND (p_type IS NULL OR type = p_type)
          AND time IS NOT NULL
        ORDER BY time ASC
    ) capped_diffs;
$$ LANGUAGE SQL STABLE;

-- Create a function to calculate durations for all users in a time range
CREATE OR REPLACE FUNCTION calculate_all_user_durations(
    p_start_time TIMESTAMPTZ,
    p_end_time TIMESTAMPTZ,
    p_timeout_seconds INT
) RETURNS TABLE (
    user_id INT,
    total_seconds BIGINT
) AS $$
    SELECT
        user_id,
        COALESCE(SUM(diff), 0)::bigint as total_seconds
    FROM (
        SELECT
            user_id,
            CASE
                WHEN LAG(time) OVER (PARTITION BY user_id ORDER BY time) IS NULL THEN 0
                ELSE LEAST(EXTRACT(EPOCH FROM (time - LAG(time) OVER (PARTITION BY user_id ORDER BY time))), p_timeout_seconds)
            END as diff
        FROM heartbeats
        WHERE time >= p_start_time
          AND time < p_end_time
          AND time IS NOT NULL
        ORDER BY user_id, time ASC
    ) capped_diffs
    GROUP BY user_id
    ORDER BY total_seconds DESC;
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

-- Create a function to calculate project time with aliases for a specific project
CREATE OR REPLACE FUNCTION calculate_project_time_with_aliases(
    p_user_id INT,
    p_project_id INT,
    p_timeout_seconds INT
) RETURNS BIGINT AS $$
    SELECT COALESCE(SUM(diff), 0)::bigint
    FROM (
        SELECT
            CASE
                WHEN prev_time IS NULL THEN 0
                ELSE LEAST(EXTRACT(EPOCH FROM (time - prev_time)), p_timeout_seconds)
            END AS diff
        FROM (
            SELECT
                time,
                LAG(time) OVER (ORDER BY time) AS prev_time
            FROM heartbeats h
            WHERE h.user_id = p_user_id
              AND h.project_id IS NOT NULL
              AND (
                  h.project_id = p_project_id
                  OR h.project_id IN (
                      SELECT pa.project_id
                      FROM project_aliases pa
                      WHERE pa.user_id = p_user_id
                        AND pa.alias_to = p_project_id
                  )
              )
        ) time_diffs
    ) capped_diffs;
$$ LANGUAGE SQL STABLE;
