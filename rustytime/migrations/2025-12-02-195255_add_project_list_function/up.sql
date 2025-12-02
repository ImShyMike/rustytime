-- Calculates total time spent on each project for a given user
CREATE OR REPLACE FUNCTION list_projects_with_time(
    p_user_id INT,
    p_timeout_seconds INT
) RETURNS TABLE (
    id INT,
    user_id INT,
    name TEXT,
    repo_url TEXT,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ,
    total_seconds BIGINT
) AS $$
    WITH resolved_projects AS (
        SELECT
            h.user_id,
            COALESCE(pa.alias_to, h.project_id) AS resolved_project_id,
            h.project_id,
            h.time,
            LAG(h.time) OVER (
                PARTITION BY COALESCE(pa.alias_to, h.project_id)
                ORDER BY h.time
            ) AS prev_time
        FROM heartbeats h
        LEFT JOIN project_aliases pa
            ON pa.user_id = h.user_id
            AND pa.project_id = h.project_id
        WHERE h.user_id = p_user_id
          AND h.project_id IS NOT NULL
    ),
    project_totals AS (
        SELECT
            resolved_project_id AS project_id,
            SUM(
                CASE
                    WHEN prev_time IS NULL THEN 0
                    ELSE LEAST(EXTRACT(EPOCH FROM (time - prev_time)), p_timeout_seconds)
                END
            )::bigint AS total_seconds
        FROM resolved_projects
        GROUP BY resolved_project_id
    )
    SELECT
        p.id,
        p.user_id,
        p.name,
        p.repo_url,
        p.created_at,
        p.updated_at,
        COALESCE(pt.total_seconds, 0)::bigint AS total_seconds
    FROM projects p
    LEFT JOIN project_totals pt ON pt.project_id = p.id
    WHERE p.user_id = p_user_id
        AND NOT EXISTS (
            SELECT 1
            FROM project_aliases pa
            WHERE pa.user_id = p.user_id
                AND pa.project_id = p.id
        )
    ORDER BY COALESCE(pt.total_seconds, 0) DESC, p.name ASC;
$$ LANGUAGE SQL STABLE;
