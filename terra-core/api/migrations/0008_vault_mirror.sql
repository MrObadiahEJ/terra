-- Vault shard protocol mirror tables (RFC-003)
-- On-chain records for threshold-encrypted vaults with Shamir secret sharing.

CREATE TABLE IF NOT EXISTS vaults (
    id              BIGSERIAL PRIMARY KEY,
    subject_pubkey  TEXT NOT NULL,
    vault_pubkey    TEXT NOT NULL UNIQUE,
    ciphertext_cid  TEXT NOT NULL,
    ciphertext_hash BYTEA NOT NULL,
    algorithm_id    SMALLINT NOT NULL DEFAULT 0,
    storage_uris    TEXT[] NOT NULL DEFAULT '{}',
    shard_holders   TEXT[] NOT NULL DEFAULT '{}',
    threshold       SMALLINT NOT NULL,
    version         INT NOT NULL DEFAULT 0,
    last_ping_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_vaults_subject ON vaults(subject_pubkey);
CREATE INDEX IF NOT EXISTS idx_vaults_vault_pubkey ON vaults(vault_pubkey);

CREATE TABLE IF NOT EXISTS vault_access_logs (
    id              BIGSERIAL PRIMARY KEY,
    vault_pubkey    TEXT NOT NULL REFERENCES vaults(vault_pubkey),
    subject_pubkey  TEXT NOT NULL,
    authority       TEXT NOT NULL,
    purpose         TEXT NOT NULL,
    expiry          TIMESTAMPTZ NOT NULL,
    nonce           BYTEA NOT NULL,
    block_time      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_vault_access_logs_vault ON vault_access_logs(vault_pubkey);

CREATE TABLE IF NOT EXISTS vault_shard_rotations (
    id                     BIGSERIAL PRIMARY KEY,
    rotation_pubkey        TEXT NOT NULL UNIQUE,
    vault_pubkey           TEXT NOT NULL REFERENCES vaults(vault_pubkey),
    old_ciphertext_hash    BYTEA NOT NULL,
    new_ciphertext_hash    BYTEA NOT NULL,
    new_shard_holders      TEXT[] NOT NULL DEFAULT '{}',
    new_threshold          SMALLINT NOT NULL,
    initiated_by           TEXT NOT NULL,
    endorsements           TEXT[] NOT NULL DEFAULT '{}',
    required_endorsements  SMALLINT NOT NULL,
    initiated_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    effective_at           TIMESTAMPTZ NOT NULL,
    status                 SMALLINT NOT NULL DEFAULT 0,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_vault_shard_rotations_vault ON vault_shard_rotations(vault_pubkey);
CREATE INDEX IF NOT EXISTS idx_vault_shard_rotations_status ON vault_shard_rotations(status);
