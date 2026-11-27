-- 0059_parcel_source_geometry.sql
--
-- Forward fix for finding 1e: the client anchors SHA-256 over the exact JSON
-- ring it drew (`JSON.stringify(ring)`), but the API only kept the PostGIS
-- polygon. `ST_AsGeoJSON` cannot reproduce those bytes (number formatting,
-- metadata and ordering differ), so the indexer had no honest way to check
-- an on-chain `geometry_hash` against the stored polygon.
--
-- Keep the submitted GeoJSON verbatim. The indexer now derives the ring from
-- it and recomputes the SHA-256 the client anchored. Existing rows have no
-- source bytes and therefore stay unverified until re-registered — which is
-- the honest outcome, not a fabricated verification.

ALTER TABLE parcels ADD COLUMN IF NOT EXISTS source_geometry TEXT;

-- The source bytes affect verification, so include them in the mirror hash
-- and recompute existing hashes (the trigger fires on every UPDATE).
CREATE OR REPLACE FUNCTION compute_parcel_mirror_hash()
RETURNS trigger AS $$
DECLARE
    h text;
BEGIN
    SELECT encode(digest(
        coalesce(NEW.name, '') || '|' ||
        coalesce(NEW.status, '') || '|' ||
        coalesce(ST_AsEWKB(NEW.geometry), '') || '|' ||
        coalesce(NEW.source_geometry, '') || '|' ||
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

UPDATE parcels SET source_geometry = source_geometry;
