-- Remove scheduled jobs
SELECT delete_job((SELECT job_id FROM timescaledb_information.jobs WHERE proc_name = 'cleanup_expired_sessions'));

-- Drop cleanup function
DROP FUNCTION IF EXISTS cleanup_expired_sessions();

-- Remove TimescaleDB policies
SELECT remove_compression_policy('heartbeats', if_exists => true);

-- Drop the heartbeats table
DROP TABLE IF EXISTS heartbeats CASCADE;

-- Drop the sessions table
DROP TABLE IF EXISTS sessions CASCADE;

-- Drop the users table
DROP TABLE IF EXISTS users CASCADE;

-- Disable TimescaleDB extension
DROP EXTENSION IF EXISTS timescaledb CASCADE;

-- Disable pgcrypto extension
DROP EXTENSION IF EXISTS "pgcrypto" CASCADE;