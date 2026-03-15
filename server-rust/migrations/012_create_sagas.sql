CREATE TABLE sagas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    a2a_transfer_id UUID NOT NULL REFERENCES a2a_transfers(id),
    status TEXT NOT NULL DEFAULT 'started',
    current_step INTEGER NOT NULL DEFAULT 0,
    total_steps INTEGER NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    timeout_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    compensated_at TIMESTAMPTZ,
    error_step INTEGER,
    error_message TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_sagas_transfer ON sagas(a2a_transfer_id);
CREATE INDEX idx_sagas_status ON sagas(status);
CREATE INDEX idx_sagas_timeout ON sagas(timeout_at) WHERE status IN ('started', 'in_progress');
