-- Chapter 12 — Distributed Runtime (REQ-028 dependent).
--
-- The runtime resolves what data is needed for a view, discovers where it is
-- stored, retrieves it, manages versions, and supplies consistent information
-- to visualization clients. Scene graphs are resolved from manifests and
-- spatial queries; assets are retrieved from providers with fallback.

CREATE TABLE IF NOT EXISTS scene_graphs (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    manifest_id       UUID NOT NULL REFERENCES world_manifests(id) ON DELETE CASCADE,
    region_id         TEXT NOT NULL,
    lod_level         INTEGER NOT NULL DEFAULT 0,
    node_count        INTEGER NOT NULL DEFAULT 0,
    status            TEXT NOT NULL DEFAULT 'ready', -- ready, partial, incomplete
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT scene_graphs_status_check
        CHECK (status IN ('ready', 'partial', 'incomplete')),
    CONSTRAINT scene_graphs_lod_check
        CHECK (lod_level >= 0 AND lod_level <= 4)
);

-- Nodes in a resolved scene graph. Each node references a component version
-- and its placement in the assembled scene.
CREATE TABLE IF NOT EXISTS scene_nodes (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scene_graph_id    UUID NOT NULL REFERENCES scene_graphs(id) ON DELETE CASCADE,
    object_id         TEXT NOT NULL,
    component_version_id TEXT NOT NULL,
    parent_node_id    UUID, -- references scene_nodes.id for parent-child
    transform         JSONB, -- local-to-global transform for this node
    lod_level         INTEGER NOT NULL DEFAULT 0,
    visibility        TEXT NOT NULL DEFAULT 'visible', -- visible, culled, lod_pending
    loading_state     TEXT NOT NULL DEFAULT 'pending', -- pending, loaded, failed, cached
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT scene_nodes_visibility_check
        CHECK (visibility IN ('visible', 'culled', 'lod_pending')),
    CONSTRAINT scene_nodes_loading_check
        CHECK (loading_state IN ('pending', 'loaded', 'failed', 'cached'))
);

-- Asset cache: tracks which assets are cached locally/shared and their state.
CREATE TABLE IF NOT EXISTS asset_cache (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    component_id      TEXT NOT NULL,
    component_version TEXT NOT NULL,
    storage_reference TEXT NOT NULL,
    cache_path        TEXT NOT NULL,
    size_bytes        BIGINT,
    cached_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_accessed_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    access_count      INTEGER NOT NULL DEFAULT 0,

    CONSTRAINT asset_cache_unique
        UNIQUE (component_id, component_version)
);

CREATE INDEX IF NOT EXISTS idx_scene_graphs_manifest
    ON scene_graphs (manifest_id);
CREATE INDEX IF NOT EXISTS idx_scene_graphs_region
    ON scene_graphs (region_id);
CREATE INDEX IF NOT EXISTS idx_scene_graphs_lod
    ON scene_graphs (lod_level);
CREATE INDEX IF NOT EXISTS idx_scene_nodes_graph
    ON scene_nodes (scene_graph_id);
CREATE INDEX IF NOT EXISTS idx_scene_nodes_object
    ON scene_nodes (object_id);
CREATE INDEX IF NOT EXISTS idx_scene_nodes_parent
    ON scene_nodes (parent_node_id);
CREATE INDEX IF NOT EXISTS idx_asset_cache_component
    ON asset_cache (component_id, component_version);
