ALTER TABLE sessions ADD COLUMN impersonated_by INTEGER REFERENCES users (id);

CREATE INDEX idx_sessions_impersonated_by ON sessions(impersonated_by);
