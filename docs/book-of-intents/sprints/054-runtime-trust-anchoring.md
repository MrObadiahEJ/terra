# Sprint 054: Runtime trust anchoring, data-plane addressing, CI repair

**Date:** 2026-11-15 · **Status:** implemented & committed (`1a35af6`); CI verification pending
**Local gates:** `cargo fmt --all -- --check` clean; `cargo check -p terra-api`
clean; `cargo test -p terra-api` **151 passed** (DB-backed tests skipped
locally); `cargo test -p terra-registry --lib` **154 passed**;
`cargo clippy -p terra-api -p terra-registry -- -D warnings` clean;
`tsc -b --noEmit` clean; `vitest run` **126 passed**; eslint on
`runtime.ts`/`runtime.test.ts` shows only the 8 pre-existing `no-explicit-any`
errors in `runtime.ts`.

## Trigger

1. CI was red from Sprint 048 onward. Run `43630959959` failed on a registry
   SDK assertion (`&&str == str`) and on migration `0046` (invalid `//` SQL
   comments), which prevented the API test job from reaching the test suite.
2. Sprint 042 tracked the gap: *"Client never verifies fetched bytes against
   signed manifest content address."* Sprint 052 claimed frontend verification,
   but it verified only the server-provided `x-content-sha256` header of the
   data-plane fallback.
3. Sprint 052 also conflated two different digests: it compared the canonical
   geometry digest (`terra-geom-v2`, a digest of canonicalized geometry) with
   the SHA-256 of raw artifact bytes (GLB / point cloud). That check would reject
   every legitimate artifact upload and had to be removed.
4. The working tree contained an unfinished version of this work: `runtime.ts`
   had a syntax error, the new test block was truncated, and `PUT .../artifact`
   bound the string path where a UUID was required.

## Implementation

### CI repair

- `migrations/0046_provider_proof_of_possession.sql`: `//` → `--` comments.
- `programs/terra_registry/src/lib.rs`: dereference the SDK schema name before
  comparing (`*n == "…"`).

### Artifact integrity and addressing (`components.rs`)

- Removed the invalid `geometry_digest == raw artifact SHA-256` check. RFC-015
  rule 1 is checked where the geometry document exists
  (`validate_with_geometry`), not at raw-artifact upload.
- Added `resolve_package_id`: the artifact endpoint accepts the human-readable
  `component_id` or the internal UUID on **both** PUT and GET. UUID-shaped
  identifiers always resolve as UUIDs, so a component_id cannot shadow another
  package.
- Private artifact authentication now signs the path actually requested
  (`/components/{id}/artifact`) rather than a rewritten UUID path.
- Upload tests now build a real signed request over the exact body bytes
  (`signed_upload_request`); the previous helper signed unrelated bytes and
  would have failed the body-hash check.

### Runtime asset retrieval (`runtime.rs`)

- `GET /runtime/assets/{component_id}/{component_version}` now returns up to
  five ranked replicas (`Json<Vec<AssetRetrieval>>`) instead of one.
- Each retrieval carries `content_address`: Terra's data-plane
  `component_artifacts.content_address` when present, otherwise the
  provider-declared `provider_replicas.checksum`. This is the SHA-256 of the raw
  artifact bytes, not the geometry digest.
- Scene resolution is read-only: it no longer inserts `scene_graphs` or
  `scene_nodes` rows on GET. The response is built from `manifest_objects`.
- Fixed the scene query to select `spatial_extent` (the struct required it but
  the query omitted it, so the endpoint previously failed at row decoding) and
  added it to `SceneNodeRow`, matching the frontend culling code.
- Added two DB-gated tests: scene resolution is read-only and returns
  `spatial_extent`; asset retrieval returns a replica with its content address.

### Client verification (`terra-web/src/lib/runtime.ts`)

- `fetchComponentAsset` ranks replicas, computes SHA-256 of the fetched bytes
  and compares it with `content_address`. Providers without a content address
  are skipped; tampered bytes fall through to the next replica and finally to
  the data-plane artifact endpoint.
- The client accepts either an array (new API) or a single object (older API).
- `fetchComponentArtifact` now **requires** `x-content-sha256`; a response
  without it is rejected rather than silently accepted.
- Tests cover verification, tamper fallback, missing-address skip,
  data-plane fallback, combined failure, missing-header rejection, and use
  dynamically computed hashes instead of the incorrect pre-computed value in
  the unfinished work.

### Protocol boundary (`components.rs`, ADR-024)

- `POST /components` rejects `simulated`, `designed`, `proposed`,
  `speculative` and AI-world provenance labels. RFC-015 §3 already defines the
  admissible evidence-backed values; creative content belongs in application
  overlays.
- Added `docs/app-overlay-anchoring.md` (overlay schema, anchoring rules,
  labeling, no trust inheritance) and ADR-024. The directive's requirement to
  support creative construction is preserved; it is simply not merged into the
  evidence-bearing component registry.

## Verification

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | clean |
| `cargo check -p terra-api` | clean |
| `cargo test -p terra-api` | **151 passed** (2 new DB-gated tests skip without `DATABASE_URL`) |
| `cargo test -p terra-registry --lib` | **154 passed** |
| `cargo clippy -p terra-api -p terra-registry -- -D warnings` | clean |
| `cargo test -p terra-api canonical_geometry` | 1 passed |
| `tsc -b --noEmit` | clean |
| `vitest run` | **126 passed** (was 120) |
| eslint (`runtime.ts`, `runtime.test.ts`) | 8 pre-existing `no-explicit-any` errors in `runtime.ts`; no new errors |

DB-backed CI job (PostgreSQL + `sqlx migrate run`) must still confirm the new
runtime tests and the repaired migration. CI run ID: pending.

## What is still missing

- **Signed-manifest runtime path.** Scene resolution still reads the mutable
  `world_manifests` tables, not stored signed envelopes. `content_address`
  makes provider bytes verifiable, but the scene itself is not yet anchored to
  a publisher signature (Sprint 042 gap remains partially open).
- **Scene materialization.** Removing writes from GET leaves `scene_graphs` /
  `scene_nodes` unused. A future explicit materialization endpoint (or removal
  of the tables) should decide their role.
- **Package UUID in scene nodes.** `fetchComponentAsset` receives
  `object_id`/`component_version_id`; exposing the package UUID/content address
  through scene nodes remains the Sprint 045 follow-up.
- **Live-cluster verification** of REQ-026/027/028 remains blocked by Q-001
  devnet funding.
- BPF/SBF suites were not run locally; CI remains the authority.

## Book updates

`work/CURRENT.md`, `CHANGELOG.md`, `SUMMARY.md`,
`ARCHITECTURAL_DECISIONS.md` (ADR-024), `sprints/052-proof-of-possession.md`
(correction note), `sprints/053-remove-terra-geom-v1.md` (CI follow-up), this
note, `docs/app-overlay-anchoring.md`.
