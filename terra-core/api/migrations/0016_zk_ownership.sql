-- RFC-011: Zero-knowledge ownership proof mirror.
-- Mirrors the on-chain ZoneSet / OwnershipRoot / NullifierRecord PDAs.
-- Only nullifier hashes, Merkle roots, and metadata live here — never
-- plaintext ownership data, parcel identifiers, or wallet addresses in proofs.

CREATE TABLE IF NOT EXISTS zone_sets (
    id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    zone_set_address     TEXT NOT NULL UNIQUE,
    zone_id              TEXT NOT NULL,
    authority            TEXT NOT NULL,
    parcel_count         INT NOT NULL DEFAULT 0,
    current_root_version INT NOT NULL DEFAULT 0,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ownership_roots (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    zone_set_id       UUID NOT NULL REFERENCES zone_sets(id) ON DELETE CASCADE,
    root_address      TEXT NOT NULL UNIQUE,
    merkle_root       TEXT NOT NULL,
    version           INT NOT NULL,
    commitment_count  INT NOT NULL DEFAULT 0,
    algorithm_id      SMALLINT NOT NULL DEFAULT 0,
    snapshot_cid      TEXT NOT NULL,
    snapshot_hash     TEXT NOT NULL,
    authority_signature TEXT NOT NULL DEFAULT '',
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (zone_set_id, version)
);

CREATE TABLE IF NOT EXISTS nullifier_records (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nullifier_hash  TEXT NOT NULL UNIQUE,
    zone_set_id     UUID NOT NULL REFERENCES zone_sets(id) ON DELETE CASCADE,
    root_version    INT NOT NULL,
    prover          TEXT NOT NULL,
    proof_purpose   TEXT NOT NULL,
    disclosure_type SMALLINT NOT NULL DEFAULT 0,
    block_time      TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_zone_sets_authority ON zone_sets (authority);
CREATE INDEX IF NOT EXISTS idx_ownership_roots_zone ON ownership_roots (zone_set_id);
CREATE INDEX IF NOT EXISTS idx_nullifier_records_zone ON nullifier_records (zone_set_id);
CREATE INDEX IF NOT EXISTS idx_nullifier_records_prover ON nullifier_records (prover);
CREATE INDEX IF NOT EXISTS idx_nullifier_records_purpose ON nullifier_records (proof_purpose);
