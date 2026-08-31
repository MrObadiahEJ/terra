-- Phase 1/3 — Multi-validator attestation + document binding mirror.
-- Heavy off-chain data is linked to the on-chain terra_registry record by a
-- content-hash anchor and a set of validator identities/signatures.

-- On-chain Attestation account mirror (PDA: ["attestation", parcel, specifier]).
CREATE TABLE IF NOT EXISTS attestations (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parcel_id     UUID NOT NULL REFERENCES parcels(id) ON DELETE CASCADE,
    onchain_id    TEXT NOT NULL,           -- hex(32) parcel PDA seed
    specifier     TEXT NOT NULL,           -- hex(32)
    content_hash  TEXT NOT NULL,           -- hex(32) sha256 over the payload
    required      SMALLINT NOT NULL,       -- signature threshold
    count         SMALLINT NOT NULL,       -- number of validators registered
    validators    TEXT[] NOT NULL DEFAULT '{}',  -- base58 wallet list (on-chain anchor)
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (parcel_id, specifier)
);

-- A wrapper document/artifact bound to a parcel.
CREATE TABLE IF NOT EXISTS documents (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parcel_id    UUID NOT NULL REFERENCES parcels(id) ON DELETE CASCADE,
    title        TEXT NOT NULL,
    category     TEXT NOT NULL,            -- deed, survey, contract, notarization, ...
    content_hash TEXT NOT NULL,            -- hex(32) sha256 over the stored file/data
    storage_ref  TEXT NOT NULL,            -- object key / path in off-chain store
    owner        TEXT NOT NULL,            -- wallet base58 that owns the document
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Per-validator signature over the attested payload.
-- Each row is an Ed25519 signature by `validator`'s key over the fixed
-- message `content_hash || onchain_id` (both hex). Verification is done against
-- the validator's public key and the anchored attestation's validator set.
CREATE TABLE IF NOT EXISTS validations (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    attestation_id UUID NOT NULL REFERENCES attestations(id) ON DELETE CASCADE,
    validator    TEXT NOT NULL,            -- wallet base58
    signature    TEXT NOT NULL,            -- hex(64) Ed25519 signature
    valid        BOOLEAN NOT NULL DEFAULT FALSE,  -- recomputed by /verify
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (attestation_id, validator)
);

CREATE INDEX IF NOT EXISTS idx_attestations_parcel ON attestations (parcel_id);
CREATE INDEX IF NOT EXISTS idx_documents_parcel   ON documents (parcel_id);
CREATE INDEX IF NOT EXISTS idx_documents_owner    ON documents (owner);
CREATE INDEX IF NOT EXISTS idx_validations_validator ON validations (validator);
