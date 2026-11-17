-- 0053_evidence_durability_and_provider_economy.sql
--
-- Evidence durability: quarantine + TTL + IPFS backend support.
-- Anti-spam bond: stake_bond on storage_providers for sybil-resistance.
-- Provider economy: per-replica usage metering (access_count, bytes_served).
--
-- Migration 0053 — closes: Evidence durability (quarantine/TTL),
-- Anti-spam bond (stake_bond), Storage-provider economy (metering).

-- ---------------------------------------------------------------------------
-- evidence_uploads: add quarantine + TTL columns
-- ---------------------------------------------------------------------------
ALTER TABLE evidence_uploads
    ADD COLUMN IF NOT EXISTS quarantined BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS quarantine_reason TEXT,
    ADD COLUMN IF NOT EXISTS ttl_seconds BIGINT,
    ADD COLUMN IF NOT EXISTS expires_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_evidence_uploads_quarantined
    ON evidence_uploads (quarantined) WHERE quarantined = TRUE;

CREATE INDEX IF NOT EXISTS idx_evidence_uploads_expires_at
    ON evidence_uploads (expires_at) WHERE expires_at IS NOT NULL;

-- ---------------------------------------------------------------------------
-- storage_providers: add stake_bond + wallet_address for anti-spam + identity
-- ---------------------------------------------------------------------------
ALTER TABLE storage_providers
    ADD COLUMN IF NOT EXISTS stake_bond BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS wallet_address TEXT;

CREATE INDEX IF NOT EXISTS idx_storage_providers_stake_bond
    ON storage_providers (stake_bond) WHERE stake_bond > 0;

-- ---------------------------------------------------------------------------
-- provider_replicas: add usage metering for settlement
-- ---------------------------------------------------------------------------
ALTER TABLE provider_replicas
    ADD COLUMN IF NOT EXISTS access_count BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS bytes_served BIGINT NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS idx_provider_replicas_access_count
    ON provider_replicas (access_count DESC);

-- ---------------------------------------------------------------------------
-- Check constraints
-- ---------------------------------------------------------------------------
ALTER TABLE storage_providers
    ADD CONSTRAINT storage_providers_stake_bond_check
        CHECK (stake_bond >= 0);

ALTER TABLE provider_replicas
    ADD CONSTRAINT provider_replicas_access_count_check
        CHECK (access_count >= 0),
    ADD CONSTRAINT provider_replicas_bytes_served_check
        CHECK (bytes_served >= 0);

-- ---------------------------------------------------------------------------
-- Comments
-- ---------------------------------------------------------------------------
COMMENT ON COLUMN evidence_uploads.quarantined IS 'TRUE when evidence is quarantined (failed hash/MIME/size verification)';
COMMENT ON COLUMN evidence_uploads.quarantine_reason IS 'Human-readable reason for quarantine';
COMMENT ON COLUMN evidence_uploads.ttl_seconds IS 'Time-to-live in seconds; NULL means no expiry';
COMMENT ON COLUMN evidence_uploads.expires_at IS 'Absolute expiry timestamp; NULL means no expiry';
COMMENT ON COLUMN storage_providers.stake_bond IS 'Refundable anti-spam bond (raw units); slashed on failed PoP';
COMMENT ON COLUMN storage_providers.wallet_address IS 'Wallet address for bond settlement and reward payouts';
COMMENT ON COLUMN provider_replicas.access_count IS 'Number of times this replica has been retrieved';
COMMENT ON COLUMN provider_replicas.bytes_served IS 'Total bytes served from this replica';
