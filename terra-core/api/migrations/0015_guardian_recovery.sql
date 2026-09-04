-- RFC-010: Guardian & Recovery Council mirror.
-- Guardianship is a Succession with kind 3 (GUARDIANSHIP) or 4
-- (COURT_APPOINTED_GUARDIAN). No new on-chain account type; the mirror gains
-- the court-order anchor (case_hash), the advisory scope convention
-- (scope_notes), and a revocation audit trail.

ALTER TABLE successions
    ADD COLUMN IF NOT EXISTS case_hash   TEXT NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS scope_notes TEXT NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS revoked_at  TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS revoked_by  TEXT NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS new_owner_after_revoke TEXT NOT NULL DEFAULT '';

-- Post-claim revocations (on-chain GuardianshipRevoked event mirror).
CREATE TABLE IF NOT EXISTS guardianship_revocations (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    identity_hash     TEXT NOT NULL,
    previous_guardian TEXT NOT NULL,
    new_owner         TEXT NOT NULL,
    revoked_by        TEXT NOT NULL,
    block_time        TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_successions_kind ON successions (kind);
CREATE INDEX IF NOT EXISTS idx_successions_case_hash ON successions (case_hash) WHERE case_hash <> '';
CREATE INDEX IF NOT EXISTS idx_guardianship_revocations_identity ON guardianship_revocations (identity_hash);
CREATE INDEX IF NOT EXISTS idx_guardianship_revocations_revoked_by ON guardianship_revocations (revoked_by);
