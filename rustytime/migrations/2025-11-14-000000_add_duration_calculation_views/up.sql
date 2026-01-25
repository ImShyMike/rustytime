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
