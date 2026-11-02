-- RFC-012 / Chapter 12 — World Representation Manifest (REQ-027).
--
-- A logical, versioned reference structure that coordinates independently
-- stored spatial objects. It stores references and relationships, not
-- geometry. Each manifest represents a geographic region/world partition and
-- identifies the objects included, their current component versions, assembly
-- versions, storage references, LODs, validation states, and relationships
-- to parent/neighboring manifests.

CREATE TABLE IF NOT EXISTS world_manifests (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    region_id         TEXT NOT NULL,             -- geographic partition id
    region_name       TEXT NOT NULL,
    version           INTEGER NOT NULL DEFAULT 1,
    parent_manifest_id UUID,                      -- hierarchical parent
    neighbor_manifest_ids UUID[] NOT NULL DEFAULT '{}', -- adjacent regions
    description       TEXT,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT world_manifests_region_version_unique
        UNIQUE (region_id, version)
);

-- One row per spatial object included in a manifest version.
-- storage_reference points at the component package / asset in the data plane.
CREATE TABLE IF NOT EXISTS manifest_objects (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    manifest_id        UUID NOT NULL REFERENCES world_manifests(id) ON DELETE CASCADE,
    object_id          TEXT NOT NULL,             -- stable spatial object id
    component_version_id TEXT NOT NULL,           -- current component version
    assembly_version_id  TEXT,                    -- current assembly version
    spatial_extent     JSONB NOT NULL,            -- GeoGeometry bounding extent
    storage_reference  TEXT NOT NULL,             -- data-plane retrieval ref
    available_lods     INTEGER[] NOT NULL DEFAULT '{0}', -- LODs available
    validation_state   TEXT NOT NULL DEFAULT 'pending',
    access_requirements JSONB NOT NULL DEFAULT '{}', -- privacy/access rules
    update_history     JSONB NOT NULL DEFAULT '[]',   -- version transition log
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT manifest_objects_validation_state_check
        CHECK (validation_state IN ('pending', 'technically_valid', 'independently_valid', 'rejected')),
    CONSTRAINT manifest_objects_unique_per_manifest
        UNIQUE (manifest_id, object_id)
);

CREATE INDEX IF NOT EXISTS idx_world_manifests_region
    ON world_manifests (region_id);
CREATE INDEX IF NOT EXISTS idx_world_manifests_parent
    ON world_manifests (parent_manifest_id);
CREATE INDEX IF NOT EXISTS idx_manifest_objects_manifest
    ON manifest_objects (manifest_id);
CREATE INDEX IF NOT EXISTS idx_manifest_objects_object
    ON manifest_objects (object_id);
CREATE INDEX IF NOT EXISTS idx_manifest_objects_validation
    ON manifest_objects (validation_state);
