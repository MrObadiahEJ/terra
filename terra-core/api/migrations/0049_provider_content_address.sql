-- 0049_provider_content_address.sql
--
-- Q-008 storage independence: add `content_address` to provider_replicas so
-- proof-of-possession challenges bind to the canonical content address rather
-- than the provider-specific `checksum`. This lets the API verify a replica
-- actually holds the bytes for a given content address, and lets reliability
-- be derived from challenge results (last_proof_at) instead of the
-- self-declared `reliability_score`.

ALTER TABLE provider_replicas
    ADD COLUMN IF NOT EXISTS content_address TEXT;

CREATE INDEX IF NOT EXISTS idx_provider_replicas_content_address
    ON provider_replicas (content_address)
    WHERE content_address IS NOT NULL;

-- Backfill content_address from checksum where checksum is a sha256-prefixed
-- digest. Records with a raw hex checksum are left NULL (callers should
-- re-register with the canonical address).
UPDATE provider_replicas
SET content_address = checksum
WHERE content_address IS NULL
  AND checksum IS NOT NULL
  AND (checksum LIKE 'sha256:%' OR length(checksum) = 64);
