-- Update calculate_user_duration to use resolved project aliases
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
DECLARE
    v_target_project_id INT;
BEGIN
    -- If filtering by project name, resolve it to canonical project_id
    IF p_project IS NOT NULL AND p_user_id IS NOT NULL THEN
        SELECT par.resolved_project_id INTO v_target_project_id
        FROM projects p
        JOIN project_alias_resolutions par ON par.user_id = p.user_id AND par.project_id = p.id
        WHERE p.user_id = p_user_id AND p.name = p_project
        LIMIT 1;
        
        IF v_target_project_id IS NULL THEN
            RETURN 0; -- project not found
        END IF;
        
        -- Calculate duration for all heartbeats that resolve to this project
        RETURN (
            SELECT COALESCE(SUM(diff), 0)::bigint
            FROM (
                SELECT CASE
                    WHEN LAG(time) OVER (ORDER BY time) IS NULL THEN 0
                    ELSE LEAST(EXTRACT(EPOCH FROM (time - LAG(time) OVER (ORDER BY time))), p_timeout_seconds)
                END as diff
                FROM heartbeats h
                JOIN project_alias_resolutions par ON par.user_id = h.user_id AND par.project_id = h.project_id
                WHERE (p_user_id IS NULL OR h.user_id = p_user_id)
                  AND par.resolved_project_id = v_target_project_id
                  AND (p_start_date IS NULL OR h.time >= p_start_date)
                  AND (p_end_date IS NULL OR h.time <= p_end_date)
                  AND (p_language IS NULL OR h.language = p_language)
                  AND (p_entity IS NULL OR h.entity = p_entity)
                  AND (p_type IS NULL OR h.type = p_type)
                  AND h.time IS NOT NULL
                ORDER BY h.time ASC
            ) capped_diffs
        );
    ELSE
        -- No project filter, use original logic
        RETURN (
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
                  AND (p_language IS NULL OR language = p_language)
                  AND (p_entity IS NULL OR entity = p_entity)
                  AND (p_type IS NULL OR type = p_type)
                  AND time IS NOT NULL
                ORDER BY time ASC
            ) capped_diffs
        );
    END IF;
END;
$$ LANGUAGE plpgsql STABLE;

-- Update list_projects_with_time to use the resolution table
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
        COALESCE(pt.total_seconds, 0)::bigint AS total_seconds
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

-- Update calculate_project_stats_with_aliases to use the resolution table
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
                    par.resolved_project_id,
                    LAG(h.time) OVER (
                        PARTITION BY par.resolved_project_id
                        ORDER BY h.time
                    ) as prev_time
                FROM heartbeats h
                JOIN project_alias_resolutions par
                    ON par.user_id = h.user_id
                    AND par.project_id = h.project_id
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

-- Update calculate_dashboard_stats_by_range to use resolved project aliases
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
    SELECT h.time, par.resolved_project_id, h.language, h.editor, h.operating_system
    FROM heartbeats h
    LEFT JOIN project_alias_resolutions par 
        ON par.user_id = h.user_id AND par.project_id = h.project_id
    WHERE h.user_id = p_user_id
      AND h.time >= p_start_time
      AND h.time IS NOT NULL
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
        p.name as name,
        CAST(COALESCE(SUM(diff), 0) AS BIGINT) as total_seconds,
        (SELECT total FROM total_time_calc) as total_time,
        ROW_NUMBER() OVER (ORDER BY SUM(diff) DESC) as rn
    FROM (
        SELECT
            resolved_project_id,
            CASE
                WHEN LAG(time) OVER (PARTITION BY resolved_project_id ORDER BY time) IS NULL THEN 0
                ELSE LEAST(EXTRACT(EPOCH FROM (time - LAG(time) OVER (PARTITION BY resolved_project_id ORDER BY time))), p_timeout_seconds)
            END as diff
        FROM base_heartbeats
        WHERE resolved_project_id IS NOT NULL
    ) capped_diffs
    JOIN projects p ON p.id = capped_diffs.resolved_project_id
    GROUP BY p.id, p.name
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

-- Update calculate_dashboard_stats to use resolution table
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

-- Add covering index that includes project_id
DROP INDEX IF EXISTS idx_heartbeats_user_time_cover;
CREATE INDEX idx_heartbeats_user_time_cover
ON heartbeats (user_id, time)
INCLUDE (project_id, language, editor, operating_system, entity, type);
