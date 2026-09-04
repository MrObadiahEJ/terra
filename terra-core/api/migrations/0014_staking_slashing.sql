-- Staking / Slashing tables (RFC-005)
CREATE TABLE IF NOT EXISTS stake_pools (
    stake_pool_address TEXT PRIMARY KEY,
    region_registry_address TEXT NOT NULL,
    total_staked_lamports NUMERIC NOT NULL DEFAULT 0,
    reward_rate_bps SMALLINT NOT NULL DEFAULT 0,
    accumulated_rewards_lamports NUMERIC NOT NULL DEFAULT 0,
    last_reward_distribution BIGINT NOT NULL DEFAULT 0,
    slash_count INT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT,
    updated_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT
);

CREATE TABLE IF NOT EXISTS validator_stakes (
    validator_stake_address TEXT PRIMARY KEY,
    stake_pool_address TEXT NOT NULL REFERENCES stake_pools(stake_pool_address),
    validator_address TEXT NOT NULL,
    staked_lamports NUMERIC NOT NULL DEFAULT 0,
    unbonding_lamports NUMERIC NOT NULL DEFAULT 0,
    unbonding_starts_at BIGINT NOT NULL DEFAULT 0,
    rewards_accrued_lamports NUMERIC NOT NULL DEFAULT 0,
    slash_history SMALLINT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT,
    updated_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT,
    UNIQUE (stake_pool_address, validator_address)
);

CREATE TABLE IF NOT EXISTS slashing_reports (
    slashing_report_address TEXT PRIMARY KEY,
    stake_pool_address TEXT NOT NULL REFERENCES stake_pools(stake_pool_address),
    reporter_address TEXT NOT NULL,
    evidence_hash BYTEA NOT NULL,
    offender_address TEXT NOT NULL,
    offense_type SMALLINT NOT NULL,
    offense_details BYTEA NOT NULL,
    reporter_bond_lamports NUMERIC NOT NULL DEFAULT 0,
    status SMALLINT NOT NULL DEFAULT 0,
    filed_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT,
    appeal_deadline BIGINT NOT NULL DEFAULT 0,
    resolved_at BIGINT NOT NULL DEFAULT 0,
    created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT
);

CREATE INDEX IF NOT EXISTS idx_validator_stakes_pool ON validator_stakes(stake_pool_address);
CREATE INDEX IF NOT EXISTS idx_validator_stakes_validator ON validator_stakes(validator_address);
CREATE INDEX IF NOT EXISTS idx_slashing_reports_pool ON slashing_reports(stake_pool_address);
CREATE INDEX IF NOT EXISTS idx_slashing_reports_offender ON slashing_reports(offender_address);
CREATE INDEX IF NOT EXISTS idx_slashing_reports_status ON slashing_reports(status);
