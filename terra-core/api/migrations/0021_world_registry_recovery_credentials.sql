-- WorldRegistry, CountryAllocations, GenesisRequests (Part 3)
-- ValidatorActivityTracker, EmergencyInjection (Part 4)
-- CredentialRequests, ThresholdCredentials, CredentialNullifiers (Part 5)

-- =========================================================================
-- Part 3: World Registry + Country Genesis
-- =========================================================================

CREATE TABLE IF NOT EXISTS world_registries (
    id            BIGSERIAL PRIMARY KEY,
    pubkey        TEXT NOT NULL UNIQUE,
    admin         TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS country_allocations (
    id                      BIGSERIAL PRIMARY KEY,
    world_registry_pubkey   TEXT NOT NULL REFERENCES world_registries(pubkey) ON DELETE CASCADE,
    country_code            TEXT NOT NULL,
    approved_admin          TEXT NOT NULL,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (world_registry_pubkey, country_code)
);

CREATE TABLE IF NOT EXISTS genesis_requests (
    id                        BIGSERIAL PRIMARY KEY,
    world_registry_pubkey     TEXT NOT NULL REFERENCES world_registries(pubkey) ON DELETE CASCADE,
    genesis_request_pubkey    TEXT NOT NULL UNIQUE,
    country_code              TEXT NOT NULL,
    requested_by              TEXT NOT NULL,
    confirmations_count       INT NOT NULL DEFAULT 0,
    distinct_countries        INT NOT NULL DEFAULT 0,
    finalized                 BOOLEAN NOT NULL DEFAULT FALSE,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_country_allocations_registry ON country_allocations (world_registry_pubkey);
CREATE INDEX IF NOT EXISTS idx_country_allocations_code ON country_allocations (country_code);
CREATE INDEX IF NOT EXISTS idx_genesis_requests_registry ON genesis_requests (world_registry_pubkey);
CREATE INDEX IF NOT EXISTS idx_genesis_requests_country ON genesis_requests (country_code);

-- =========================================================================
-- Part 4: Validator Recovery (activity tracking + emergency injection)
-- =========================================================================

CREATE TABLE IF NOT EXISTS validator_activities (
    id                  BIGSERIAL PRIMARY KEY,
    registry_pubkey     TEXT NOT NULL,
    validator_pubkey    TEXT NOT NULL,
    last_active         BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (registry_pubkey, validator_pubkey)
);

CREATE TABLE IF NOT EXISTS emergency_injections (
    id                  BIGSERIAL PRIMARY KEY,
    registry_pubkey     TEXT NOT NULL,
    validator_pubkey    TEXT NOT NULL,
    injected_by         TEXT NOT NULL,
    queued_at           BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT,
    execute_after       BIGINT NOT NULL DEFAULT 0,
    executed            BOOLEAN NOT NULL DEFAULT FALSE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_validator_activities_registry ON validator_activities (registry_pubkey);
CREATE INDEX IF NOT EXISTS idx_validator_activities_validator ON validator_activities (validator_pubkey);
CREATE INDEX IF NOT EXISTS idx_emergency_injections_registry ON emergency_injections (registry_pubkey);

-- =========================================================================
-- Part 5: Threshold Credentials
-- =========================================================================

CREATE TABLE IF NOT EXISTS credential_requests (
    id                  BIGSERIAL PRIMARY KEY,
    request_pubkey      TEXT NOT NULL UNIQUE,
    holder_pubkey       TEXT NOT NULL,
    zone_id             TEXT NOT NULL,
    request_hash        TEXT NOT NULL,
    status              TEXT NOT NULL DEFAULT 'pending',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS threshold_credentials (
    id                  BIGSERIAL PRIMARY KEY,
    credential_pubkey   TEXT NOT NULL UNIQUE,
    holder_pubkey       TEXT NOT NULL,
    zone_id             TEXT NOT NULL,
    issued_at           BIGINT NOT NULL DEFAULT 0,
    expires_at          BIGINT NOT NULL DEFAULT 0,
    revoked             BOOLEAN NOT NULL DEFAULT FALSE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS credential_nullifiers (
    id                  BIGSERIAL PRIMARY KEY,
    nullifier_pubkey    TEXT NOT NULL UNIQUE,
    credential_pubkey   TEXT NOT NULL REFERENCES threshold_credentials(credential_pubkey) ON DELETE CASCADE,
    nullified_at        BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())::BIGINT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_credential_requests_holder ON credential_requests (holder_pubkey);
CREATE INDEX IF NOT EXISTS idx_credential_requests_zone ON credential_requests (zone_id);
CREATE INDEX IF NOT EXISTS idx_threshold_credentials_holder ON threshold_credentials (holder_pubkey);
CREATE INDEX IF NOT EXISTS idx_threshold_credentials_zone ON threshold_credentials (zone_id);
CREATE INDEX IF NOT EXISTS idx_credential_nullifiers_credential ON credential_nullifiers (credential_pubkey);
