-- Escrow settlement records (RFC-004)
CREATE TABLE IF NOT EXISTS escrows (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parcel_id         UUID NOT NULL REFERENCES parcels(id) ON DELETE CASCADE,
    seller            TEXT NOT NULL,
    buyer             TEXT NOT NULL,
    amount            BIGINT NOT NULL,
    deposit_amount    BIGINT NOT NULL DEFAULT 0,
    vault             TEXT NOT NULL DEFAULT '',
    status            TEXT NOT NULL DEFAULT 'created',
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    deposited_at      TIMESTAMPTZ,
    accepted_at       TIMESTAMPTZ,
    settle_deadline   TIMESTAMPTZ,
    cancel_deadline   TIMESTAMPTZ NOT NULL,
    dispute_case_hash TEXT
);

CREATE INDEX IF NOT EXISTS idx_escrows_parcel   ON escrows (parcel_id);
CREATE INDEX IF NOT EXISTS idx_escrows_seller   ON escrows (seller);
CREATE INDEX IF NOT EXISTS idx_escrows_buyer    ON escrows (buyer);
CREATE INDEX IF NOT EXISTS idx_escrows_status   ON escrows (status);
