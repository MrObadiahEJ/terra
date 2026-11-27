-- 0058_indexer_mirror_honesty.sql
--
-- Forward fix for Book of Intents findings 1d/1e (Sprint C addendum).
--
-- 1. `rights` had no onchain_id, so the Chain indexer had no stable key for an
--    on-chain Rights PDA and would duplicate a row on every poll. Add the
--    key (partial UNIQUE index; client-created rows keep NULL) and include it
--    in the mirror hash so the anchor itself is tamper-evident.
--
-- 2. `verification_attestations_onchain` was created with the opposite of
--    ADR-026 semantics: chain_verified DEFAULT TRUE, chain_slot DEFAULT 0,
--    chain_signature DEFAULT ''. Every new row therefore claimed to be
--    chain-verified without a transaction. Correct the defaults and
--    downgrade existing rows. The indexer re-establishes honest provenance
--    from a real transaction on its next successful poll; until then the
--    rows are correctly reported as unverified.

ALTER TABLE rights ADD COLUMN IF NOT EXISTS onchain_id TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_rights_onchain_id
    ON rights (onchain_id) WHERE onchain_id IS NOT NULL;

-- Include the anchor in the rights mirror hash, matching parcels. Existing
-- rows get their hash recomputed by the same UPDATE below.
CREATE OR REPLACE FUNCTION compute_rights_mirror_hash()
RETURNS trigger AS $$
DECLARE
    h text;
BEGIN
    SELECT encode(digest(
        coalesce(NEW.onchain_id, '') || '|' ||
        coalesce(NEW.parcel_id::text, '') || '|' ||
        NEW.rights_kind::text || '|' ||
        coalesce(NEW.holder, '') || '|' ||
        coalesce(NEW.granter, '') || '|' ||
        coalesce(NEW.notes, '') || '|' ||
        coalesce(to_char(NEW.created_at, 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'), '') || '|' ||
        coalesce(to_char(NEW.expires_at, 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"'), ''),
        'sha256'), 'hex')
    INTO h;
    NEW.mirror_hash := h;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- The trigger fires BEFORE UPDATE for every row, so this no-op update
-- recomputes all existing rights mirror hashes with the new anchor field.
UPDATE rights SET onchain_id = onchain_id;

-- ADR-026: no transaction means no verification, no slot, no signature.
ALTER TABLE verification_attestations_onchain
    ALTER COLUMN chain_verified SET DEFAULT FALSE,
    ALTER COLUMN chain_slot SET DEFAULT NULL,
    ALTER COLUMN chain_signature SET DEFAULT NULL;

-- Downgrade all pre-existing rows. Their slot was the poll-time slot (not a
-- transaction slot) and an empty signature is not a signature; re-mirroring
-- is the only honest way to restore chain_verified = TRUE.
UPDATE verification_attestations_onchain
SET chain_verified = FALSE,
    chain_slot = NULL,
    chain_signature = NULL
WHERE chain_verified <> FALSE
   OR chain_slot IS NOT NULL
   OR (chain_signature IS NOT NULL AND chain_signature <> '');
