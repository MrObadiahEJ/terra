-- RFC-007: Dispute resolution & parcel freeze protocol.
-- Mirrors the on-chain Dispute account and dispute lifecycle.

CREATE TABLE IF NOT EXISTS disputes (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parcel_id       UUID NOT NULL REFERENCES parcels(id) ON DELETE CASCADE,
    filed_by        TEXT NOT NULL,
    case_hash       TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'filed',
    required        SMALLINT NOT NULL DEFAULT 2,
    count           SMALLINT NOT NULL DEFAULT 0,
    validators      TEXT[] NOT NULL DEFAULT '{}',
    filed_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    frozen_at       TIMESTAMPTZ,
    adjudicated_at  TIMESTAMPTZ,
    outcome         TEXT,
    new_owner       TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_disputes_parcel ON disputes (parcel_id);
CREATE INDEX IF NOT EXISTS idx_disputes_status ON disputes (status);
CREATE INDEX IF NOT EXISTS idx_disputes_filed_by ON disputes (filed_by);
