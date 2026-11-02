-- AI provenance for geometry drafts (REQ-024 / RISK R-014).
--
-- A draft may be produced by a deterministic algorithm or by an AI model.
-- These columns record *what produced it* — they are description, not
-- verification. An AI draft is still a draft: it becomes a fact only after
-- the on-chain anchor is confirmed and the normal review path accepts it.
--
-- provenance_kind:
--   deterministic  — classical geometry (convex hull, footprint fallback, …)
--   ai-inference   — model output used directly
--   ai-assisted    — model output refined/validated by deterministic steps

ALTER TABLE geometry_drafts
    ADD COLUMN IF NOT EXISTS provenance_kind  TEXT NOT NULL DEFAULT 'deterministic',
    ADD COLUMN IF NOT EXISTS model_provider   TEXT,
    ADD COLUMN IF NOT EXISTS model_name       TEXT,
    ADD COLUMN IF NOT EXISTS model_version    TEXT,
    ADD COLUMN IF NOT EXISTS model_confidence DOUBLE PRECISION;

-- A confidence claim must be a probability, and AI kinds must name the model
-- (version is enforced in the API so the constraint stays backward-compatible
-- with any pre-existing rows).
ALTER TABLE geometry_drafts
    ADD CONSTRAINT geometry_drafts_provenance_kind_check
    CHECK (provenance_kind IN ('deterministic', 'ai-inference', 'ai-assisted'));

ALTER TABLE geometry_drafts
    ADD CONSTRAINT geometry_drafts_model_confidence_check
    CHECK (model_confidence IS NULL OR (model_confidence >= 0 AND model_confidence <= 1));

ALTER TABLE geometry_drafts
    ADD CONSTRAINT geometry_drafts_ai_has_model
    CHECK (provenance_kind = 'deterministic' OR model_name IS NOT NULL);
