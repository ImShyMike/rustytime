-- Restore original calculate_user_duration
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

-- Restore original list_projects_with_time
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

-- Restore original calculate_project_stats_with_aliases
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

-- Restore original calculate_dashboard_stats_by_range
CREATE OR REPLACE FUNCTION calculate_dashboard_stats_by_range(
    p_user_id INT,
    p_start_time TIMESTAMPTZ,
    p_timeout_seconds INT,
    p_limit_count INT DEFAULT 10
) RETURNS TABLE (
    metric_type TEXT,
    name TEXT,
    total_seconds BIGINT,
    total_time BIGINT
) AS $$
WITH 
base_heartbeats AS (
    SELECT time, project, language, editor, operating_system
    FROM heartbeats
    WHERE user_id = p_user_id
      AND time >= p_start_time
      AND time IS NOT NULL
),
total_time_calc AS (
    SELECT CAST(COALESCE(SUM(diff), 0) AS BIGINT) as total
    FROM (
        SELECT CASE
            WHEN LAG(time) OVER (ORDER BY time) IS NULL THEN 0
            ELSE LEAST(EXTRACT(EPOCH FROM (time - LAG(time) OVER (ORDER BY time))), p_timeout_seconds)
        END as diff
        FROM base_heartbeats
    ) capped_diffs
),
projects AS (
    SELECT 
        'project' as metric_type,
        project as name,
        CAST(COALESCE(SUM(diff), 0) AS BIGINT) as total_seconds,
        (SELECT total FROM total_time_calc) as total_time,
        ROW_NUMBER() OVER (ORDER BY SUM(diff) DESC) as rn
    FROM (
        SELECT
            project,
            CASE
                WHEN LAG(time) OVER (PARTITION BY project ORDER BY time) IS NULL THEN 0
                ELSE LEAST(EXTRACT(EPOCH FROM (time - LAG(time) OVER (PARTITION BY project ORDER BY time))), p_timeout_seconds)
            END as diff
        FROM base_heartbeats
        WHERE project IS NOT NULL
    ) capped_diffs
    GROUP BY project
    HAVING SUM(diff) > 0
),
editors AS (
    SELECT 
        'editor' as metric_type,
        editor as name,
        CAST(COALESCE(SUM(diff), 0) AS BIGINT) as total_seconds,
        (SELECT total FROM total_time_calc) as total_time,
        ROW_NUMBER() OVER (ORDER BY SUM(diff) DESC) as rn
    FROM (
        SELECT
            editor,
            CASE
                WHEN LAG(time) OVER (PARTITION BY editor ORDER BY time) IS NULL THEN 0
                ELSE LEAST(EXTRACT(EPOCH FROM (time - LAG(time) OVER (PARTITION BY editor ORDER BY time))), p_timeout_seconds)
            END as diff
        FROM base_heartbeats
        WHERE editor IS NOT NULL
    ) capped_diffs
    GROUP BY editor
    HAVING SUM(diff) > 0
),
oses AS (
    SELECT 
        'operating_system' as metric_type,
        operating_system as name,
        CAST(COALESCE(SUM(diff), 0) AS BIGINT) as total_seconds,
        (SELECT total FROM total_time_calc) as total_time,
        ROW_NUMBER() OVER (ORDER BY SUM(diff) DESC) as rn
    FROM (
        SELECT
            operating_system,
            CASE
                WHEN LAG(time) OVER (PARTITION BY operating_system ORDER BY time) IS NULL THEN 0
                ELSE LEAST(EXTRACT(EPOCH FROM (time - LAG(time) OVER (PARTITION BY operating_system ORDER BY time))), p_timeout_seconds)
            END as diff
        FROM base_heartbeats
        WHERE operating_system IS NOT NULL
    ) capped_diffs
    GROUP BY operating_system
    HAVING SUM(diff) > 0
),
languages AS (
    SELECT 
        'language' as metric_type,
        language as name,
        CAST(COALESCE(SUM(diff), 0) AS BIGINT) as total_seconds,
        (SELECT total FROM total_time_calc) as total_time,
        ROW_NUMBER() OVER (ORDER BY SUM(diff) DESC) as rn
    FROM (
        SELECT
            language,
            CASE
                WHEN LAG(time) OVER (PARTITION BY language ORDER BY time) IS NULL THEN 0
                ELSE LEAST(EXTRACT(EPOCH FROM (time - LAG(time) OVER (PARTITION BY language ORDER BY time))), p_timeout_seconds)
            END as diff
        FROM base_heartbeats
        WHERE language IS NOT NULL
    ) capped_diffs
    GROUP BY language
    HAVING SUM(diff) > 0
)
SELECT 'total_time'::text, NULL::text, 0::bigint, (SELECT total FROM total_time_calc)
UNION ALL
SELECT metric_type, name, total_seconds, total_time FROM projects WHERE rn <= p_limit_count
UNION ALL
SELECT metric_type, name, total_seconds, total_time FROM editors WHERE rn <= p_limit_count
UNION ALL
SELECT metric_type, name, total_seconds, total_time FROM oses WHERE rn <= p_limit_count
UNION ALL
SELECT metric_type, name, total_seconds, total_time FROM languages WHERE rn <= p_limit_count;
$$ LANGUAGE SQL STABLE;

-- Restore original calculate_dashboard_stats
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

-- Restore original covering index
DROP INDEX IF EXISTS idx_heartbeats_user_time_cover;
CREATE INDEX idx_heartbeats_user_time_cover
ON heartbeats (user_id, time)
INCLUDE (project, language, editor, operating_system);
