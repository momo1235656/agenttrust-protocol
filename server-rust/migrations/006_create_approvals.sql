-- AgentTrust Phase 2: approvals table (Human-in-the-Loop)
CREATE TABLE IF NOT EXISTS approvals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_did TEXT NOT NULL REFERENCES agents(did),
    transaction_amount BIGINT NOT NULL,
    transaction_currency TEXT DEFAULT 'jpy',
    transaction_description TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    requested_at TIMESTAMPTZ DEFAULT NOW(),
    responded_at TIMESTAMPTZ,
    responded_by TEXT,
    webhook_url TEXT,
    expires_at TIMESTAMPTZ NOT NULL,
    idempotency_key TEXT
);

CREATE INDEX IF NOT EXISTS idx_approvals_agent_did ON approvals(agent_did);
CREATE INDEX IF NOT EXISTS idx_approvals_status ON approvals(status);
