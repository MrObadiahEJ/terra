-- Chapter 11 — Spatial Assembly (REQ-026 dependent).
--
-- Assembly connects accepted, independently produced components into coherent
-- representations of buildings, parcels, cities. An assembly records which
-- component versions compose a spatial object, their relationships
-- (parent-child, containment, adjacency, connection, dependency), the
-- registration transforms between them, and any conflicts that must be
-- resolved before the assembly is accepted.

CREATE TABLE IF NOT EXISTS assemblies (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assembly_id       TEXT NOT NULL UNIQUE,
    name              TEXT NOT NULL,
    object_type       TEXT NOT NULL,               -- building, parcel, district, terrain, infrastructure
    root_object_id    TEXT NOT NULL,               -- stable id of the assembled physical object
    version           INTEGER NOT NULL DEFAULT 1,
    status            TEXT NOT NULL DEFAULT 'pending', -- pending, valid, complete, rejected
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT assemblies_status_check
        CHECK (status IN ('pending', 'valid', 'complete', 'rejected')),
    CONSTRAINT assemblies_object_type_check
        CHECK (object_type IN ('building', 'parcel', 'district', 'terrain', 'infrastructure', 'interior')),
    CONSTRAINT assemblies_assembly_version_unique
        UNIQUE (root_object_id, version)
);

-- Component-to-assembly relationships. Each row records how one component
-- version participates in an assembly version, its relationship to the
-- parent object, and its registration transform (local-to-global).
CREATE TABLE IF NOT EXISTS assembly_objects (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assembly_id        UUID NOT NULL REFERENCES assemblies(id) ON DELETE CASCADE,
    object_id          TEXT NOT NULL,             -- stable id of the physical sub-object
    component_version_id TEXT NOT NULL,           -- the component version used
    relationship_type  TEXT NOT NULL,             -- parent_child, containment, adjacency, connection, dependency
    transform          JSONB,                     -- local-to-global transform matrix
    position_in_assembly JSONB,                   -- placement hint (e.g. floor, facade side)
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT assembly_objects_relationship_check
        CHECK (relationship_type IN ('parent_child', 'containment', 'adjacency', 'connection', 'dependency')),
    CONSTRAINT assembly_objects_unique_per_assembly
        UNIQUE (assembly_id, object_id)
);

-- Conflict detection results. When two components cannot safely be merged,
-- both candidates are preserved and a conflict is recorded for resolution.
CREATE TABLE IF NOT EXISTS assembly_conflicts (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assembly_id    UUID NOT NULL REFERENCES assemblies(id) ON DELETE CASCADE,
    object_a_id    TEXT NOT NULL,
    object_b_id    TEXT NOT NULL,
    conflict_type  TEXT NOT NULL,                 -- duplicate_geometry, boundary_mismatch, intersection, terrain_inconsistency, elevation_datum
    severity       TEXT NOT NULL DEFAULT 'medium', -- low, medium, high, critical
    status         TEXT NOT NULL DEFAULT 'open',  -- open, resolved_by_merge, resolved_by_split, rejected
    description    TEXT,
    resolution     TEXT,                          -- how the conflict was resolved
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at    TIMESTAMPTZ,

    CONSTRAINT assembly_conflicts_type_check
        CHECK (conflict_type IN ('duplicate_geometry', 'boundary_mismatch', 'intersection', 'terrain_inconsistency', 'elevation_datum')),
    CONSTRAINT assembly_conflicts_severity_check
        CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    CONSTRAINT assembly_conflicts_status_check
        CHECK (status IN ('open', 'resolved_by_merge', 'resolved_by_split', 'rejected')),
    CONSTRAINT assembly_conflicts_distinct_objects
        CHECK (object_a_id <> object_b_id)
);

CREATE INDEX IF NOT EXISTS idx_assemblies_root_object
    ON assemblies (root_object_id);
CREATE INDEX IF NOT EXISTS idx_assemblies_status
    ON assemblies (status);
CREATE INDEX IF NOT EXISTS idx_assembly_objects_assembly
    ON assembly_objects (assembly_id);
CREATE INDEX IF NOT EXISTS idx_assembly_objects_object
    ON assembly_objects (object_id);
CREATE INDEX IF NOT EXISTS idx_assembly_objects_component
    ON assembly_objects (component_version_id);
CREATE INDEX IF NOT EXISTS idx_assembly_conflicts_assembly
    ON assembly_conflicts (assembly_id);
CREATE INDEX IF NOT EXISTS idx_assembly_conflicts_status
    ON assembly_conflicts (status);
