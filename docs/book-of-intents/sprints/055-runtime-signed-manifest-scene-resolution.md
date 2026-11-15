# Sprint 055: Runtime scene resolution grounded in signed manifests

**Date:** 2026-11-15 · **Status:** done, uncommitted (working tree)
**Gate:** doc + code; DB-backed tests executed against local PostgreSQL
(`DATABASE_URL=postgres://terra:terra@localhost/terra_dev`).

## Trigger

Sprint 054 "What is still missing" listed the signed-manifest runtime path as
open: scene resolution read mutable `world_manifests`/`manifest_objects` rows
and was not anchored to a publisher signature. CURRENT.md item 8 named it the
next unblocked code slice. No blocker applies (zero-SOL local work; B6 devnet
funding gates only live-cluster verification).

## Implementation

### Migration

- `migrations/0047_runtime_signed_manifest_binding.sql`: adds nullable
  `signed_content_address` to `world_manifests` (+ partial index), binding a
  runtime manifest to a verified `signed_manifests` envelope. Additive;
  manifests without a binding keep the legacy `manifest_objects` path. Applied
  to `terra_dev` and recorded in `_sqlx_migrations` (version 47); `sqlx::migrate!()`
  applies it on API startup.

### Backend (`api/src/routes/runtime.rs`)

- Extracted `load_verified_signed_manifest(state, address)`: fetches the stored
  envelope by content address, deserializes, re-verifies the Ed25519 signature,
  and rejects unless the recomputed address matches the key. Shared by
  `GET /manifests/signed/{address}` (deduplicated from its previous inline
  copy) and by scene resolution.
- `GET /runtime/scene/{manifest_id}` (via `resolve_scene_internal`):
  - If `signed_content_address` is set: parse the address, load + verify the
    envelope, build scene nodes from the signed payload. Fail closed — a bound
    manifest whose envelope is missing or no longer verifies is an error,
    never a silent fall back to mutable rows.
  - Otherwise: legacy read-only path over `manifest_objects` (unchanged
    behavior, `content_address: None`).
- Each `SceneNodeRow` now carries `content_address: Option<String>` — the
  object's SHA-256 content digest from the signed manifest when on the signed
  path — so clients can reject fetched bytes that do not match the
  publisher-signed manifest. Scene resolution remains read-only (no
  `scene_graphs`/`scene_nodes` writes on GET).

### Frontend (`terra-web/src/lib/runtime.ts`)

- `SceneNodeRow` gains `content_address: string | null`, mirroring the backend
  schema.

### Tests (DB-gated, skip without `DATABASE_URL`)

- `resolve_scene_grounded_in_signed_manifest`: stores a signed envelope, binds
  a `world_manifests` row to its content address, resolves the scene, and
  asserts node count, object id, component version, `content_address` (the
  object's digest, not the manifest's), validation state, and GeoJSON polygon
  spatial extent match the signed payload.
- `resolve_scene_bound_to_missing_envelope_errors`: a manifest bound to an
  unstored address fails with `NotFound` — fail-closed, no silent fallback.

## Verification

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | clean |
| `cargo check -p terra-api` | clean |
| `cargo clippy -- -D warnings` (workspace) | clean |
| `cargo test -p terra-api` (DATABASE_URL set) | **153 passed** |
| `tsc -b --noEmit` | clean |
| `vitest run` | **126 passed** |
| `pnpm build:check` | PASS (entry 305.8 kB gz ≤ 360; Cesium 1098.5 kB lazy ≤ 1200; total 1883.1 kB gz ≤ 1900) |
| eslint | 17 pre-existing errors, 7 warnings; none new |

## What is still missing

- **Live-cluster verification** of REQ-026/027/028 remains blocked by Q-001
  devnet funding (deployer wallet holds 0 SOL).
- `scene_graphs` / `scene_nodes` are written by nothing after GET was made
  read-only; a future explicit materialization endpoint (or removal of the
  tables) should decide their role (noted in Sprint 054).
- **Package UUID/content-address auto-wiring** so `fetchComponentArtifact` is
  automatic from scene nodes (Sprint 045 follow-up; partially addressed by
  `content_address` on nodes here).

## Book updates

`work/CURRENT.md`, `CHANGELOG.md`, `IMPLEMENTATION_STATUS.md` (B16, B17, E4),
`REQUIREMENTS_TRACEABILITY.md` (REQ-027, REQ-028 rows), this note.
