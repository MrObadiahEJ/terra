-- Reconstruction worker pipeline (RFC-013 Phase D): job leasing + geometry drafts.
--
-- The worker leases a queued reconstruction job, runs its pipeline (photo /
-- LiDAR → geometry draft → anchor prep), and reports progress/completion.
-- A geometry draft is the durable intermediate: the GeoJSON produced by the
-- pipeline step, its SHA-256 hash (the on-chain `geometry_hash` claim), and
-- the prepared `append_geometry_version` anchor payload. Nothing here is a
-- fact until the draft's hash is anchored on-chain and verified — the same
-- observation → evidence → validation split as everywhere else.

ALTER TABLE reconstruction_jobs
    ADD COLUMN IF NOT EXISTS claimed_by        TEXT,
    ADD COLUMN IF NOT EXISTS claimed_at        TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS lease_expires_at  TIMESTAMPTZ;

-- Geometry drafts produced by the worker (intermediate between capture and
-- the on-chain GeometryVersion claim).
CREATE TABLE IF NOT EXISTS geometry_drafts (
    id                   VARCHAR(64) PRIMARY KEY,
    reconstruction_job_id VARCHAR(64) NOT NULL REFERENCES reconstruction_jobs (id) ON DELETE CASCADE,
    parcel_id            VARCHAR(64),
    method               TEXT NOT NULL,          -- 'capture-hull' | 'parcel-footprint-fallback'
    geometry_geojson     TEXT NOT NULL,          -- GeoJSON geometry object (the canonical document)
    geometry_hash        VARCHAR(64) NOT NULL,   -- SHA-256 hex of geometry_geojson bytes
    storage_reference    TEXT NOT NULL,
    source               SMALLINT NOT NULL DEFAULT 5, -- geometry_source (PHOTOGRAMMETRY)
    dimension            SMALLINT NOT NULL DEFAULT 1, -- spatial_dimension (D2_5)
    elevation_min_mm     INTEGER NOT NULL DEFAULT 0,
    elevation_max_mm     INTEGER NOT NULL DEFAULT 0,
    elevation_source     SMALLINT NOT NULL DEFAULT 0, -- elevation_source
    anchor_status        TEXT NOT NULL DEFAULT 'pending_anchor', -- pending_anchor | anchored | rejected
    anchor_signature     TEXT,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_geometry_drafts_job
    ON geometry_drafts (reconstruction_job_id);
CREATE INDEX IF NOT EXISTS idx_geometry_drafts_parcel
    ON geometry_drafts (parcel_id, created_at DESC)
    WHERE parcel_id IS NOT NULL;
