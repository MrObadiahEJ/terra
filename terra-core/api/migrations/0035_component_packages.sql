-- RFC-015 — Spatial Component Package storage (REQ-026).
--
-- Stores validated component packages submitted by independent reconstruction
-- workers. Each package is conformance-checked (RFC-015 §4) before insert;
-- invalid packages are rejected with 400.
--
-- geometry_artifact is a URI (off-chain model file); geometry_digest commits
-- to the canonical geometry document (terra-geom-v1 or terra-geom-v2).
-- validation_state transitions pending -> technically_valid ->
-- independently_valid -> rejected; rejected packages are retained, not deleted.

CREATE TABLE IF NOT EXISTS component_packages (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    component_id      TEXT NOT NULL UNIQUE,
    physical_entity_id TEXT NOT NULL,
    parent_id         TEXT,
    component_type    TEXT NOT NULL,
    geometry_artifact TEXT NOT NULL,
    format            TEXT NOT NULL,
    format_version    TEXT NOT NULL,
    geometry_digest   TEXT NOT NULL,
    digest_algorithm_version TEXT NOT NULL,
    crs               TEXT NOT NULL,
    elevation_ref     TEXT NOT NULL,
    vertical_units    TEXT NOT NULL,
    local_frame       JSONB,
    global_transform  JSONB,
    bounding_volume   JSONB NOT NULL,
    spatial_extent    JSONB NOT NULL,
    resolution        DOUBLE PRECISION NOT NULL,
    lod               INTEGER NOT NULL,
    source_evidence_refs TEXT[] NOT NULL DEFAULT '{}',
    reconstruction_engine_id TEXT NOT NULL,
    reconstruction_engine_version TEXT NOT NULL,
    processing_configuration JSONB,
    creation_timestamp  TIMESTAMPTZ NOT NULL,
    resource_usage_receipt JSONB,
    accuracy            DOUBLE PRECISION,
    uncertainty         DOUBLE PRECISION,
    coverage            JSONB,
    confidence          DOUBLE PRECISION,
    provenance_classification TEXT NOT NULL,
    dependencies        TEXT[] NOT NULL DEFAULT '{}',
    overlap_ids         TEXT[] NOT NULL DEFAULT '{}',
    privacy_classification TEXT NOT NULL,
    access_policy       JSONB NOT NULL,
    validation_state    TEXT NOT NULL DEFAULT 'pending',
    version_lineage     JSONB NOT NULL DEFAULT '{}',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT component_packages_provenance_check
        CHECK (provenance_classification IN ('imported', 'observed', 'surveyed', 'reconstructed', 'inferred', 'manually_corrected')),
    CONSTRAINT component_packages_validation_state_check
        CHECK (validation_state IN ('pending', 'technically_valid', 'independently_valid', 'rejected')),
    CONSTRAINT component_packages_privacy_check
        CHECK (privacy_classification IN ('public', 'restricted', 'private')),
    CONSTRAINT component_packages_vertical_units_check
        CHECK (vertical_units IN ('m', 'ft')),
    CONSTRAINT component_packages_elevation_ref_check
        CHECK (elevation_ref IN ('EGM2008', 'WGS84-ELLIPSOID', 'local', 'unknown')),
    CONSTRAINT component_packages_confidence_check
        CHECK (confidence IS NULL OR (confidence >= 0 AND confidence <= 1)),
    CONSTRAINT component_packages_crs_check
        CHECK (crs = 'EPSG:4326' OR crs ILIKE '%crs84%' OR crs ILIKE '%epsg::4326%')
);

CREATE INDEX IF NOT EXISTS idx_component_packages_physical_entity
    ON component_packages (physical_entity_id);
CREATE INDEX IF NOT EXISTS idx_component_packages_component_type
    ON component_packages (component_type);
CREATE INDEX IF NOT EXISTS idx_component_packages_validation_state
    ON component_packages (validation_state);
CREATE INDEX IF NOT EXISTS idx_component_packages_created_at
    ON component_packages (created_at DESC);
