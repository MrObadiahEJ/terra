-- 0060_chain_accounts_generic_mirror.sql
--
-- Q-033 (b): staged rollout of the chain read model. Stage 0 keeps the eight
-- typed projections (parcels, rights, claims, spatial mirrors, attestations).
-- Stage 1 adds a lossless generic mirror for EVERY account type in the IDL,
-- so no program account is left client-trust only. Promoting a type to a
-- typed table + API endpoint is a later stage, funded separately.
--
-- Every row carries ADR-026 transaction provenance: chain_verified=TRUE only
-- when a real, non-failing signature was resolved for that account.

CREATE TABLE IF NOT EXISTS chain_accounts (
    pubkey TEXT PRIMARY KEY,
    account_type TEXT NOT NULL,
    fields JSONB NOT NULL,
    mirror_hash TEXT,
    chain_verified BOOLEAN NOT NULL DEFAULT FALSE,
    chain_slot BIGINT,
    chain_signature TEXT,
    mirrored_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_chain_accounts_account_type
    ON chain_accounts (account_type);
CREATE INDEX IF NOT EXISTS idx_chain_accounts_unverified
    ON chain_accounts (chain_verified) WHERE chain_verified = FALSE;
CREATE INDEX IF NOT EXISTS idx_chain_accounts_mirror_hash
    ON chain_accounts (mirror_hash) WHERE mirror_hash IS NOT NULL;

CREATE OR REPLACE FUNCTION compute_chain_accounts_mirror_hash()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    -- jsonb::text is canonical (sorted keys), so this hash is stable.
    NEW.mirror_hash := encode(digest(
        NEW.pubkey || '|' || NEW.account_type || '|' || NEW.fields::text,
        'sha256'), 'hex');
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_chain_accounts_mirror_hash ON chain_accounts;
CREATE TRIGGER trg_chain_accounts_mirror_hash
    BEFORE INSERT OR UPDATE ON chain_accounts
    FOR EACH ROW EXECUTE FUNCTION compute_chain_accounts_mirror_hash();
