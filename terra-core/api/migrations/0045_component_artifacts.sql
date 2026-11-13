-- 0045_component_artifacts.sql
--
-- Data-plane storage for component bytes (RFC-015 `geometry_artifact`).
-- A package's artifact is immutable: it is stored once under the SHA-256 of
-- the raw bytes and served by the API with digest re-verification.
--
-- Access control stays with the component package; this table only records
-- where the bytes live. `storage_ref` is backend-internal and never exposed.

CREATE TABLE IF NOT EXISTS component_artifacts (
    component_package_id UUID PRIMARY KEY
        REFERENCES component_packages(id) ON DELETE CASCADE,
    content_address TEXT NOT NULL,
    storage_ref     TEXT NOT NULL,
    storage_backend TEXT NOT NULL,
    content_type    TEXT NOT NULL,
    size_bytes      BIGINT NOT NULL CHECK (size_bytes >= 0),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_component_artifacts_content_address
    ON component_artifacts (content_address);