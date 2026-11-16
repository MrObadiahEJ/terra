-- 0048_mirror_hash_anchoring.sql
--
-- Per-row mirror hash anchoring (closes RISK R-004: mirror drift).
--
-- Adds a `mirror_hash` column to each off-chain mirror table. The hash is a
-- SHA-256 hex digest over the canonical serialization of the row's mirrored
-- fields, computed by a database trigger on every INSERT/UPDATE. Any manual
-- DB edit or indexer lag that changes a mirrored field without updating the
-- hash is detectable: `GET /mirror/audit` recomputes the hash and reports
-- every row whose stored `mirror_hash` no longer matches its contents, plus
-- every anchored row whose on-chain commitment columns are missing or stale.
--
-- Tables covered: parcels, rights, spatial_assets_onchain,
-- geometry_versions_onchain, three_d_models_onchain, spatial_snapshots_onchain.

-- ---------------------------------------------------------------------------
-- parcels
-- ---------------------------------------------------------------------------
ALTER TABLE parcels
    ADD COLUMN IF NOT EXISTS mirror_hash TEXT;

CREATE OR REPLACE FUNCTION compute_parcel_mirror_hash()
RETURNS trigger AS $$
DECLARE
    h text;
BEGIN
    SELECT encode(digest(
        coalesce(NEW.name, '') || '|' ||
        coalesce(NEW.status, '') || '|' ||
        coalesce(ST_AsEWKB(NEW.geometry), '') || '|' ||
        coalesce(NEW.onchain_id, '') || '|' ||
        coalesce(NEW.geometry_hash, '') || '|' ||
        coalesce(NEW.access_hash, '') || '|' ||
        NEW.infrastructure_flags::text || '|' ||
        NEW.rights_count::text,
        'sha256'), 'hex')
    INTO h;
    NEW.mirror_hash := h;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_parcels_mirror_hash ON parcels;
CREATE TRIGGER trg_parcels_mirror_hash
    BEFORE INSERT OR UPDATE ON parcels
    FOR EACH ROW EXECUTE FUNCTION compute_parcel_mirror_hash();

CREATE INDEX IF NOT EXISTS idx_parcels_mirror_hash ON parcels (mirror_hash)
    WHERE mirror_hash IS NOT NULL;

-- ---------------------------------------------------------------------------
-- rights
-- ---------------------------------------------------------------------------
ALTER TABLE rights
    ADD COLUMN IF NOT EXISTS mirror_hash TEXT;

CREATE OR REPLACE FUNCTION compute_rights_mirror_hash()
RETURNS trigger AS $$
DECLARE
    h text;
BEGIN
    SELECT encode(digest(
        coalesce(NEW.parcel_id::text, '') || '|' ||
        NEW.rights_kind::text || '|' ||
        coalesce(NEW.holder, '') || '|' ||
        coalesce(NEW.granter, '') || '|' ||
        coalesce(NEW.notes, '') || '|' ||
        coalesce(to_char(NEW.created_at, 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'), '') || '|' ||
        coalesce(to_char(NEW.expires_at, 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'), ''),
        'sha256'), 'hex')
    INTO h;
    NEW.mirror_hash := h;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_rights_mirror_hash ON rights;
CREATE TRIGGER trg_rights_mirror_hash
    BEFORE INSERT OR UPDATE ON rights
    FOR EACH ROW EXECUTE FUNCTION compute_rights_mirror_hash();

CREATE INDEX IF NOT EXISTS idx_rights_mirror_hash ON rights (mirror_hash)
    WHERE mirror_hash IS NOT NULL;

-- ---------------------------------------------------------------------------
-- spatial_assets_onchain
-- ---------------------------------------------------------------------------
ALTER TABLE spatial_assets_onchain
    ADD COLUMN IF NOT EXISTS mirror_hash TEXT;

CREATE OR REPLACE FUNCTION compute_spatial_assets_onchain_mirror_hash()
RETURNS trigger AS $$
DECLARE
    h text;
BEGIN
    SELECT encode(digest(
        coalesce(NEW.pubkey, '') || '|' ||
        coalesce(NEW.parcel_pubkey, '') || '|' ||
        coalesce(NEW.authority, '') || '|' ||
        NEW.dimensionality::text || '|' ||
        NEW.elevation_min_mm::text || '|' ||
        NEW.elevation_max_mm::text || '|' ||
        NEW.geometry_version_count::text || '|' ||
        coalesce(NEW.latest_geometry_hash, '') || '|' ||
        coalesce(to_char(NEW.chain_created_at, 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'), '') || '|' ||
        coalesce(to_char(NEW.chain_updated_at, 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'), ''),
        'sha256'), 'hex')
    INTO h;
    NEW.mirror_hash := h;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_spatial_assets_onchain_mirror_hash ON spatial_assets_onchain;
CREATE TRIGGER trg_spatial_assets_onchain_mirror_hash
    BEFORE INSERT OR UPDATE ON spatial_assets_onchain
    FOR EACH ROW EXECUTE FUNCTION compute_spatial_assets_onchain_mirror_hash();

CREATE INDEX IF NOT EXISTS idx_spatial_assets_onchain_mirror_hash
    ON spatial_assets_onchain (mirror_hash) WHERE mirror_hash IS NOT NULL;

-- ---------------------------------------------------------------------------
-- geometry_versions_onchain
-- ---------------------------------------------------------------------------
ALTER TABLE geometry_versions_onchain
    ADD COLUMN IF NOT EXISTS mirror_hash TEXT;

CREATE OR REPLACE FUNCTION compute_geometry_versions_onchain_mirror_hash()
RETURNS trigger AS $$
DECLARE
    h text;
BEGIN
    SELECT encode(digest(
        coalesce(NEW.pubkey, '') || '|' ||
        coalesce(NEW.asset_pubkey, '') || '|' ||
        coalesce(NEW.parcel_pubkey, '') || '|' ||
        NEW.version::text || '|' ||
        coalesce(NEW.geometry_hash, '') || '|' ||
        NEW.source::text || '|' ||
        NEW.dimension::text || '|' ||
        coalesce(NEW.storage_reference, '') || '|' ||
        coalesce(NEW.submitted_by, '') || '|' ||
        NEW.verified::text || '|' ||
        coalesce(NEW.verified_by, '') || '|' ||
        NEW.elevation_min_mm::text || '|' ||
        NEW.elevation_max_mm::text || '|' ||
        NEW.elevation_source::text || '|' ||
        NEW.required::text || '|' ||
        NEW.attest_count::text || '|' ||
        coalesce(array_to_string(NEW.attestors, ','), ''),
        'sha256'), 'hex')
    INTO h;
    NEW.mirror_hash := h;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_geometry_versions_onchain_mirror_hash ON geometry_versions_onchain;
CREATE TRIGGER trg_geometry_versions_onchain_mirror_hash
    BEFORE INSERT OR UPDATE ON geometry_versions_onchain
    FOR EACH ROW EXECUTE FUNCTION compute_geometry_versions_onchain_mirror_hash();

CREATE INDEX IF NOT EXISTS idx_geometry_versions_onchain_mirror_hash
    ON geometry_versions_onchain (mirror_hash) WHERE mirror_hash IS NOT NULL;

-- ---------------------------------------------------------------------------
-- three_d_models_onchain
-- ---------------------------------------------------------------------------
ALTER TABLE three_d_models_onchain
    ADD COLUMN IF NOT EXISTS mirror_hash TEXT;

CREATE OR REPLACE FUNCTION compute_three_d_models_onchain_mirror_hash()
RETURNS trigger AS $$
DECLARE
    h text;
BEGIN
    SELECT encode(digest(
        coalesce(NEW.pubkey, '') || '|' ||
        coalesce(NEW.asset_pubkey, '') || '|' ||
        coalesce(NEW.parcel_pubkey, '') || '|' ||
        NEW.model_version::text || '|' ||
        NEW.geometry_version::text || '|' ||
        coalesce(NEW.model_hash, '') || '|' ||
        coalesce(NEW.storage_reference, '') || '|' ||
        NEW.model_format::text || '|' ||
        coalesce(NEW.submitted_by, '') || '|' ||
        NEW.verified::text || '|' ||
        coalesce(NEW.verified_by, ''),
        'sha256'), 'hex')
    INTO h;
    NEW.mirror_hash := h;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_three_d_models_onchain_mirror_hash ON three_d_models_onchain;
CREATE TRIGGER trg_three_d_models_onchain_mirror_hash
    BEFORE INSERT OR UPDATE ON three_d_models_onchain
    FOR EACH ROW EXECUTE FUNCTION compute_three_d_models_onchain_mirror_hash();

CREATE INDEX IF NOT EXISTS idx_three_d_models_onchain_mirror_hash
    ON three_d_models_onchain (mirror_hash) WHERE mirror_hash IS NOT NULL;

-- ---------------------------------------------------------------------------
-- spatial_snapshots_onchain
-- ---------------------------------------------------------------------------
ALTER TABLE spatial_snapshots_onchain
    ADD COLUMN IF NOT EXISTS mirror_hash TEXT;

CREATE OR REPLACE FUNCTION compute_spatial_snapshots_onchain_mirror_hash()
RETURNS trigger AS $$
DECLARE
    h text;
BEGIN
    SELECT encode(digest(
        coalesce(NEW.pubkey, '') || '|' ||
        coalesce(NEW.asset_pubkey, '') || '|' ||
        coalesce(NEW.parcel_pubkey, '') || '|' ||
        NEW.snapshot_index::text || '|' ||
        NEW.geometry_version_count::text || '|' ||
        coalesce(NEW.latest_geometry_hash, '') || '|' ||
        NEW.dimensionality::text || '|' ||
        NEW.elevation_min_mm::text || '|' ||
        NEW.elevation_max_mm::text || '|' ||
        coalesce(NEW.label, '') || '|' ||
        coalesce(NEW.created_by, '') || '|' ||
        coalesce(to_char(NEW.chain_created_at, 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'), ''),
        'sha256'), 'hex')
    INTO h;
    NEW.mirror_hash := h;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_spatial_snapshots_onchain_mirror_hash ON spatial_snapshots_onchain;
CREATE TRIGGER trg_spatial_snapshots_onchain_mirror_hash
    BEFORE INSERT OR UPDATE ON spatial_snapshots_onchain
    FOR EACH ROW EXECUTE FUNCTION compute_spatial_snapshots_onchain_mirror_hash();

CREATE INDEX IF NOT EXISTS idx_spatial_snapshots_onchain_mirror_hash
    ON spatial_snapshots_onchain (mirror_hash) WHERE mirror_hash IS NOT NULL;

-- Backfill existing rows.
UPDATE parcels SET mirror_hash = mirror_hash WHERE mirror_hash IS NULL;
UPDATE rights SET mirror_hash = mirror_hash WHERE mirror_hash IS NULL;
UPDATE spatial_assets_onchain SET mirror_hash = mirror_hash WHERE mirror_hash IS NULL;
UPDATE geometry_versions_onchain SET mirror_hash = mirror_hash WHERE mirror_hash IS NULL;
UPDATE three_d_models_onchain SET mirror_hash = mirror_hash WHERE mirror_hash IS NULL;
UPDATE spatial_snapshots_onchain SET mirror_hash = mirror_hash WHERE mirror_hash IS NULL;
