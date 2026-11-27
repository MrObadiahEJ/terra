-- 0057_claims_onchain_mirror.sql
--
-- P0 forward-fix: the chain indexer decodes on-chain `Claim` accounts and
-- calls upsert_claim() unconditionally, but no migration ever created the
-- `claims_onchain` mirror table. The first Claim PDA on devnet made
-- `SELECT id FROM claims_onchain` fail, `?` aborted the whole poll, and every
-- account after it in RPC order stopped mirroring (warn-logged only).
--
-- Layout matches ParsedClaim 1:1. Provenance columns follow 0051 semantics:
-- `chain_verified` is TRUE only when the indexer resolved a real transaction
-- signature for the account; `chain_slot` is that transaction's slot; and
-- `chain_signature` is that signature. They are NULL/FALSE otherwise — never
-- the poll-time slot and never an empty string standing in for a signature.

CREATE TABLE IF NOT EXISTS claims_onchain (
    id                     BIGSERIAL PRIMARY KEY,
    pubkey                 TEXT NOT NULL UNIQUE,
    claim_id               TEXT NOT NULL DEFAULT '',
    parcel_pubkey          TEXT NOT NULL DEFAULT '',
    claim_type             SMALLINT NOT NULL DEFAULT 0,
    submitted_by           TEXT NOT NULL DEFAULT '',
    status                 SMALLINT NOT NULL DEFAULT 0,
    statement_hash         TEXT NOT NULL DEFAULT '',
    version                SMALLINT NOT NULL DEFAULT 0,
    evidence_count         SMALLINT NOT NULL DEFAULT 0,
    observation_count      SMALLINT NOT NULL DEFAULT 0,
    attestation_count      SMALLINT NOT NULL DEFAULT 0,
    required_attestations  SMALLINT NOT NULL DEFAULT 0,
    chain_created_at       TIMESTAMPTZ,
    chain_updated_at       TIMESTAMPTZ,
    chain_verified         BOOLEAN NOT NULL DEFAULT FALSE,
    chain_slot             BIGINT,
    chain_signature        TEXT,
    mirror_hash            TEXT,
    mirrored_at            TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_claims_onchain_claim_id
    ON claims_onchain (claim_id);
CREATE INDEX IF NOT EXISTS idx_claims_onchain_parcel
    ON claims_onchain (parcel_pubkey);
CREATE INDEX IF NOT EXISTS idx_claims_onchain_chain_verified
    ON claims_onchain (chain_verified) WHERE chain_verified = FALSE;

-- Mirror hash anchoring (R-004), computed on every INSERT/UPDATE like the
-- other mirror tables. `GET /mirror/audit` can be extended to claims without
-- another migration.
CREATE OR REPLACE FUNCTION compute_claims_onchain_mirror_hash()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE h TEXT;
BEGIN
    h := encode(digest(
        coalesce(NEW.pubkey, '') || '|' ||
        coalesce(NEW.claim_id, '') || '|' ||
        coalesce(NEW.parcel_pubkey, '') || '|' ||
        NEW.claim_type::text || '|' ||
        coalesce(NEW.submitted_by, '') || '|' ||
        NEW.status::text || '|' ||
        coalesce(NEW.statement_hash, '') || '|' ||
        NEW.version::text || '|' ||
        NEW.evidence_count::text || '|' ||
        NEW.observation_count::text || '|' ||
        NEW.attestation_count::text || '|' ||
        NEW.required_attestations::text || '|' ||
        coalesce(to_char(NEW.chain_created_at, 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'), '') || '|' ||
        coalesce(to_char(NEW.chain_updated_at, 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'), ''),
        'sha256'), 'hex');
    NEW.mirror_hash := h;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_claims_onchain_mirror_hash ON claims_onchain;
CREATE TRIGGER trg_claims_onchain_mirror_hash
    BEFORE INSERT OR UPDATE ON claims_onchain
    FOR EACH ROW EXECUTE FUNCTION compute_claims_onchain_mirror_hash();

CREATE INDEX IF NOT EXISTS idx_claims_onchain_mirror_hash
    ON claims_onchain (mirror_hash) WHERE mirror_hash IS NOT NULL;
