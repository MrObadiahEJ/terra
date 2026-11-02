-- Stage 5: Distributed providers — worker registration + job assignment (REQ-028 dependent).
--
-- Workers register their capabilities (CPU, GPU, storage, bandwidth, regions)
-- so the scheduler can match reconstruction jobs to capable providers.
-- Job assignments record which worker is responsible for which job.

CREATE TABLE IF NOT EXISTS worker_registrations (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    worker_id         TEXT NOT NULL UNIQUE,
    wallet_address    TEXT NOT NULL,
    capabilities      JSONB NOT NULL DEFAULT '{}',
                     -- { cpu_cores: N, gpu: bool, gpu_vram_mb: N, storage_mb: N,
                     --   bandwidth_mbps: N, regions: ["..."], engines: ["..."] }
    status            TEXT NOT NULL DEFAULT 'active',
                     -- active, inactive, drained
    last_heartbeat_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT worker_registrations_status_check
        CHECK (status IN ('active', 'inactive', 'drained'))
);

-- Job assignments: which worker is assigned to which reconstruction job.
CREATE TABLE IF NOT EXISTS job_assignments (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id            TEXT NOT NULL,
    worker_id         TEXT NOT NULL,
    assigned_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at      TIMESTAMPTZ,
    result_uri        TEXT,
    resource_receipt  JSONB,
                     -- { cpu_seconds: N, gpu_seconds: N, storage_mb: N, bandwidth_mb: N }

    CONSTRAINT job_assignments_job_worker_unique UNIQUE (job_id, worker_id)
);

CREATE INDEX IF NOT EXISTS idx_worker_registrations_wallet
    ON worker_registrations (wallet_address);
CREATE INDEX IF NOT EXISTS idx_worker_registrations_status
    ON worker_registrations (status);
CREATE INDEX IF NOT EXISTS idx_job_assignments_job
    ON job_assignments (job_id);
CREATE INDEX IF NOT EXISTS idx_job_assignments_worker
    ON job_assignments (worker_id);
