CREATE TABLE verifiable_credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_did TEXT NOT NULL REFERENCES agents(did),
    credential_type TEXT NOT NULL,
    credential_json JSONB NOT NULL,
    issuer_did TEXT NOT NULL,
    issuance_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expiration_date TIMESTAMPTZ NOT NULL,
    revoked BOOLEAN NOT NULL DEFAULT false,
    revoked_at TIMESTAMPTZ,
    revocation_reason TEXT,
    proof_type TEXT NOT NULL DEFAULT 'Ed25519Signature2020',
    proof_value TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_vc_agent_did ON verifiable_credentials(agent_did);
CREATE INDEX idx_vc_type ON verifiable_credentials(credential_type);
CREATE INDEX idx_vc_active ON verifiable_credentials(agent_did) WHERE revoked = false;
