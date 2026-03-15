-- AgentTrust Phase 2: oauth_tokens table
CREATE TABLE IF NOT EXISTS oauth_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    access_token_hash TEXT UNIQUE NOT NULL,
    refresh_token_hash TEXT UNIQUE,
    client_id TEXT NOT NULL REFERENCES oauth_clients(client_id),
    agent_did TEXT NOT NULL REFERENCES agents(did),
    scopes JSONB NOT NULL,
    access_token_expires_at TIMESTAMPTZ NOT NULL,
    refresh_token_expires_at TIMESTAMPTZ,
    revoked BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_oauth_tokens_access ON oauth_tokens(access_token_hash);
CREATE INDEX IF NOT EXISTS idx_oauth_tokens_refresh ON oauth_tokens(refresh_token_hash);
