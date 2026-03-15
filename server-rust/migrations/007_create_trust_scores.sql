CREATE TABLE trust_scores (
    id BIGSERIAL,
    agent_did TEXT NOT NULL REFERENCES agents(did),
    score SMALLINT NOT NULL CHECK (score >= 0 AND score <= 100),
    total_transactions BIGINT NOT NULL DEFAULT 0,
    successful_transactions BIGINT NOT NULL DEFAULT 0,
    failed_transactions BIGINT NOT NULL DEFAULT 0,
    success_rate DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    dispute_count BIGINT NOT NULL DEFAULT 0,
    dispute_rate DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    total_volume BIGINT NOT NULL DEFAULT 0,
    avg_transaction_value BIGINT NOT NULL DEFAULT 0,
    unique_counterparties BIGINT NOT NULL DEFAULT 0,
    account_age_days INTEGER NOT NULL DEFAULT 0,
    calculation_version TEXT NOT NULL DEFAULT 'v1.0',
    calculated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (agent_did, calculated_at)
);
CREATE INDEX idx_trust_scores_did ON trust_scores(agent_did, calculated_at DESC);
CREATE INDEX idx_trust_scores_score ON trust_scores(score);
