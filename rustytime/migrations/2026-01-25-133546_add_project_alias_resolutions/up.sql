-- Create a cached alias-resolution mapping table
CREATE TABLE project_alias_resolutions (
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    project_id INT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    resolved_project_id INT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, project_id)
);

CREATE INDEX idx_par_user_resolved ON project_alias_resolutions(user_id, resolved_project_id);

-- Function to refresh the resolution table for a single user
CREATE OR REPLACE FUNCTION refresh_project_alias_resolutions(p_user_id INT)
RETURNS VOID AS $$
BEGIN
    DELETE FROM project_alias_resolutions WHERE user_id = p_user_id;

    -- Insert identity rows for all projects (project maps to itself)
    INSERT INTO project_alias_resolutions(user_id, project_id, resolved_project_id)
    SELECT p_user_id, p.id, p.id
    FROM projects p
    WHERE p.user_id = p_user_id;

    -- Overwrite aliased rows with final canonical target
    WITH RECURSIVE r AS (
        SELECT pa.project_id, pa.alias_to AS resolved, 1 AS depth, ARRAY[pa.project_id, pa.alias_to] AS path
        FROM project_aliases pa
        WHERE pa.user_id = p_user_id
        UNION ALL
        SELECT r.project_id, pa.alias_to, r.depth + 1, r.path || pa.alias_to
        FROM r
        JOIN project_aliases pa
            ON pa.user_id = p_user_id AND pa.project_id = r.resolved
        WHERE NOT (pa.alias_to = ANY(r.path))
          AND r.depth < 10
    ),
    final AS (
        SELECT DISTINCT ON (project_id) project_id, resolved
        FROM r
        ORDER BY project_id, depth DESC
    )
    UPDATE project_alias_resolutions par
    SET resolved_project_id = f.resolved, updated_at = now()
    FROM final f
    WHERE par.user_id = p_user_id AND par.project_id = f.project_id;
END;
$$ LANGUAGE plpgsql;

-- Trigger function for project_aliases changes
CREATE OR REPLACE FUNCTION project_aliases_refresh_trigger()
RETURNS TRIGGER AS $$
DECLARE uid INT;
BEGIN
    uid := COALESCE(NEW.user_id, OLD.user_id);
    PERFORM refresh_project_alias_resolutions(uid);
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_project_aliases_refresh
AFTER INSERT OR UPDATE OR DELETE ON project_aliases
FOR EACH ROW EXECUTE FUNCTION project_aliases_refresh_trigger();

-- Trigger function for projects changes (new project needs identity row)
CREATE OR REPLACE FUNCTION projects_alias_resolution_trigger()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO project_alias_resolutions(user_id, project_id, resolved_project_id)
        VALUES (NEW.user_id, NEW.id, NEW.id)
        ON CONFLICT (user_id, project_id) DO NOTHING;
    ELSIF TG_OP = 'DELETE' THEN
        DELETE FROM project_alias_resolutions 
        WHERE user_id = OLD.user_id AND project_id = OLD.id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_projects_alias_resolution
AFTER INSERT OR DELETE ON projects
FOR EACH ROW EXECUTE FUNCTION projects_alias_resolution_trigger();

-- Populate the table with existing data
DO $$
DECLARE
    uid INT;
BEGIN
    FOR uid IN SELECT DISTINCT id FROM users LOOP
        PERFORM refresh_project_alias_resolutions(uid);
    END LOOP;
END;
$$;
