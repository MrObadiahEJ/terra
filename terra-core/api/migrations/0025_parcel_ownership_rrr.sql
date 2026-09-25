-- RFC-012 RRR: ownership is a canonical right, not a parcel column.
--
-- The on-chain `Parcel` account no longer carries an `owner` field; protocol
-- ownership lives in the ownership `Rights` PDA (seeds ["ownership", parcel]).
-- This migration mirrors that model off-chain:
--   * new table `parcel_ownership` (one row per parcel, holder = current wallet)
--   * backfill from the legacy `parcels.owner` column
--   * drop `parcels.owner` and its index
--   * rebuild `parcel_spatial_stats` to source the holder from the new table

CREATE TABLE parcel_ownership (
    parcel_id  UUID PRIMARY KEY REFERENCES parcels(id) ON DELETE CASCADE,
    holder     TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO parcel_ownership (parcel_id, holder)
SELECT id, owner FROM parcels;

-- The view depends on parcels.owner and must be rebuilt around the new table.
DROP VIEW IF EXISTS parcel_spatial_stats;

ALTER TABLE parcels DROP COLUMN owner;
DROP INDEX IF EXISTS idx_parcels_owner;

CREATE INDEX idx_parcel_ownership_holder ON parcel_ownership (holder);

CREATE OR REPLACE VIEW parcel_spatial_stats AS
SELECT
    p.id,
    p.name,
    o.holder,
    p.status,
    p.onchain_id,
    ST_Area(p.geometry::geography)::float8 AS area_m2,
    ST_AsGeoJSON(p.geometry)::text AS geometry,
    ST_AsGeoJSON(p.centroid)::text AS centroid,
    ST_XMin(p.geometry) AS minx,
    ST_YMin(p.geometry) AS miny,
    ST_XMax(p.geometry) AS maxx,
    ST_YMax(p.geometry) AS maxy
FROM parcels p
JOIN parcel_ownership o ON o.parcel_id = p.id;

COMMENT ON TABLE parcel_ownership IS
    'Off-chain mirror of the on-chain ownership Rights PDA (RFC-012 RRR): holder = current wallet of the canonical ownership right.';
COMMENT ON COLUMN parcel_ownership.holder IS
    'Base58 wallet holding the parcel ownership right (may be an Identity PDA address when identity-held).';
