-- Mirror of the on-chain ValidatorNomination account.
-- Created when an existing validator nominates a new candidate; rows are
-- updated as confirmations arrive and eventually finalized.

CREATE TABLE IF NOT EXISTS validator_nominations (
    pubkey               BYTEA PRIMARY KEY,
    registry             BYTEA NOT NULL,
    country_code         BYTEA NOT NULL,          -- 2-byte ISO 3166-1 alpha-2
    sponsor              BYTEA NOT NULL,
    candidate            BYTEA NOT NULL,
    documents_hash       BYTEA NOT NULL,          -- SHA-256
    location_hash        BYTEA NOT NULL,          -- SHA-256
    assigned_physical_confirmer BYTEA NOT NULL,
    confirmers           BYTEA[] NOT NULL DEFAULT '{}',
    finalized            BOOLEAN NOT NULL DEFAULT FALSE,
    created_at           BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_validator_nominations_registry
    ON validator_nominations (registry);

CREATE INDEX IF NOT EXISTS idx_validator_nominations_candidate
    ON validator_nominations (candidate);

CREATE INDEX IF NOT EXISTS idx_validator_nominations_sponsor
    ON validator_nominations (sponsor);
