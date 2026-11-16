-- 0051_chain_verified_columns.sql
--
-- Sprint C: chain-verified mirror columns.
--
-- Adds `chain_verified`, `chain_slot`, and `chain_signature` to each off-chain
-- mirror table. The indexer (api/src/indexer.rs) polls the Solana RPC,
-- decodes Anchor accounts, and upserts mirror rows with these fields set.
-- Client-POSTed rows start with chain_verified = false and remain so until the
-- indexer confirms them against the chain.
--
-- Tables covered: parcels, rights, spatial_assets_onchain,
-- geometry_versions_onchain, three_d_models_onchain, spatial_snapshots_onchain.

-- ---------------------------------------------------------------------------
-- parcels
-- ---------------------------------------------------------------------------
ALTER TABLE parcels
    ADD COLUMN IF NOT EXISTS chain_verified BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS chain_slot BIGINT,
    ADD COLUMN IF NOT EXISTS chain_signature TEXT;

CREATE INDEX IF NOT EXISTS idx_parcels_chain_verified
    ON parcels (chain_verified) WHERE chain_verified = FALSE;

-- ---------------------------------------------------------------------------
-- rights
-- ---------------------------------------------------------------------------
ALTER TABLE rights
    ADD COLUMN IF NOT EXISTS chain_verified BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS chain_slot BIGINT,
    ADD COLUMN IF NOT EXISTS chain_signature TEXT;

CREATE INDEX IF NOT EXISTS idx_rights_chain_verified
    ON rights (chain_verified) WHERE chain_verified = FALSE;

-- ---------------------------------------------------------------------------
-- spatial_assets_onchain
-- ---------------------------------------------------------------------------
ALTER TABLE spatial_assets_onchain
    ADD COLUMN IF NOT EXISTS chain_verified BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS chain_slot BIGINT,
    ADD COLUMN IF NOT EXISTS chain_signature TEXT;

CREATE INDEX IF NOT EXISTS idx_spatial_assets_onchain_chain_verified
    ON spatial_assets_onchain (chain_verified) WHERE chain_verified = FALSE;

-- ---------------------------------------------------------------------------
-- geometry_versions_onchain
-- ---------------------------------------------------------------------------
ALTER TABLE geometry_versions_onchain
    ADD COLUMN IF NOT EXISTS chain_verified BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS chain_slot BIGINT,
    ADD COLUMN IF NOT EXISTS chain_signature TEXT;

CREATE INDEX IF NOT EXISTS idx_geometry_versions_onchain_chain_verified
    ON geometry_versions_onchain (chain_verified) WHERE chain_verified = FALSE;

-- ---------------------------------------------------------------------------
-- three_d_models_onchain
-- ---------------------------------------------------------------------------
ALTER TABLE three_d_models_onchain
    ADD COLUMN IF NOT EXISTS chain_verified BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS chain_slot BIGINT,
    ADD COLUMN IF NOT EXISTS chain_signature TEXT;

CREATE INDEX IF NOT EXISTS idx_three_d_models_onchain_chain_verified
    ON three_d_models_onchain (chain_verified) WHERE chain_verified = FALSE;

-- ---------------------------------------------------------------------------
-- spatial_snapshots_onchain
-- ---------------------------------------------------------------------------
ALTER TABLE spatial_snapshots_onchain
    ADD COLUMN IF NOT EXISTS chain_verified BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS chain_slot BIGINT,
    ADD COLUMN IF NOT EXISTS chain_signature TEXT;

CREATE INDEX IF NOT EXISTS idx_spatial_snapshots_onchain_chain_verified
    ON spatial_snapshots_onchain (chain_verified) WHERE chain_verified = FALSE;
