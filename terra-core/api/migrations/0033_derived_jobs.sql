-- Incremental derived compute (RFC-014 Phase 04): compute-once, reuse-many.
--
-- A derived job is any expensive, deterministic result (a routed polyline, a
-- reconstruction draft, a geometry digest) whose output is a pure function of
-- its inputs. The `processing_key` is a SHA-256 over the canonical inputs; it
-- carries a UNIQUE constraint so two callers asking for the same work share a
-- single row instead of recomputing. This is a cache with ledger semantics —
-- not a fact. The derived result is only as trustworthy as its inputs, and it
-- is invalidated (never silently trusted stale) when an upstream dependency
-- changes.
--
-- Nothing here is on-chain state; it is a Postgres-side optimization whose
-- only correctness obligation is: same key => same output, and a changed
-- dependency => the key is recomputed (old row marked stale, new row written).

CREATE TABLE IF NOT EXISTS derived_jobs (
    id             VARCHAR(64) PRIMARY KEY,
    kind           TEXT NOT NULL,                  -- 'route' | 'reconstruction' | 'geometry-digest' | ...
    processing_key VARCHAR(64) NOT NULL UNIQUE,    -- SHA-256 hex over canonical inputs
    input_hash     VARCHAR(64) NOT NULL,           -- SHA-256 hex over the raw input document
    status         TEXT NOT NULL DEFAULT 'pending',-- pending | running | done | failed | stale
    result_ref     TEXT,                           -- opaque pointer to the computed artifact
    result_hash    VARCHAR(64),                    -- SHA-256 hex of the result (integrity)
    error          TEXT,
    created_by     TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at   TIMESTAMPTZ,
    -- A stale row is retained (not deleted) so the invalidation history is
    -- auditable; a fresh row with the same key supersedes it.
    CONSTRAINT derived_jobs_status_ck
        CHECK (status IN ('pending', 'running', 'done', 'failed', 'stale'))
);

-- Recomputing the same key while a done row exists is the reuse path; while a
-- stale/failed row exists it is a recompute. The UNIQUE constraint on
-- processing_key is what makes the reuse atomic (ON CONFLICT ... DO UPDATE).
CREATE INDEX IF NOT EXISTS idx_derived_jobs_kind ON derived_jobs (kind, status);
CREATE INDEX IF NOT EXISTS idx_derived_jobs_created ON derived_jobs (created_at DESC);

-- Dependency edges: an edge (A -> B) means "A depends on B", so when B is
-- invalidated, A must become stale too. Stored as an adjacency list; the
-- invalidation walk is a recursive CTE over it.
CREATE TABLE IF NOT EXISTS derived_job_dependencies (
    job_id        VARCHAR(64) NOT NULL REFERENCES derived_jobs (id) ON DELETE CASCADE,
    depends_on_id VARCHAR(64) NOT NULL REFERENCES derived_jobs (id) ON DELETE CASCADE,
    PRIMARY KEY (job_id, depends_on_id),
    CONSTRAINT derived_job_no_self_loop CHECK (job_id <> depends_on_id)
);

CREATE INDEX IF NOT EXISTS idx_derived_deps_target
    ON derived_job_dependencies (depends_on_id);
