-- Remove expired sessions older than 30 days
CREATE OR REPLACE FUNCTION cleanup_expired_sessions() RETURNS void AS $$
BEGIN
  DELETE FROM sessions WHERE expires_at < now() - INTERVAL '30 days';
END;
$$ LANGUAGE plpgsql;

-- Schedule session cleanup to run daily
SELECT add_job('cleanup_expired_sessions', '1 day');
