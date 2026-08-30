-- Phase 2 — Pilot data layer: PostGIS fusion schema.
-- Adds OSM road network, POIs, on-chain mirror tables, and photogrammetry assets.

-- === OSM roads (persisted road graph, source: OpenStreetMap) ===
CREATE TABLE IF NOT EXISTS roads (
    id          BIGINT PRIMARY KEY,
    name        TEXT,
    highway     TEXT NOT NULL,
    oneway      BOOLEAN NOT NULL DEFAULT FALSE,
    geometry    geometry(LineString, 4326) NOT NULL,
    length_m    DOUBLE PRECISION NOT NULL,
    source      TEXT NOT NULL DEFAULT 'osm',
    ingested_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_roads_geometry ON roads USING GIST (geometry);
CREATE INDEX IF NOT EXISTS idx_roads_highway ON roads (highway);

-- === OSM points of interest (persisted) ===
CREATE TABLE IF NOT EXISTS pois (
    id          BIGINT PRIMARY KEY,
    name        TEXT,
    category    TEXT NOT NULL,
    kind        TEXT NOT NULL,
    tags        JSONB NOT NULL DEFAULT '{}',
    geometry    geometry(Point, 4326) NOT NULL,
    source      TEXT NOT NULL DEFAULT 'osm',
    ingested_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_pois_geometry ON pois USING GIST (geometry);
CREATE INDEX IF NOT EXISTS idx_pois_category ON pois (category, kind);

-- === Pilot zones (e.g. Soa/Biteng survey area) ===
CREATE TABLE IF NOT EXISTS pilot_zones (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL,
    description TEXT,
    geometry    geometry(Polygon, 4326) NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_pilot_zones_geometry ON pilot_zones USING GIST (geometry);

-- === Photogrammetry assets (drone orthophotos / point clouds / surfaces) ===
CREATE TABLE IF NOT EXISTS photogrammetry_assets (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pilot_zone_id   UUID NOT NULL REFERENCES pilot_zones(id) ON DELETE CASCADE,
    asset_type      TEXT NOT NULL CHECK (asset_type IN ('orthophoto', 'point_cloud', 'dsm', 'dtm', 'mesh', 'other')),
    name            TEXT NOT NULL,
    format          TEXT,
    file_path       TEXT,
    resolution_m    DOUBLE PRECISION,
    point_count     BIGINT,
    metadata        JSONB NOT NULL DEFAULT '{}',
    geometry        geometry(Geometry, 4326),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_photogrammetry_assets_zone ON photogrammetry_assets (pilot_zone_id);
CREATE INDEX IF NOT EXISTS idx_photogrammetry_assets_geometry ON photogrammetry_assets USING GIST (geometry);
