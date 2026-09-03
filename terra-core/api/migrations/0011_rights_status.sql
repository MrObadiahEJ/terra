-- RFC-009: Time-bound credentials — add status + grace_period_secs to rights
ALTER TABLE rights
    ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT 'active',
    ADD COLUMN IF NOT EXISTS grace_period_secs BIGINT NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS idx_rights_status ON rights (status);
