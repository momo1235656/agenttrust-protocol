-- AgentTrust Phase 2: agents table
CREATE TABLE IF NOT EXISTS agents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    did TEXT UNIQUE NOT NULL,
    public_key BYTEA NOT NULL,
    display_name TEXT,
    did_document JSONB NOT NULL DEFAULT '{}'::jsonb,
    max_transaction_limit BIGINT DEFAULT 100000,
    daily_transaction_limit BIGINT DEFAULT 500000,
    allowed_categories JSONB DEFAULT '[]'::jsonb,
    requires_approval_above BIGINT DEFAULT 30000,
    is_active BOOLEAN DEFAULT true,
    frozen_at TIMESTAMPTZ,
    frozen_reason TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_agents_did ON agents(did);
