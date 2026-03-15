-- AgentTrust PostgreSQL initialization
-- This file runs when the container is first created.
-- Actual schema migrations are handled by sqlx::migrate! at server startup.

-- Enable pgcrypto for gen_random_uuid()
CREATE EXTENSION IF NOT EXISTS pgcrypto;
