-- Faster project ID lookups
CREATE INDEX idx_heartbeats_user_project_id ON heartbeats (user_id, project_id);