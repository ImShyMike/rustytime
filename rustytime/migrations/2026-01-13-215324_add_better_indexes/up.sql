CREATE INDEX idx_heartbeats_editor_notnull 
  ON heartbeats(user_id, time DESC, editor) 
  WHERE editor IS NOT NULL;

CREATE INDEX idx_heartbeats_language_notnull 
  ON heartbeats(user_id, time DESC, language) 
  WHERE language IS NOT NULL;

CREATE INDEX idx_heartbeats_os_notnull 
  ON heartbeats(user_id, time DESC, operating_system) 
  WHERE operating_system IS NOT NULL;

CREATE INDEX idx_heartbeats_category_notnull 
  ON heartbeats(user_id, time DESC, category) 
  WHERE category IS NOT NULL;

CREATE INDEX idx_heartbeats_user_project_time_desc 
  ON heartbeats(user_id, project_id, time DESC) 
  WHERE project_id IS NOT NULL;
