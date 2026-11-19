-- 0055_provider_settlement_multi_asset.sql
--
-- Removes the US-only assumption from the provider settlement rail.
--
-- 0054 hard-coded `currency = 'USDC'`, which made an operator-run payout rail
-- depend on a single permissioned issuer. The data plane (registry, storage,
-- retrieval, proofs) never touched USDC; only the payout accounting did. This
-- migration makes the settlement asset a row in a registry table instead of a
-- CHECK constant, so adding or removing a rail is a data change, not a
-- redeploy.
--
-- Q-005 hard gate is unchanged: `settlement_kind` stays `service_fee`. The
-- registry models units of account for paying verified storage work. It is not
-- a token, not a yield instrument, and not a profit share.
--
-- Assets are identified by a short uppercase symbol and carry their own
-- decimals + default rate. `amount_micro_units` is denominated in 10^-6 of the
-- asset's smallest unit (SOL: lamports, USDC: micro-USDC).
--
-- Pre-launch reset: 0054 had no production epochs (devnet deploy is still
-- gated on Q-001 funding), and the leaf hash domain changes in this migration
-- to bind the currency. Existing rows were test artifacts, so the rail is
-- cleared rather than migrated with a dual hash-version code path.

-- ---------------------------------------------------------------------------
-- 1. Asset registry
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS settlement_assets (
    currency TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    -- Number of decimal places in one whole unit of the asset.
    decimals SMALLINT NOT NULL,
    kind TEXT NOT NULL,
    -- Issuer/operator, for transparency only. NULL means no issuer (native).
    issuer TEXT,
    is_enabled BOOLEAN NOT NULL DEFAULT true,
    -- Exactly one enabled asset is the default for epoch closes that do not
    -- name a currency. A partial unique index below enforces the singleton.
    is_default BOOLEAN NOT NULL DEFAULT false,
    default_rate_micro_units_per_gb BIGINT NOT NULL,
    min_payout_micro_units BIGINT NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT settlement_assets_decimals_check
        CHECK (decimals >= 0 AND decimals <= 18),
    CONSTRAINT settlement_assets_kind_check
        CHECK (kind IN ('native', 'stablecoin', 'other')),
    CONSTRAINT settlement_assets_rate_check
        CHECK (default_rate_micro_units_per_gb > 0),
    CONSTRAINT settlement_assets_min_payout_check
        CHECK (min_payout_micro_units >= 0),
    CONSTRAINT settlement_assets_currency_format_check
        CHECK (currency ~ '^[A-Z0-9]{2,16}$')
);

CREATE UNIQUE INDEX IF NOT EXISTS settlement_assets_single_default
    ON settlement_assets ((is_default)) WHERE is_default;

COMMENT ON TABLE settlement_assets IS
    'Payout assets for storage service fees; rows, not CHECK constants, so no single issuer is hard-coded';

-- Neutral default: the chain's native asset, issued by no company or country.
-- Rate is a starting point (~0.00066667 SOL/GB, roughly $0.10/GB at $150 SOL)
-- that operators override per epoch as the market moves.
INSERT INTO settlement_assets (
    currency, display_name, decimals, kind, issuer, is_enabled, is_default,
    default_rate_micro_units_per_gb, min_payout_micro_units, notes
)
VALUES
    ('SOL', 'Solana (native)', 9, 'native', NULL, true, true,
     666_667, 0,
     'Default payout asset. No issuer; censorship resistance bounded by validator set.'),
    ('USDC', 'USD Coin (Solana)', 6, 'stablecoin', 'Circle', true, false,
     100_000, 0,
     'Optional stable unit of account. Permissioned asset: issuer can freeze addresses.'),
    ('EURC', 'Euro Coin (Solana)', 6, 'stablecoin', 'Circle', true, false,
     100_000, 0,
     'Optional euro unit of account. Same issuer-freeze caveat as USDC.')
ON CONFLICT (currency) DO NOTHING;

-- ---------------------------------------------------------------------------
-- 2. Provider payout preference
-- ---------------------------------------------------------------------------

-- NULL means "use the current default asset". A provider that wants to be paid
-- in a specific rail opts in explicitly; nobody is forced onto USD.
ALTER TABLE storage_providers
    ADD COLUMN IF NOT EXISTS payout_currency TEXT
        REFERENCES settlement_assets (currency);

COMMENT ON COLUMN storage_providers.payout_currency IS
    'Preferred settlement asset; NULL = default asset. NULL after 2026 = SOL.';

CREATE INDEX IF NOT EXISTS idx_storage_providers_payout_currency
    ON storage_providers (payout_currency)
    WHERE payout_currency IS NOT NULL;

-- ---------------------------------------------------------------------------
-- 3. Reset the pre-launch rail (leaf hash gains the currency domain)
-- ---------------------------------------------------------------------------

DELETE FROM provider_settlement_leaves;
DELETE FROM provider_settlement_epochs;

-- ---------------------------------------------------------------------------
-- 4. Generic units
-- ---------------------------------------------------------------------------

ALTER TABLE provider_settlement_epochs
    RENAME COLUMN rate_micro_usdc_per_gb TO rate_micro_units_per_gb;

ALTER TABLE provider_settlement_epochs
    RENAME COLUMN min_payout_micro_usdc TO min_payout_micro_units;

ALTER TABLE provider_settlement_epochs
    RENAME COLUMN total_micro_usdc TO total_micro_units;

ALTER TABLE provider_settlement_leaves
    RENAME COLUMN amount_micro_usdc TO amount_micro_units;

-- ---------------------------------------------------------------------------
-- 5. Currency is a registry reference, not a constant
-- ---------------------------------------------------------------------------

ALTER TABLE provider_settlement_epochs
    DROP CONSTRAINT IF EXISTS provider_settlement_epochs_currency_check;

ALTER TABLE provider_settlement_epochs
    ADD CONSTRAINT provider_settlement_epochs_currency_fk
        FOREIGN KEY (currency) REFERENCES settlement_assets (currency);

ALTER TABLE provider_settlement_leaves
    ADD COLUMN IF NOT EXISTS currency TEXT NOT NULL DEFAULT 'SOL';

ALTER TABLE provider_settlement_leaves
    ADD CONSTRAINT provider_settlement_leaves_currency_fk
        FOREIGN KEY (currency) REFERENCES settlement_assets (currency);

COMMENT ON COLUMN provider_settlement_epochs.currency IS
    'Settlement asset symbol from settlement_assets; asset-agnostic since 0055';
COMMENT ON COLUMN provider_settlement_epochs.rate_micro_units_per_gb IS
    'Micro-units of `currency` per GB served in this epoch';
COMMENT ON COLUMN provider_settlement_epochs.total_micro_units IS
    'Sum of leaf amounts, denominated in 10^-6 of the asset smallest unit';
COMMENT ON COLUMN provider_settlement_leaves.amount_micro_units IS
    'Payout amount in 10^-6 units of the leaf currency';
