-- Remove the scheduled job
SELECT delete_job((SELECT job_id FROM timescaledb_information.jobs WHERE proc_name = 'cleanup_expired_sessions'));

-- Drop the cleanup function
DROP FUNCTION IF EXISTS cleanup_expired_sessions();
