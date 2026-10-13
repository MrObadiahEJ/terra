-- RFC-013 Phase E — spatial account mirrors.
--
-- Mirrors the on-chain SpatialAsset / GeometryVersion / ThreeDModel /
-- SpatialSnapshot accounts written by terra_registry Phases A–C. The
-- indexer owns these tables; the read-only REST layer
-- (routes/spatial_accounts.rs) serves them to the globe client (Phase D:
-- geometry-version history, 3D models, point-in-time snapshots).
--
-- Layout mirrors the Anchor structs 1:1. Chain timestamps (unix i64) land
-- as TIMESTAMPTZ; `mirrored_at` records indexer freshness. Hex strings
-- carry the 32-byte keys/hashes (64 chars, no 0x prefix).

-- SpatialAsset PDA (PDA: ["spatial_asset", parcel]).
CREATE TABLE IF NOT EXISTS spatial_assets_onchain (
    id                     BIGSERIAL PRIMARY KEY,
    pubkey                 TEXT NOT NULL UNIQUE,
    parcel_pubkey          TEXT NOT NULL,
    authority              TEXT NOT NULL,
    dimensionality         SMALLINT NOT NULL DEFAULT 0, -- spatial_dimension
    elevation_min_mm       INTEGER NOT NULL DEFAULT 0,
    elevation_max_mm       INTEGER NOT NULL DEFAULT 0,
    geometry_version_count INTEGER NOT NULL DEFAULT 0,
    latest_geometry_hash   TEXT NOT NULL DEFAULT '',
    chain_created_at       TIMESTAMPTZ,
    chain_updated_at       TIMESTAMPTZ,
    mirrored_at            TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_spatial_assets_onchain_parcel
    ON spatial_assets_onchain (parcel_pubkey);

-- GeometryVersion PDA (PDA: ["geometry_version", asset, version]).
CREATE TABLE IF NOT EXISTS geometry_versions_onchain (
    id                 BIGSERIAL PRIMARY KEY,
    pubkey             TEXT NOT NULL UNIQUE,
    asset_pubkey       TEXT NOT NULL REFERENCES spatial_assets_onchain (pubkey) ON DELETE CASCADE,
    parcel_pubkey      TEXT NOT NULL,
    version            INTEGER NOT NULL,
    geometry_hash      TEXT NOT NULL DEFAULT '',
    source             SMALLINT NOT NULL DEFAULT 0,  -- geometry_source
    dimension          SMALLINT NOT NULL DEFAULT 0,  -- spatial_dimension
    storage_reference  TEXT NOT NULL DEFAULT '',
    submitted_by       TEXT NOT NULL DEFAULT '',
    chain_submitted_at TIMESTAMPTZ,
    verified           BOOLEAN NOT NULL DEFAULT FALSE,
    verified_by        TEXT,
    chain_verified_at  TIMESTAMPTZ,
    evidence_manifest  TEXT,                         -- pubkey, NULL = standalone
    elevation_min_mm   INTEGER NOT NULL DEFAULT 0,
    elevation_max_mm   INTEGER NOT NULL DEFAULT 0,
    elevation_source   SMALLINT NOT NULL DEFAULT 0,  -- elevation_source
    required           SMALLINT NOT NULL DEFAULT 0,
    attest_count       SMALLINT NOT NULL DEFAULT 0,
    attestors          TEXT[] NOT NULL DEFAULT '{}', -- hex pubkeys, zeros dropped
    mirrored_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (asset_pubkey, version)
);

CREATE INDEX IF NOT EXISTS idx_geometry_versions_onchain_asset
    ON geometry_versions_onchain (asset_pubkey, version);

-- ThreeDModel PDA (PDA: ["three_d_model", asset, model_version]).
-- Distinct from `three_d_models` (0029): that table holds reconstruction
-- pipeline *products*; this one mirrors the on-chain *claims*.
CREATE TABLE IF NOT EXISTS three_d_models_onchain (
    id                 BIGSERIAL PRIMARY KEY,
    pubkey             TEXT NOT NULL UNIQUE,
    asset_pubkey       TEXT NOT NULL REFERENCES spatial_assets_onchain (pubkey) ON DELETE CASCADE,
    parcel_pubkey      TEXT NOT NULL,
    model_version      INTEGER NOT NULL,
    geometry_version   INTEGER NOT NULL,
    model_hash         TEXT NOT NULL DEFAULT '',
    storage_reference  TEXT NOT NULL DEFAULT '',
    model_format       SMALLINT NOT NULL DEFAULT 0,  -- model_format
    submitted_by       TEXT NOT NULL DEFAULT '',
    chain_submitted_at TIMESTAMPTZ,
    verified           BOOLEAN NOT NULL DEFAULT FALSE,
    verified_by        TEXT,
    chain_verified_at  TIMESTAMPTZ,
    mirrored_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (asset_pubkey, model_version)
);

CREATE INDEX IF NOT EXISTS idx_three_d_models_onchain_asset
    ON three_d_models_onchain (asset_pubkey, model_version);

-- SpatialSnapshot PDA (PDA: ["spatial_snapshot", asset, snapshot_index]).
CREATE TABLE IF NOT EXISTS spatial_snapshots_onchain (
    id                     BIGSERIAL PRIMARY KEY,
    pubkey                 TEXT NOT NULL UNIQUE,
    asset_pubkey           TEXT NOT NULL REFERENCES spatial_assets_onchain (pubkey) ON DELETE CASCADE,
    parcel_pubkey          TEXT NOT NULL,
    snapshot_index         INTEGER NOT NULL,
    geometry_version_count INTEGER NOT NULL DEFAULT 0,
    latest_geometry_hash   TEXT NOT NULL DEFAULT '',
    dimensionality         SMALLINT NOT NULL DEFAULT 0,
    elevation_min_mm       INTEGER NOT NULL DEFAULT 0,
    elevation_max_mm       INTEGER NOT NULL DEFAULT 0,
    label                  TEXT NOT NULL DEFAULT '',
    created_by             TEXT NOT NULL DEFAULT '',
    chain_created_at       TIMESTAMPTZ,
    mirrored_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (asset_pubkey, snapshot_index)
);

CREATE INDEX IF NOT EXISTS idx_spatial_snapshots_onchain_asset
    ON spatial_snapshots_onchain (asset_pubkey, snapshot_index);
