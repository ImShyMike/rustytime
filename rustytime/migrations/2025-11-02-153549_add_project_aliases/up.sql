-- Create project aliases table
CREATE TABLE project_aliases (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    alias_to INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    UNIQUE(user_id, project_id),
    CHECK(project_id != alias_to)
);

CREATE INDEX idx_project_aliases_user_project ON project_aliases(user_id, project_id);
CREATE INDEX idx_project_aliases_lookup ON project_aliases(user_id, alias_to);
