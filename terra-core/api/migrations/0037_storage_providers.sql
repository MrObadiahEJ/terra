-- RFC-012 / Chapter 12 — Storage provider discovery (REQ-028).
--
-- Storage providers hold Spatial Component Packages and other data-plane
-- assets. The runtime discovers providers, selects the best replica for each
-- component (version availability, authorization, latency, cost, reliability,
-- data-locality, privacy), and falls back when the first provider fails.

CREATE TABLE IF NOT EXISTS storage_providers (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id       TEXT NOT NULL UNIQUE,
    name              TEXT NOT NULL,
    provider_type     TEXT NOT NULL,             -- object_storage, cdn, ipfs, cloud, p2p
    base_url          TEXT NOT NULL,
    capabilities      JSONB NOT NULL DEFAULT '{}', -- supported formats, protocols, regions
    geographic_scope  TEXT[],                    -- regions this provider covers
    is_active         BOOLEAN NOT NULL DEFAULT true,
    reliability_score DOUBLE PRECISION NOT NULL DEFAULT 1.0, -- 0..1
    cost_per_gb       DOUBLE PRECISION,          -- USD/GB, for cost-aware selection
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT storage_providers_reliability_check
        CHECK (reliability_score >= 0 AND reliability_score <= 1),
    CONSTRAINT storage_providers_type_check
        CHECK (provider_type IN ('object_storage', 'cdn', 'ipfs', 'commercial_cloud', 'p2p'))
);

-- Replicas of an asset on a specific provider.
CREATE TABLE IF NOT EXISTS provider_replicas (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id       UUID NOT NULL REFERENCES storage_providers(id) ON DELETE CASCADE,
    component_id      TEXT NOT NULL,             -- component id (from component_packages)
    component_version TEXT NOT NULL,             -- component version
    storage_reference TEXT NOT NULL,             -- provider-specific object ref
    version           TEXT NOT NULL,             -- asset version
    size_bytes        BIGINT,
    checksum          TEXT,                      -- sha256 of the stored bytes
    latency_ms        INTEGER,                   -- observed last-mile latency
    is_available      BOOLEAN NOT NULL DEFAULT true,
    last_checked_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT provider_replicas_unique
        UNIQUE (provider_id, component_id, component_version)
);

CREATE INDEX IF NOT EXISTS idx_storage_providers_active
    ON storage_providers (is_active);
CREATE INDEX IF NOT EXISTS idx_storage_providers_type
    ON storage_providers (provider_type);
CREATE INDEX IF NOT EXISTS idx_provider_replicas_provider
    ON provider_replicas (provider_id);
CREATE INDEX IF NOT EXISTS idx_provider_replicas_component
    ON provider_replicas (component_id, component_version);
CREATE INDEX IF NOT EXISTS idx_provider_replicas_available
    ON provider_replicas (is_available);
