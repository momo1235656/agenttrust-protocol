-- AgentTrust Phase 2: audit_logs table
CREATE TABLE IF NOT EXISTS audit_logs (
    id BIGSERIAL PRIMARY KEY,
    index_num BIGINT NOT NULL,
    agent_did TEXT NOT NULL REFERENCES agents(did),
    transaction_id TEXT NOT NULL REFERENCES transactions(id),
    amount BIGINT NOT NULL,
    status TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    prev_hash TEXT NOT NULL,
    hash TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_audit_logs_agent_did ON audit_logs(agent_did);
CREATE INDEX IF NOT EXISTS idx_audit_logs_index ON audit_logs(agent_did, index_num);
