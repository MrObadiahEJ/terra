-- Phase 2/3 — Validator-endorsed passation + judicial forfeiture mirror.
-- Closes the stolen-wallet hole: a succession (passation) now requires BOTH a
-- configurable grace window AND a minimum number of validator endorsements
-- before it can be claimed. Also records collective validator forfeiture
-- (judicial seizure) per a court order and its court-relay fail-safe.

-- Configurable grace + endorsement threshold on in-flight successions
-- (mirrors the on-chain Succession account additions: grace_secs, required,
-- validations_count, validators).
ALTER TABLE successions
    ADD COLUMN IF NOT EXISTS grace_secs           BIGINT NOT NULL DEFAULT 2592000, -- 30 days
    ADD COLUMN IF NOT EXISTS required             SMALLINT NOT NULL DEFAULT 1,
    ADD COLUMN IF NOT EXISTS validations_count    SMALLINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS validators           TEXT[] NOT NULL DEFAULT '{}';

-- Each validator endorsement of a succession, for auditability (who signed,
-- when). Mirrors the on-chain SuccessionEndorsed event.
CREATE TABLE IF NOT EXISTS succession_endorsements (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    succession_id  UUID NOT NULL REFERENCES successions(id) ON DELETE CASCADE,
    validator      TEXT NOT NULL,              -- base58 signing wallet
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (succession_id, validator)
);

CREATE INDEX IF NOT EXISTS idx_succession_endorsements_succession
    ON succession_endorsements (succession_id);

-- Judicial forfeiture mirror (on-chain ParcelForfeited event). Deliberately
-- heavier than a normal transfer: threshold (>=2) validators must have signed.
-- The court_relay wallet is a fail-safe operator channel that reconciles the
-- DB row if the on-chain relay was skipped.
CREATE TABLE IF NOT EXISTS forfeitures (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parcel_id      UUID NOT NULL REFERENCES parcels(id) ON DELETE CASCADE,
    case_hash      TEXT NOT NULL,              -- hex(32) of the court order
    from_owner     TEXT NOT NULL,              -- base58 wallet being divested
    to_owner       TEXT NOT NULL,              -- base58 wallet receiving control
    threshold      SMALLINT NOT NULL,          -- validator signers required (>=2)
    present        SMALLINT NOT NULL DEFAULT 0,-- validator signers recorded
    sealed_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    status         TEXT NOT NULL DEFAULT 'recorded', -- recorded | sealed | challenged
    court_relay    TEXT NOT NULL DEFAULT ''    -- base58 relaying authority wallet
);

CREATE INDEX IF NOT EXISTS idx_forfeitures_parcel    ON forfeitures (parcel_id);
CREATE INDEX IF NOT EXISTS idx_forfeitures_case_hash ON forfeitures (case_hash);
