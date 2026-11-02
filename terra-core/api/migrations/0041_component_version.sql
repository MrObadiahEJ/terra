-- Stage 5: Add component_version to component_packages so providers can be
-- joined by (component_id, component_version) and the global_transform can be
-- returned to the client for proper 3D model placement.

ALTER TABLE component_packages
    ADD COLUMN IF NOT EXISTS component_version TEXT NOT NULL DEFAULT 'v1';

-- Create a composite index for joining with provider_replicas.
CREATE INDEX IF NOT EXISTS idx_component_packages_id_version
    ON component_packages (component_id, component_version);
