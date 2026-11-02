# Sprint 008 — AI provenance on geometry drafts (REQ-024 / R-014)

**Loop:** Research → Understand → Compare → Identify Gaps → Implement → Validate → Record
**Phase derived from evidence:** Test (validated) → Loop (this record)

## Research

- REQ-024 requires AI outputs to be distinguishable from verified results;
  the worker pipeline (migration 0031) labelled drafts by *method*
  (`capture-hull`, `parcel-footprint-fallback`) but recorded nothing about a
  model. RISK R-014 flagged this as partial.
- The worker itself is external to the API, so the API must accept the
  provenance the worker reports and refuse ambiguous combinations. There is
  no `workers` table to extend — the durable record is `geometry_drafts`.
- The Book's rule applies: provenance is *description*, never verification.
  An AI draft stays a draft until the anchor is confirmed and review accepts
  it (OBSERVATION ≠ EVIDENCE ≠ VERIFICATION).

## Plan

- Migration 0034: `provenance_kind` (`deterministic` | `ai-inference` |
  `ai-assisted`, default `deterministic`) plus `model_provider`,
  `model_name`, `model_version`, `model_confidence`.
- `job_complete` validates fail-closed *before* touching job state:
  - model fields on a `deterministic` draft → 400;
  - AI kind without `model_name` **and** `model_version` → 400;
  - confidence outside `[0,1]` (or non-finite) → 400;
  - provenance strings 1..=120 chars.
- DB constraints mirror the invariants (kind enum, confidence range, AI kind
  requires a model name) so a direct writer cannot bypass the API.
- No confidence value is interpreted as a protocol truth score; it is
  model-reported metadata returned verbatim.

## Build

- `api/migrations/0034_ai_provenance.sql`.
- `routes/reconstruction.rs`: `GeometryDraft` and `CompleteInput` carry the
  new fields; `SELECT_DRAFT` and the draft INSERT include them;
  `validate_provenance()` runs in `job_complete`.
- Existing drafts backfill to `deterministic` by column default.

## Test (evidence)

- `cargo test -p terra-api` → **128/128** (was 121; 7 new provenance tests:
  default deterministic, full AI metadata accepted, missing name/version,
  model metadata with deterministic kind, confidence bounds incl. NaN/∞,
  unknown kind, string bounds).
- `cargo fmt --check` clean; `cargo clippy -p terra-api -- -D warnings` clean.
- Live smoke (`/tmp/opencode/smoke_provenance.py`, Ed25519-signed) →
  **13/13**:
  - AI draft persisted `ai-inference` /
    `acme-vision/parcel-segmenter@2026.10-ckpt3` / confidence `0.82`, and
    `GET /jobs/{id}/draft` returned the same values;
  - the four ambiguous payloads each returned **400**;
  - a default draft stayed `deterministic` with null model fields.
  - Note: the dev DB held older queued jobs, so the smoke leases until it
    reaches its own job; that is queue behaviour, not a bug.
- Constraints verified directly with psql (rolled back): AI kind without
  `model_name` → `geometry_drafts_ai_has_model` violation; confidence `1.7`
  → `geometry_drafts_model_confidence_check` violation.

## Loop

- R-014 resolved; REQ-024 → status 1; B3/B10 registers updated; API baseline
  121 → 128.
- Follow-up (not required now): if AI-generated *model artifacts* (not just
  geometry drafts) become common, record the same provenance on
  `three_d_models`. Story replay does not consume drafts, so REQ-018 output
  is unaffected.
- Devnet funding remains the last step (founder directive); this work needed
  no SOL.
