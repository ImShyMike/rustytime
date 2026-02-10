CREATE OR REPLACE FUNCTION top_projects_by_range(
    p_user_id INT,
    p_timeout_seconds INT,
    p_start_time TIMESTAMPTZ,
    p_end_time TIMESTAMPTZ,
    p_limit INT
) RETURNS TABLE (
    name TEXT,
    project_url TEXT,
    total_seconds BIGINT
) AS $$
    WITH project_totals AS (
        SELECT
            resolved_project_id AS project_id,
            SUM(
                CASE
                    WHEN prev_time IS NULL THEN 0
                    ELSE LEAST(EXTRACT(EPOCH FROM (time - prev_time)), p_timeout_seconds)
                END
            )::bigint AS total_seconds
        FROM (
            SELECT
                h.time,
                par.resolved_project_id,
                LAG(h.time) OVER (
                    PARTITION BY par.resolved_project_id
                    ORDER BY h.time
                ) AS prev_time
            FROM heartbeats h
            JOIN project_alias_resolutions par
                ON par.user_id = h.user_id AND par.project_id = h.project_id
            WHERE h.user_id = p_user_id
              AND h.project_id IS NOT NULL
              AND h.time >= p_start_time
              AND h.time <= p_end_time
        ) resolved_with_lag
        GROUP BY resolved_project_id
    )
    SELECT
        p.name,
        p.project_url,
        pt.total_seconds
    FROM project_totals pt
    JOIN projects p ON p.id = pt.project_id
    WHERE p.hidden = false
      AND pt.total_seconds > 0
      AND NOT EXISTS (
          SELECT 1
          FROM project_alias_resolutions par
          WHERE par.user_id = p.user_id
            AND par.project_id = p.id
            AND par.resolved_project_id != p.id
      )
    ORDER BY pt.total_seconds DESC, p.name ASC
    LIMIT p_limit;
$$ LANGUAGE SQL STABLE;
