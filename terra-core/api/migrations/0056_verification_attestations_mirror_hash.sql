-- 0056_verification_attestations_mirror_hash.sql
-- Add mirror_hash to verification_attestations_onchain (missed in 0048/0052).

ALTER TABLE verification_attestations_onchain
    ADD COLUMN IF NOT EXISTS mirror_hash TEXT;

CREATE OR REPLACE FUNCTION compute_verification_attestations_onchain_mirror_hash()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
DECLARE h TEXT;
BEGIN
    h := encode(digest(
        coalesce(NEW.claim_pubkey, '') || '|' ||
        coalesce(NEW.validator_pubkey, '') || '|' ||
        coalesce(NEW.observation_pubkey, '') || '|' ||
        coalesce(NEW.credential_subject, '') || '|' ||
        NEW.vc_type || '|' ||
        to_char(NEW.issuance_date, 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') || '|' ||
        to_char(NEW.expiration_date, 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') || '|' ||
        NEW.sd_jwt_digest || '|' ||
        NEW.selective_disclosure::text,
        'sha256'), 'hex');
    NEW.mirror_hash := h;
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_verification_attestations_onchain_mirror_hash
    ON verification_attestations_onchain;
CREATE TRIGGER trg_verification_attestations_onchain_mirror_hash
    BEFORE INSERT OR UPDATE ON verification_attestations_onchain
    FOR EACH ROW EXECUTE FUNCTION compute_verification_attestations_onchain_mirror_hash();

CREATE INDEX IF NOT EXISTS idx_verification_attestations_onchain_mirror_hash
    ON verification_attestations_onchain (mirror_hash) WHERE mirror_hash IS NOT NULL;
