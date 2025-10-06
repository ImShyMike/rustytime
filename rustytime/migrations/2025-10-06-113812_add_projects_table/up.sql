-- Create the projects table
CREATE TABLE projects (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    repo_url TEXT,
    created_at TIMESTAMPTZ DEFAULT now (),
    updated_at TIMESTAMPTZ DEFAULT now (),
    UNIQUE (user_id, name)
);

-- Auto manage updated_at column
SELECT diesel_manage_updated_at ('projects');

-- Create indexes on projects
CREATE INDEX idx_projects_user_id ON projects (user_id);
CREATE INDEX idx_projects_name ON projects (name);

-- Add project_id column to heartbeats
ALTER TABLE heartbeats ADD COLUMN project_id INT;

-- Create index on heartbeats.project_id
CREATE INDEX idx_heartbeats_project_id_time 
  ON heartbeats (project_id, time DESC);
