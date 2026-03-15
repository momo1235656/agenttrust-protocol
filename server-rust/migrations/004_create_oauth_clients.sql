-- AgentTrust Phase 2: oauth_clients table
CREATE TABLE IF NOT EXISTS oauth_clients (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id TEXT UNIQUE NOT NULL,
    client_secret_hash TEXT NOT NULL,
    agent_did TEXT NOT NULL REFERENCES agents(did),
    client_name TEXT NOT NULL,
    redirect_uris JSONB DEFAULT '[]'::jsonb,
    allowed_scopes JSONB DEFAULT '["payment:execute","balance:read"]'::jsonb,
    client_type TEXT DEFAULT 'confidential',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Authorization codes for OAuth 2.0 Authorization Code Grant
CREATE TABLE IF NOT EXISTS oauth_authorization_codes (
    code TEXT PRIMARY KEY,
    client_id TEXT NOT NULL REFERENCES oauth_clients(client_id),
    agent_did TEXT NOT NULL REFERENCES agents(did),
    redirect_uri TEXT NOT NULL,
    scopes JSONB NOT NULL,
    code_challenge TEXT,
    code_challenge_method TEXT DEFAULT 'S256',
    expires_at TIMESTAMPTZ NOT NULL,
    used BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
