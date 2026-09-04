-- Fix RFC-005 mirror column types (0014 used NUMERIC/epoch-BIGINT, but the
-- API maps lamports to BIGINT/i64 and timestamps to TIMESTAMPTZ/DateTime).
-- Expression defaults are dropped before the cast and re-set afterwards.

-- stake_pools
ALTER TABLE stake_pools
    ALTER COLUMN last_reward_distribution DROP DEFAULT,
    ALTER COLUMN created_at DROP DEFAULT,
    ALTER COLUMN updated_at DROP DEFAULT;
ALTER TABLE stake_pools
    ALTER COLUMN total_staked_lamports TYPE BIGINT,
    ALTER COLUMN accumulated_rewards_lamports TYPE BIGINT,
    ALTER COLUMN last_reward_distribution TYPE TIMESTAMPTZ USING to_timestamp(last_reward_distribution),
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING to_timestamp(created_at),
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING to_timestamp(updated_at);
ALTER TABLE stake_pools
    ALTER COLUMN last_reward_distribution SET DEFAULT now(),
    ALTER COLUMN created_at SET DEFAULT now(),
    ALTER COLUMN updated_at SET DEFAULT now();

-- validator_stakes (unbonding_starts_at stays epoch-BIGINT: 0 = not unbonding)
ALTER TABLE validator_stakes
    ALTER COLUMN created_at DROP DEFAULT,
    ALTER COLUMN updated_at DROP DEFAULT;
ALTER TABLE validator_stakes
    ALTER COLUMN staked_lamports TYPE BIGINT,
    ALTER COLUMN unbonding_lamports TYPE BIGINT,
    ALTER COLUMN rewards_accrued_lamports TYPE BIGINT,
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING to_timestamp(created_at),
    ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING to_timestamp(updated_at);
ALTER TABLE validator_stakes
    ALTER COLUMN created_at SET DEFAULT now(),
    ALTER COLUMN updated_at SET DEFAULT now();

-- slashing_reports (resolved_at stays epoch-BIGINT: 0 = unresolved)
ALTER TABLE slashing_reports
    ALTER COLUMN filed_at DROP DEFAULT,
    ALTER COLUMN appeal_deadline DROP DEFAULT,
    ALTER COLUMN created_at DROP DEFAULT;
ALTER TABLE slashing_reports
    ALTER COLUMN reporter_bond_lamports TYPE BIGINT,
    ALTER COLUMN filed_at TYPE TIMESTAMPTZ USING to_timestamp(filed_at),
    ALTER COLUMN appeal_deadline TYPE TIMESTAMPTZ USING to_timestamp(appeal_deadline),
    ALTER COLUMN created_at TYPE TIMESTAMPTZ USING to_timestamp(created_at);
ALTER TABLE slashing_reports
    ALTER COLUMN filed_at SET DEFAULT now(),
    ALTER COLUMN created_at SET DEFAULT now();
