-- Add project_url column
ALTER TABLE projects ADD COLUMN project_url TEXT;

-- Copy data from repo_url to project_url
UPDATE projects SET project_url = repo_url WHERE repo_url IS NOT NULL;

-- Set repo_url to NULL
UPDATE projects SET repo_url = NULL;

-- Drop and recreate list_projects_with_time to include project_url
DROP FUNCTION IF EXISTS list_projects_with_time(INT, INT);
CREATE FUNCTION list_projects_with_time(
    p_user_id INT,
    p_timeout_seconds INT
) RETURNS TABLE (
    id INT,
    user_id INT,
    name TEXT,
    repo_url TEXT,
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ,
    total_seconds BIGINT,
    hidden BOOLEAN,
    project_url TEXT
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
        ) resolved_with_lag
        GROUP BY resolved_project_id
    )
    SELECT
        p.id,
        p.user_id,
        p.name,
        p.repo_url,
        p.created_at,
        p.updated_at,
        COALESCE(pt.total_seconds, 0)::bigint AS total_seconds,
        p.hidden,
        p.project_url
    FROM projects p
    LEFT JOIN project_totals pt ON pt.project_id = p.id
    WHERE p.user_id = p_user_id
        -- Only show canonical projects (not aliased to something else)
        AND NOT EXISTS (
            SELECT 1
            FROM project_alias_resolutions par
            WHERE par.user_id = p.user_id
                AND par.project_id = p.id
                AND par.resolved_project_id != p.id
        )
    ORDER BY COALESCE(pt.total_seconds, 0) DESC, p.name ASC;
$$ LANGUAGE SQL STABLE;
