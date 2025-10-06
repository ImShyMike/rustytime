-- Drop index on heartbeats.project_id
DROP INDEX IF EXISTS idx_heartbeats_project_id_time;

-- Drop project_id column from heartbeats
ALTER TABLE heartbeats DROP COLUMN IF EXISTS project_id;

-- Drop indexes on projects
DROP INDEX IF EXISTS idx_projects_user_id;
DROP INDEX IF EXISTS idx_projects_name;

-- Disable automatic updated_at management for projects
SELECT diesel_manage_updated_at('projects', 'disable');

-- Drop projects table
DROP TABLE IF EXISTS projects;
