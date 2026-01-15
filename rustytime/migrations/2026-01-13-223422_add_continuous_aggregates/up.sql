-- Daily aggregates
CREATE OR REPLACE VIEW heartbeat_daily_agg AS
SELECT 
  user_id,
  time_bucket('1 day'::interval, time) AS day,
  project,
  editor,
  operating_system,
  language,
  COALESCE(SUM(diff), 0)::bigint AS total_duration
FROM (
  SELECT
    user_id,
    time,
    project,
    editor,
    operating_system,
    language,
    CASE
      WHEN LAG(time) OVER (PARTITION BY user_id, project, editor, operating_system, language ORDER BY time) IS NULL THEN 0
      ELSE LEAST(EXTRACT(EPOCH FROM (time - LAG(time) OVER (PARTITION BY user_id, project, editor, operating_system, language ORDER BY time))), 120)
    END as diff
  FROM heartbeats
) capped_diffs
GROUP BY user_id, time_bucket('1 day'::interval, time), project, editor, operating_system, language;

-- Weekly aggregates
CREATE OR REPLACE VIEW heartbeat_weekly_agg AS
SELECT 
  user_id,
  time_bucket('7 days'::interval, time) AS week,
  project,
  editor,
  operating_system,
  language,
  COALESCE(SUM(diff), 0)::bigint AS total_duration
FROM (
  SELECT
    user_id,
    time,
    project,
    editor,
    operating_system,
    language,
    CASE
      WHEN LAG(time) OVER (PARTITION BY user_id, project, editor, operating_system, language ORDER BY time) IS NULL THEN 0
      ELSE LEAST(EXTRACT(EPOCH FROM (time - LAG(time) OVER (PARTITION BY user_id, project, editor, operating_system, language ORDER BY time))), 120)
    END as diff
  FROM heartbeats
) capped_diffs
GROUP BY user_id, time_bucket('7 days'::interval, time), project, editor, operating_system, language;

-- Monthly aggregates
CREATE OR REPLACE VIEW heartbeat_monthly_agg AS
SELECT 
  user_id,
  time_bucket('30 days'::interval, time) AS month,
  project,
  editor,
  operating_system,
  language,
  COALESCE(SUM(diff), 0)::bigint AS total_duration
FROM (
  SELECT
    user_id,
    time,
    project,
    editor,
    operating_system,
    language,
    CASE
      WHEN LAG(time) OVER (PARTITION BY user_id, project, editor, operating_system, language ORDER BY time) IS NULL THEN 0
      ELSE LEAST(EXTRACT(EPOCH FROM (time - LAG(time) OVER (PARTITION BY user_id, project, editor, operating_system, language ORDER BY time))), 120)
    END as diff
  FROM heartbeats
) capped_diffs
GROUP BY user_id, time_bucket('30 days'::interval, time), project, editor, operating_system, language;
