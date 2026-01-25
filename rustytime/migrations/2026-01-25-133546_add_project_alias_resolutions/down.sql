DROP TRIGGER IF EXISTS trg_projects_alias_resolution ON projects;
DROP FUNCTION IF EXISTS projects_alias_resolution_trigger();
DROP TRIGGER IF EXISTS trg_project_aliases_refresh ON project_aliases;
DROP FUNCTION IF EXISTS project_aliases_refresh_trigger();
DROP FUNCTION IF EXISTS refresh_project_alias_resolutions(INT);
DROP TABLE IF EXISTS project_alias_resolutions;
