CREATE TABLE flow_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_did TEXT REFERENCES agents(did),
    max_transactions_per_minute INTEGER NOT NULL DEFAULT 10,
    max_transactions_per_hour INTEGER NOT NULL DEFAULT 100,
    max_transactions_per_day INTEGER NOT NULL DEFAULT 1000,
    max_a2a_with_same_agent_per_day INTEGER NOT NULL DEFAULT 10,
    max_chain_depth INTEGER NOT NULL DEFAULT 5,
    max_saga_timeout_minutes INTEGER NOT NULL DEFAULT 60,
    max_escrow_timeout_hours INTEGER NOT NULL DEFAULT 24,
    auto_freeze_on_consecutive_failures INTEGER NOT NULL DEFAULT 10,
    auto_freeze_on_daily_amount_exceed BIGINT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
