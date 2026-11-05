-- Worker leases for derived reconstruction jobs.

ALTER TABLE derived_jobs
    ADD COLUMN IF NOT EXISTS claimed_by TEXT,
    ADD COLUMN IF NOT EXISTS claimed_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS lease_expires_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_derived_jobs_pending_queue
    ON derived_jobs (status, created_at ASC)
    WHERE status = 'pending';
