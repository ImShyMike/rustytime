-- Better dashboard stats function
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

-- Create covering index
CREATE INDEX IF NOT EXISTS idx_heartbeats_user_time_cover
ON heartbeats (user_id, time)
INCLUDE (project, language, editor, operating_system);
