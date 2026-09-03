-- Cross-border identity jurisdictions (RFC-006)
CREATE TABLE IF NOT EXISTS jurisdictions (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    country_code          TEXT NOT NULL UNIQUE,
    authority             TEXT NOT NULL,
    jurisdiction_name     TEXT NOT NULL,
    credential_schema_cid TEXT NOT NULL,
    revocation_registry   TEXT NOT NULL,
    verification_key_hash TEXT NOT NULL,
    algorithm_id          SMALLINT NOT NULL DEFAULT 0,
    status                TEXT NOT NULL DEFAULT 'active',
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS cross_border_bindings (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    jurisdiction_id         UUID NOT NULL REFERENCES jurisdictions(id) ON DELETE CASCADE,
    identity_hash           TEXT NOT NULL,
    credential_commitment   TEXT NOT NULL,
    nullifier               TEXT NOT NULL,
    proof_data              TEXT NOT NULL DEFAULT '',
    proof_version           SMALLINT NOT NULL DEFAULT 0,
    algorithm_id            SMALLINT NOT NULL DEFAULT 0,
    revoked                 BOOLEAN NOT NULL DEFAULT false,
    revoked_at              TIMESTAMPTZ,
    revoked_by              TEXT,
    bound_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at              TIMESTAMPTZ,
    version                 BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_jurisdictions_country      ON jurisdictions (country_code);
CREATE INDEX IF NOT EXISTS idx_jurisdictions_authority    ON jurisdictions (authority);
CREATE INDEX IF NOT EXISTS idx_bindings_jurisdiction      ON cross_border_bindings (jurisdiction_id);
CREATE INDEX IF NOT EXISTS idx_bindings_identity          ON cross_border_bindings (identity_hash);
CREATE INDEX IF NOT EXISTS idx_bindings_nullifier         ON cross_border_bindings (nullifier);
CREATE INDEX IF NOT EXISTS idx_bindings_revoked           ON cross_border_bindings (revoked);
