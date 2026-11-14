-- RFC-017 — Secure Evidence Intake, Rendition & Accountable Validation
-- §7: Assurance Profiles; §2: Intake pipeline
--
-- Provider proof-of-possession: a provider that claims to hold a replica of
-- a component artifact must periodically prove it actually has the bytes.
-- The API issues a challenge; the provider computes SHA-256(challenge +
// content_address) and signs it. The API verifies the proof against its own
// copy of the artifact bytes.
--
-- `last_challenge` stores the most recent challenge sent to the provider.
-- `last_proof_at` records when the provider last successfully proved
-- possession. A NULL `last_proof_at` means the replica's proof has expired
-- or was never verified.

ALTER TABLE provider_replicas
    ADD COLUMN IF NOT EXISTS last_challenge TEXT,
    ADD COLUMN IF NOT EXISTS last_proof_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_provider_replicas_last_proof_at
    ON provider_replicas (last_proof_at);
