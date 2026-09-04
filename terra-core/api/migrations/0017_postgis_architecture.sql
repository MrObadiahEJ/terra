-- PostGIS spatial architecture consolidation.
--
-- What exists already:
--   0001 parcels(id, name, owner, status, geometry Polygon 4326 + GIST index)
--   0002 roads / pois / pilot_zones / photogrammetry_assets (+ GIST indexes)
--   0004 parcels on-chain mirror columns (onchain_id, geometry_hash, flags...)
--
-- This migration closes the remaining gaps:
--   1. Maintained parcel centroids (Point 4326) for fast near/radius queries.
--   2. Geometry write-guard: only valid Polygon/MultiPolygon in EPSG:4326.
--   3. Missing lookup indexes (status, centroid).
--   4. Canonical spatial views used by the API:
--        parcel_spatial_stats  (area, centroid, bbox per parcel)
--        zone_parcel_counts    (parcel count + total area per pilot zone)

CREATE EXTENSION IF NOT EXISTS postgis;

-- === 1. Maintained centroids ===
ALTER TABLE parcels
    ADD COLUMN IF NOT EXISTS centroid geometry(Point, 4326);

UPDATE parcels
SET centroid = ST_Centroid(geometry)
WHERE centroid IS NULL AND geometry IS NOT NULL;

CREATE OR REPLACE FUNCTION maintain_parcel_centroid()
RETURNS trigger AS $$
BEGIN
    NEW.centroid := ST_Centroid(NEW.geometry);
    NEW.updated_at := now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_parcels_centroid ON parcels;
CREATE TRIGGER trg_parcels_centroid
    BEFORE INSERT OR UPDATE OF geometry ON parcels
    FOR EACH ROW EXECUTE FUNCTION maintain_parcel_centroid();

-- === 2. Geometry write-guard (new writes only; never breaks existing rows) ===
CREATE OR REPLACE FUNCTION guard_parcel_geometry()
RETURNS trigger AS $$
BEGIN
    IF NEW.geometry IS NULL THEN
        RAISE EXCEPTION 'parcel geometry is required';
    END IF;
    IF ST_SRID(NEW.geometry) != 4326 THEN
        RAISE EXCEPTION 'parcel geometry must use SRID 4326 (EPSG:4326)';
    END IF;
    IF ST_GeometryType(NEW.geometry) NOT IN ('ST_Polygon', 'ST_MultiPolygon') THEN
        RAISE EXCEPTION 'parcel geometry must be a Polygon or MultiPolygon';
    END IF;
    IF NOT ST_IsValid(NEW.geometry) THEN
        RAISE EXCEPTION 'parcel geometry is not valid: %', ST_IsValidReason(NEW.geometry);
    END IF;
    IF ST_Area(NEW.geometry::geography) <= 0 THEN
        RAISE EXCEPTION 'parcel geometry must have positive area';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_parcels_geometry_guard ON parcels;
CREATE TRIGGER trg_parcels_geometry_guard
    BEFORE INSERT OR UPDATE OF geometry ON parcels
    FOR EACH ROW EXECUTE FUNCTION guard_parcel_geometry();

-- === 3. Lookup indexes ===
CREATE INDEX IF NOT EXISTS idx_parcels_status ON parcels (status);
CREATE INDEX IF NOT EXISTS idx_parcels_centroid ON parcels USING GIST (centroid);
CREATE INDEX IF NOT EXISTS idx_parcels_onchain_status ON parcels (status) WHERE onchain_id IS NOT NULL;

-- === 4. Canonical spatial views ===
CREATE OR REPLACE VIEW parcel_spatial_stats AS
SELECT
    p.id,
    p.name,
    p.owner,
    p.status,
    p.onchain_id,
    ST_Area(p.geometry::geography)::float8 AS area_m2,
    ST_AsGeoJSON(p.geometry)::text AS geometry,
    ST_AsGeoJSON(p.centroid)::text AS centroid,
    ST_XMin(p.geometry) AS minx,
    ST_YMin(p.geometry) AS miny,
    ST_XMax(p.geometry) AS maxx,
    ST_YMax(p.geometry) AS maxy
FROM parcels p;

CREATE OR REPLACE VIEW zone_parcel_counts AS
SELECT
    z.id AS zone_id,
    z.name AS zone_name,
    count(p.id)::bigint AS parcel_count,
    coalesce(sum(ST_Area(p.geometry::geography)), 0)::float8 AS total_area_m2
FROM pilot_zones z
LEFT JOIN parcels p ON ST_Intersects(p.geometry, z.geometry)
GROUP BY z.id, z.name;

COMMENT ON VIEW parcel_spatial_stats IS 'Canonical per-parcel spatial facts (area, centroid, bbox). Single source for API spatial responses.';
COMMENT ON VIEW zone_parcel_counts IS 'Parcel count + total area per pilot zone via ST_Intersects. Basis for zone stats endpoints.';
