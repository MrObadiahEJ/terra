-- 0061_parcel_geometry_artifacts.sql
--
-- Q-035(a): content-addressed geometry artifacts.
--
-- The on-chain instruction `register_parcel_geometry_artifact` writes a new PDA
-- keyed by [parcel, sha256(raw GeoJSON bytes)]. The `Parcel` Borsh layout is
-- deliberately unchanged: it still carries only `geometry_hash` (the client's
-- exterior-ring digest). This migration adds:
--
--   * `parcel_geometry_artifacts`: typed chain mirror of that new PDA, with
--     ADR-026 transaction provenance and a mirror drift hash;
--   * `parcel_geometry_uploads`: API-side bytes actually stored under the
--     declared content hash, including the ring hash re-derived from the
--     bytes. Indexer materialization only accepts a chain-only parcel when the
--     artifact is chain-verified, sha256(bytes) == content_hash, the ring
--     digest == Parcel.geometry_hash, and the on-chain URI == stored ref.

CREATE TABLE IF NOT EXISTS parcel_geometry_artifacts (
    pubkey TEXT PRIMARY KEY,
    parcel_pubkey TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    storage_uri TEXT NOT NULL,
    registrant TEXT NOT NULL,
    chain_created_at TIMESTAMPTZ,
    mirror_hash TEXT,
    chain_verified BOOLEAN NOT NULL DEFAULT FALSE,
    chain_slot BIGINT,
    chain_signature TEXT,
    mirrored_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_pga_parcel
    ON parcel_geometry_artifacts (parcel_pubkey);
CREATE INDEX IF NOT EXISTS idx_pga_content_hash
    ON parcel_geometry_artifacts (content_hash);
CREATE INDEX IF NOT EXISTS idx_pga_unverified
    ON parcel_geometry_artifacts (chain_verified) WHERE chain_verified = FALSE;
CREATE INDEX IF NOT EXISTS idx_pga_mirror_hash
    ON parcel_geometry_artifacts (mirror_hash) WHERE mirror_hash IS NOT NULL;

CREATE OR REPLACE FUNCTION compute_parcel_geometry_artifact_mirror_hash()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    NEW.mirror_hash := encode(digest(
        NEW.pubkey || '|' ||
        NEW.parcel_pubkey || '|' ||
        NEW.content_hash || '|' ||
        NEW.storage_uri || '|' ||
        NEW.registrant || '|' ||
        coalesce(to_char(NEW.chain_created_at, 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'), ''),
        'sha256'), 'hex');
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_parcel_geometry_artifact_mirror_hash
    ON parcel_geometry_artifacts;
CREATE TRIGGER trg_parcel_geometry_artifact_mirror_hash
    BEFORE INSERT OR UPDATE ON parcel_geometry_artifacts
    FOR EACH ROW EXECUTE FUNCTION compute_parcel_geometry_artifact_mirror_hash();

-- Bytes uploaded to the API. Keyed by (parcel PDA, content hash) so a caller
-- cannot overwrite a verified artifact with different bytes under the same
-- digest. The ring hash makes the Parcel.geometry_hash check possible without
-- re-reading PostGIS-normalized geometry.
CREATE TABLE IF NOT EXISTS parcel_geometry_uploads (
    parcel_pubkey TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    geometry_hash TEXT NOT NULL,
    storage_ref TEXT NOT NULL,
    storage_backend TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (parcel_pubkey, content_hash)
);

CREATE INDEX IF NOT EXISTS idx_pgu_storage_ref
    ON parcel_geometry_uploads (storage_ref);
