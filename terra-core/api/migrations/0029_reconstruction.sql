-- 3D reconstruction scaffolding (RFC-013 Phase C).
--
-- Capture sessions are on-the-ground photo/video surveys of a parcel.
-- Reconstruction jobs turn a capture session into a 3D model (photogrammetry
-- pipeline — the worker itself is a later increment; this migration reserves
-- the durable job queue). Three-d models are the resulting tilesets/meshes.
--
-- Per Design Rule 10, large payloads stay off-chain (object storage / IPFS);
-- only the URI + content hash live here, and the hash can be anchored on-chain
-- through the EvidenceManifest artifacts (RFC-012 Ph.5).

CREATE TABLE IF NOT EXISTS capture_sessions (
    id             VARCHAR(64) PRIMARY KEY,
    parcel_id      VARCHAR(64),
    title          TEXT,
    captured_by    VARCHAR(128),
    captured_at    TIMESTAMPTZ,
    media_count    INTEGER NOT NULL DEFAULT 0,
    evidence_refs  TEXT[] NOT NULL DEFAULT '{}',
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_capture_sessions_parcel
    ON capture_sessions (parcel_id, captured_at DESC)
    WHERE parcel_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS reconstruction_jobs (
    id                 VARCHAR(64) PRIMARY KEY,
    capture_session_id VARCHAR(64) NOT NULL REFERENCES capture_sessions (id) ON DELETE CASCADE,
    status             TEXT NOT NULL DEFAULT 'queued',
    progress           REAL NOT NULL DEFAULT 0,
    error              TEXT,
    artifact_uri       TEXT,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_reconstruction_jobs_session
    ON reconstruction_jobs (capture_session_id, created_at DESC);

CREATE TABLE IF NOT EXISTS three_d_models (
    id                   VARCHAR(64) PRIMARY KEY,
    parcel_id            VARCHAR(64),
    reconstruction_job_id VARCHAR(64) REFERENCES reconstruction_jobs (id) ON DELETE SET NULL,
    uri                  TEXT NOT NULL,
    format               TEXT,
    content_hash         VARCHAR(128),
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_three_d_models_parcel
    ON three_d_models (parcel_id, created_at DESC)
    WHERE parcel_id IS NOT NULL;
