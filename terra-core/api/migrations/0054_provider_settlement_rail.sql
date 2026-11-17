-- 0054_provider_settlement_rail.sql
--
-- Storage-provider settlement rail (RFC-014 direction, service-fee only).
--
-- Usage is metered per replica (`access_count`, `bytes_served`). At epoch
-- close, each provider's un-settled byte delta is priced in USDC, committed to
-- a Merkle root, and claimed with an inclusion proof. Q-005 stays a hard gate:
-- only payment for verified storage service is modelled — no token, no
-- fractional investment, no profit share.

-- Per-provider cumulative usage already settled. Epoch deltas are
-- `replica totals - settled_*`, so replays cannot pay the same bytes twice.
ALTER TABLE storage_providers
    ADD COLUMN IF NOT EXISTS settled_access_count BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS settled_bytes_served BIGINT NOT NULL DEFAULT 0;

ALTER TABLE storage_providers
    ADD CONSTRAINT storage_providers_settled_access_count_check
        CHECK (settled_access_count >= 0),
    ADD CONSTRAINT storage_providers_settled_bytes_served_check
        CHECK (settled_bytes_served >= 0);

CREATE TABLE IF NOT EXISTS provider_settlement_epochs (
    epoch_id BIGSERIAL PRIMARY KEY,
    currency TEXT NOT NULL DEFAULT 'USDC',
    settlement_kind TEXT NOT NULL DEFAULT 'service_fee',
    rate_micro_usdc_per_gb BIGINT NOT NULL,
    min_payout_micro_usdc BIGINT NOT NULL DEFAULT 0,
    total_micro_usdc BIGINT NOT NULL,
    merkle_root TEXT NOT NULL,
    leaf_count INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'finalized',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT provider_settlement_epochs_currency_check
        CHECK (currency = 'USDC'),
    CONSTRAINT provider_settlement_epochs_kind_check
        CHECK (settlement_kind = 'service_fee'),
    CONSTRAINT provider_settlement_epochs_status_check
        CHECK (status IN ('finalized', 'settled')),
    CONSTRAINT provider_settlement_epochs_rate_check
        CHECK (rate_micro_usdc_per_gb > 0),
    CONSTRAINT provider_settlement_epochs_min_check
        CHECK (min_payout_micro_usdc >= 0),
    CONSTRAINT provider_settlement_epochs_total_check
        CHECK (total_micro_usdc >= 0),
    CONSTRAINT provider_settlement_epochs_leaf_count_check
        CHECK (leaf_count >= 0)
);

CREATE TABLE IF NOT EXISTS provider_settlement_leaves (
    epoch_id BIGINT NOT NULL
        REFERENCES provider_settlement_epochs (epoch_id) ON DELETE CASCADE,
    provider_id UUID NOT NULL
        REFERENCES storage_providers (id) ON DELETE CASCADE,
    provider_wallet TEXT NOT NULL,
    access_count BIGINT NOT NULL,
    bytes_served BIGINT NOT NULL,
    amount_micro_usdc BIGINT NOT NULL,
    leaf_hash TEXT NOT NULL,
    leaf_index INTEGER NOT NULL,
    claimed_at TIMESTAMPTZ,
    claim_reference TEXT,
    PRIMARY KEY (epoch_id, provider_id),
    CONSTRAINT provider_settlement_leaves_amount_check
        CHECK (amount_micro_usdc > 0),
    CONSTRAINT provider_settlement_leaves_usage_check
        CHECK (access_count >= 0 AND bytes_served >= 0)
);

CREATE INDEX IF NOT EXISTS idx_provider_settlement_leaves_unclaimed
    ON provider_settlement_leaves (epoch_id) WHERE claimed_at IS NULL;

COMMENT ON COLUMN storage_providers.settled_bytes_served IS
    'Cumulative replica bytes already paid in a settlement epoch';
COMMENT ON COLUMN storage_providers.settled_access_count IS
    'Cumulative replica retrievals already covered by a settlement epoch';
COMMENT ON TABLE provider_settlement_epochs IS
    'RFC-014-style USDC service-fee epochs; Q-005 hard gate (no token/yield)';
COMMENT ON TABLE provider_settlement_leaves IS
    'Per-provider settlement leaf + Merkle inclusion claim state';
