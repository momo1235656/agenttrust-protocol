CREATE TABLE a2a_transfers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sender_did TEXT NOT NULL REFERENCES agents(did),
    sender_trust_score SMALLINT NOT NULL,
    receiver_did TEXT NOT NULL REFERENCES agents(did),
    receiver_trust_score SMALLINT NOT NULL,
    amount BIGINT NOT NULL,
    currency TEXT NOT NULL DEFAULT 'jpy',
    description TEXT,
    service_type TEXT,
    status TEXT NOT NULL DEFAULT 'initiated',
    escrow_id UUID,
    saga_id UUID,
    stripe_transfer_id TEXT,
    stripe_payment_intent_id TEXT,
    initiated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    trust_verified_at TIMESTAMPTZ,
    escrowed_at TIMESTAMPTZ,
    service_completed_at TIMESTAMPTZ,
    released_at TIMESTAMPTZ,
    settled_at TIMESTAMPTZ,
    timeout_at TIMESTAMPTZ,
    sender_audit_hash TEXT,
    receiver_audit_hash TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_a2a_sender ON a2a_transfers(sender_did);
CREATE INDEX idx_a2a_receiver ON a2a_transfers(receiver_did);
CREATE INDEX idx_a2a_status ON a2a_transfers(status);
CREATE INDEX idx_a2a_saga ON a2a_transfers(saga_id);
