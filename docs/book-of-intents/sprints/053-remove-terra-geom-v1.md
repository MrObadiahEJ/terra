# Sprint 053: Remove `terra-geom-v1`, keep only `terra-geom-v2`

**Date:** 2026-11-14 · **Status:** done & committed (`9f05f83`)
**Gate (as committed):** `cargo check --workspace` clean; `cargo clippy --workspace -- -D warnings` clean; `cargo fmt --all --check` clean; `cargo test -p terra-geo --lib` 42 passed

## Trigger

ADR-016 left `terra-geom-v1` frozen for existing hashes and deferred an
integrate-or-deprecate decision. Every live consumer already used the 3D
`terra-geom-v2` digest, so maintaining two canonicalizers invited accidental
v1 use in new code.

## Implementation

- Deleted `terra-core/geo-engine/src/canonical.rs` (v1, 574 lines) and its
  module re-exports from `geo-engine/src/lib.rs`.
- `sdk_meta` now declares only `terra-geom-v2`.
- `reconstruction.rs` uses `geometry_digest_hex_v2`; removed the v1 comparison
  test from `canonical3d.rs`.
- RFC-015, RFC-013, `canonical-geometry-digest.md`, `IMPLEMENTATION_STATUS.md`,
  `README.md` and `terra-core/README.md` updated to v2 only.
- Migration `0035` left untouched: applied migrations are historical records.

## Verification

The commit gate covered the workspace build, clippy, formatting and the
`terra-geo` library. The first CI run after the push (`43630959959`) exposed two
pre-existing/latent failures rather than v1 regressions:

1. `.github` CI's registry SDK assertion compared `&&str == str`
   (`programs/terra_registry/src/lib.rs:8269`).
2. Migration `0046_provider_proof_of_possession.sql` used `//` comments, which
   PostgreSQL rejected before the API tests could run.

Both are fixed in Sprint 054; v1 removal itself is complete.

## What is still missing

- None for v1 removal. Old v1 digests remain valid only as historical on-chain
  values; no code path can recompute them.
- Sprint 054 additionally aligns the API's canonical-geometry test input with
  the v2 document shape (CRS, elevation reference, precision, 3D ordinates).

## Book updates

`IMPLEMENTATION_STATUS.md`, `REQUIREMENTS_TRACEABILITY.md`,
`ARCHITECTURAL_DECISIONS.md` (ADR-016 status), `CHANGELOG.md`, `SUMMARY.md`,
`README.md`, `terra-core/README.md`, this note.
