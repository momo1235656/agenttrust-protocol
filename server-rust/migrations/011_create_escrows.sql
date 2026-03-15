CREATE TABLE escrows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    a2a_transfer_id UUID NOT NULL REFERENCES a2a_transfers(id),
    amount BIGINT NOT NULL,
    currency TEXT NOT NULL DEFAULT 'jpy',
    payer_did TEXT NOT NULL REFERENCES agents(did),
    payee_did TEXT NOT NULL REFERENCES agents(did),
    status TEXT NOT NULL DEFAULT 'pending',
    stripe_payment_intent_id TEXT,
    stripe_transfer_id TEXT,
    funded_at TIMESTAMPTZ,
    released_at TIMESTAMPTZ,
    refunded_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ NOT NULL,
    dispute_reason TEXT,
    dispute_opened_at TIMESTAMPTZ,
    dispute_resolved_at TIMESTAMPTZ,
    dispute_resolution TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_escrows_transfer ON escrows(a2a_transfer_id);
CREATE INDEX idx_escrows_status ON escrows(status);
CREATE INDEX idx_escrows_expires ON escrows(expires_at) WHERE status = 'funded';
