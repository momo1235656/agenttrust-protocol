-- AgentTrust Phase 2: transactions table
CREATE TABLE IF NOT EXISTS transactions (
    id TEXT PRIMARY KEY,
    agent_did TEXT NOT NULL REFERENCES agents(did),
    amount BIGINT NOT NULL,
    currency TEXT DEFAULT 'jpy',
    description TEXT,
    status TEXT NOT NULL,
    payment_provider TEXT DEFAULT 'stripe',
    provider_payment_id TEXT,
    idempotency_key TEXT UNIQUE,
    approval_id UUID,
    audit_hash TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_transactions_agent_did ON transactions(agent_did);
CREATE INDEX IF NOT EXISTS idx_transactions_idempotency ON transactions(idempotency_key);
CREATE INDEX IF NOT EXISTS idx_transactions_created_at ON transactions(created_at);
