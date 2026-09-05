-- Mirror for on-chain JurisdictionBinding.verified / verified_by.
-- Bindings are permissionless claims until a validator attests them.
ALTER TABLE cross_border_bindings
    ADD COLUMN IF NOT EXISTS verified BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS verified_by TEXT;

CREATE INDEX IF NOT EXISTS idx_cross_border_bindings_verified
    ON cross_border_bindings (verified) WHERE verified = true;
