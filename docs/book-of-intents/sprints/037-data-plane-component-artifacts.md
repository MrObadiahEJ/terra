# Sprint 037 — Data-Plane Component Artifact Storage

**Date:** 2026-11-13 · **Status:** done, uncommitted (`components.rs` + `storage.rs` + `runtime.ts` + `runtime.test.ts` + migration `0045_component_artifacts.sql`)
**Gate:** `cargo fmt --all -- --check`; `cargo clippy -- -D warnings`;
`cargo test -p terra-api` — **147 passed**; `cargo test -p terra-geo --lib` — **48 passed, 2 ignored**;
`tsc -b --noEmit` clean; vitest **86/86**; `pnpm build:check` PASS (entry 300.2 / initial 305.5 / Cesium 1098.5 / total 1866.9 kB gz).

## Trigger

Sprint 036 tracked "Content-addressed storage for data-plane component bytes" and
"derive quadtree cells from camera bounds in scene requests" as the next local
work (work/CURRENT.md item 4). Chapter 13 §13.8 and `work/CURRENT.md` named the
same gap: component packages are submitted and listed, but their actual mesh/point
cloud bytes had no storage path — `geometry_artifact` was just a URI. This sprint
closes that gap with content-addressed artifact storage, self-verifying retrieval,
and the frontend fallback that pulls canonical bytes from the API data plane.

## Implementation

**Backend — migration `0045_component_artifacts.sql`**

- `component_artifacts(component_package_id PK → component_packages, content_address, storage_ref, storage_backend, content_type, size_bytes)` keyed by component package.
- Index on `content_address` for content-addressed lookup.

**Backend — `api/src/routes/components.rs`**

- `PUT /components/{id}/artifact` — raw body upload. Computes the SHA-256 content address of the raw bytes; optional `X-Content-SHA256` header is normalized (`sha256:` prefix) and checked before storage; identical bytes are idempotent, different bytes → `409 Conflict` ("packages are immutable"); content-type + size + storage ref recorded.
- `GET /components/{id}/artifact` — enforces component privacy access, loads bytes by `storage_ref`, re-hashes and rejects if the re-hashed address differs from the recorded one; public components get `ETAG`/`x-content-sha256`/immutable-cache headers.
- Bug fixes exposed by the new DB-backed tests:
  - `submit_component` INSERT had 35 columns but 36 `$N` placeholders (leftover from migration 0041's `component_version` addition) → removed the stray `$36`; `component_version` now takes its DEFAULT `'v1'`.
  - `creation_timestamp` was bound as `String` into a `TIMESTAMPTZ` column → parse the RFC3339 string to `DateTime<Utc>` with a 400 on bad input.
  - `dependencies`/`overlap_ids` (`Option<Vec<String>>`) were bound as `NULL` into `TEXT[] NOT NULL` columns → coerce `None` to `[]`.

**Backend — `api/src/storage.rs`**

- `StorageBackend::store_content_addressed` — stores bytes under `content/sha256/aa/bb/<hex>` (idempotent, same bytes → same ref); 2 unit tests.

**Frontend — `src/lib/runtime.ts`**

- `fetchComponentArtifact(componentId)` — `GET /components/{id}/artifact`, returns `{ data, contentType, contentAddress }`, re-verifies the `x-content-sha256` header against the fetched bytes and throws on digest mismatch.
- `fetchComponentAsset` now falls back to the data-plane artifact endpoint when provider retrieval fails, so components are served from the canonical content-addressed store when no provider replica is reachable.

**Frontend — `src/lib/runtime.test.ts`**: 3 new tests — content-address verification roundtrip, digest-mismatch rejection, non-OK response handling.

## Verification

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | clean |
| `cargo clippy -- -D warnings` | clean workspace-wide |
| `cargo test -p terra-api` | **147 passed** (2 DB-backed artifact roundtrip + 2 storage unit tests; 2 manifest roundtrip) |
| `cargo test -p terra-geo --lib` | **48 passed, 2 ignored** |
| `tsc -b --noEmit` | clean |
| `vitest run` | **86/86** (was 83/83) |
| `eslint` | no new errors (runtime.ts carries 8 pre-existing `no-explicit-any` errors, unchanged) |
| `pnpm build:check` | PASS (entry 300.2 / initial 305.5 / Cesium 1098.5 / total 1866.9 kB gz) |

No `DATABASE_URL` needed for the API unit tests (they skip); DB-backed component-artifact tests ran against local PostgreSQL at `terra_dev`. No BPF/SBF build was run locally.

## What is still missing

- Frontend does not yet expose the component package UUID through the scene graph, so `fetchComponentArtifact` is a manual fallback rather than automatic from `GlobePage` scene nodes (scene nodes carry `object_id`/`component_version_id`, not the package `id`).
- Scene-path quadtree use, frustum/occlusion culling, screen-space-error LOD (Chapter 13 §13.8), live-cluster verification (B6 devnet funding).
- Terra-operated publisher key custody (ADR-021); pagination beyond clamped `limit` on signed-manifest listing.

## Book updates

`IMPLEMENTATION_STATUS.md` (B15 split into B15 + B15a artifact storage, counts → 147/147, D12 → 86/86, baselines table → 147/48/86/86, E4 → sprints 001–037, E5 gap text), `CHANGELOG.md`, `SUMMARY.md`, `ROADMAP.md` (data-plane bytes note under Stage 4 item), `work/CURRENT.md` (item 4 now lists this sprint as in progress/done), this note.