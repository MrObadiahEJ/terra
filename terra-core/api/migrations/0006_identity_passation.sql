-- Phase 1/3 — Identity + wallet passation mirror.
-- Binds a person (via a hashed credential) to the wallet they hold (the
-- provisioned, person-held key), records a recovery wallet, and tracks the
-- time-boxed succession (passation) that lets control pass to an heir, a
-- recovery account, or a deliberate transferee — and lets a dead validator's
-- slot be rotated.

-- On-chain Identity account mirror (PDA: ["identity", identity_hash]).
CREATE TABLE IF NOT EXISTS identities (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    identity_hash TEXT NOT NULL,          -- hex(32) sha256 over the person's credential
    owner         TEXT NOT NULL,          -- base58 wallet the person holds (active key)
    recovery      TEXT NOT NULL DEFAULT '', -- base58 backup wallet, '' = none
    parcel_count  SMALLINT NOT NULL DEFAULT 0,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (identity_hash)
);

-- Optional human metadata for an identity (name, docs reference). The on-chain
-- hash is the chain of trust; this is display/index only and never trusted on
-- its own.
CREATE TABLE IF NOT EXISTS identity_metadata (
    identity_id   UUID PRIMARY KEY REFERENCES identities(id) ON DELETE CASCADE,
    display_name  TEXT NOT NULL,
    national_id   TEXT,                   -- raw national id if the person consents to store it
    phone         TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- On-chain Succession account mirror (PDA: ["succession", identity, successor]).
-- kind: 0=successor(heir), 1=recovery, 2=transfer.
CREATE TABLE IF NOT EXISTS successions (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    identity_id  UUID NOT NULL REFERENCES identities(id) ON DELETE CASCADE,
    identity_hash TEXT NOT NULL,
    kind         SMALLINT NOT NULL,
    successor     TEXT NOT NULL,          -- base58 wallet gaining control
    requested_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    effective_at  TIMESTAMPTZ NOT NULL,   -- requested_at + grace period
    claimed_at    TIMESTAMPTZ,
    cancelled_at  TIMESTAMPTZ,
    status        TEXT NOT NULL DEFAULT 'pending', -- pending | effective | cancelled | claimed
    UNIQUE (identity_id, successor)
);

CREATE INDEX IF NOT EXISTS idx_identities_owner     ON identities (owner);
CREATE INDEX IF NOT EXISTS idx_successions_identity ON successions (identity_id);
CREATE INDEX IF NOT EXISTS idx_successions_successor ON successions (successor);

-- Attestation validator-rotation tracking (mirrors Attestation.version + the
-- history of rotate_validators calls, so a reconstituted set is auditable).
ALTER TABLE attestations
    ADD COLUMN IF NOT EXISTS version SMALLINT NOT NULL DEFAULT 0;

CREATE TABLE IF NOT EXISTS validator_rotations (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    attestation_id UUID NOT NULL REFERENCES attestations(id) ON DELETE CASCADE,
    version        SMALLINT NOT NULL,
    required       SMALLINT NOT NULL,
    validators     TEXT[] NOT NULL DEFAULT '{}',
    rotated_by     TEXT NOT NULL,          -- base58 wallet that authorized the rotation
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_rotations_attestation ON validator_rotations (attestation_id);

