-- Drop the heartbeats table
DROP TABLE IF EXISTS heartbeats;

-- Drop the users table
DROP TABLE IF EXISTS users;

-- Disable TimescaleDB extension
DROP EXTENSION IF EXISTS timescaledb;

-- Disable pgcrypto extension
DROP EXTENSION IF EXISTS "pgcrypto";