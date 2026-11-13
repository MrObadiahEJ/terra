-- 0044_signed_manifests.sql
--
-- Immutable storage for signed `terra.manifest/v1` envelopes (ADR-020).
-- The primary identity of a manifest is the SHA-256 of its canonical payload,
-- so the same payload can be published by more than one signer. Envelopes are
-- stored verbatim (re-serialized JSON) and re-verified on retrieval.

CREATE TABLE IF NOT EXISTS signed_manifests (
    content_address TEXT NOT NULL,
    publisher TEXT NOT NULL,
    region_id TEXT NOT NULL,
    region_name TEXT NOT NULL,
    version BIGINT NOT NULL CHECK (version > 0),
    region_cell TEXT NOT NULL,
    signature TEXT NOT NULL,
    envelope JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (content_address, publisher)
);

CREATE INDEX IF NOT EXISTS signed_manifests_region_version_idx
    ON signed_manifests (region_id, version DESC);

CREATE INDEX IF NOT EXISTS signed_manifests_cell_idx
    ON signed_manifests (region_cell);
