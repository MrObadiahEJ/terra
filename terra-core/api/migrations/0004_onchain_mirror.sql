-- Phase 1/3 — On-chain state mirror.
-- Reconciles PostGIS state with the terra_registry program account model.

-- Extend parcels with the on-chain Parcel fields.
ALTER TABLE parcels
    ADD COLUMN IF NOT EXISTS onchain_id  TEXT,
    ADD COLUMN IF NOT EXISTS geometry_hash TEXT,
    ADD COLUMN IF NOT EXISTS infrastructure_flags SMALLINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS rights_count SMALLINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS access_hash TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_parcels_onchain_id ON parcels (onchain_id) WHERE onchain_id IS NOT NULL;

-- Mirror of the on-chain Rights account (PDA: ["rights", parcel, nonce]).
CREATE TABLE IF NOT EXISTS rights (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parcel_id    UUID NOT NULL REFERENCES parcels(id) ON DELETE CASCADE,
    rights_kind  SMALLINT NOT NULL,
    holder       TEXT NOT NULL,
    granter      TEXT NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at   TIMESTAMPTZ,
    notes        TEXT NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_rights_parcel ON rights (parcel_id);
CREATE INDEX IF NOT EXISTS idx_rights_holder ON rights (holder);
