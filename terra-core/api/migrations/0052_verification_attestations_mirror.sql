-- 0052_verification_attestations_mirror.sql
--
-- Mirrors on-chain VerificationAttestation accounts (W3C VC 2.0 + SD-JWT).
-- PDA: ["verification_attestation", claim, validator]
--
-- Layout matches VerificationAttestation Anchor account 1:1.

CREATE TABLE IF NOT EXISTS verification_attestations_onchain (
    id                   BIGSERIAL PRIMARY KEY,
    pubkey               TEXT NOT NULL UNIQUE,
    claim_pubkey         TEXT NOT NULL,
    validator_pubkey     TEXT NOT NULL,
    observation_pubkey   TEXT NOT NULL,
    credential_subject   TEXT NOT NULL,
    vc_type              VARCHAR(64) NOT NULL,
    issuance_date        TIMESTAMPTZ NOT NULL,
    expiration_date      TIMESTAMPTZ NOT NULL,
    sd_jwt_digest        TEXT NOT NULL,
    selective_disclosure BOOLEAN NOT NULL DEFAULT FALSE,
    protocol_version     SMALLINT NOT NULL DEFAULT 2,
    chain_verified       BOOLEAN NOT NULL DEFAULT TRUE,
    chain_slot           BIGINT DEFAULT 0,
    chain_signature      TEXT DEFAULT '',
    chain_created_at     TIMESTAMPTZ,
    mirrored_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_vc_attestations_claim ON verification_attestations_onchain (claim_pubkey);
CREATE INDEX IF NOT EXISTS idx_vc_attestations_subject ON verification_attestations_onchain (credential_subject);
CREATE INDEX IF NOT EXISTS idx_vc_attestations_validator ON verification_attestations_onchain (validator_pubkey);
