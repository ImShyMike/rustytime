-- Faster time per project queries
CREATE INDEX idx_heartbeats_user_project_time ON heartbeats (user_id, project_id, time);
