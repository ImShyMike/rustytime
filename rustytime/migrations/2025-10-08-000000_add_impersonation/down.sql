DROP INDEX IF EXISTS idx_sessions_impersonated_by;

ALTER TABLE sessions DROP COLUMN impersonated_by;
