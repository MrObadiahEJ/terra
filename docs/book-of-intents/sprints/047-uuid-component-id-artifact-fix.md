# Sprint 047: Fix UUID/component_id mismatch in artifact retrieval

**Date:** 2026-11-13 · **Status:** done & committed (`39d8728`)
**Gate:** `cargo check -p terra-api` clean; `cargo clippy -p terra-api -- -D warnings` clean; `cargo fmt --check` clean; `tsc -b --noEmit` clean; vitest 26/26.

## Trigger

work/CURRENT.md item 3 (RFC-016 Phase 2+ code) lists the UUID/component_id
fallback mismatch as unblocked agent work. The frontend `fetchComponentArtifact(componentId)`
passes a TEXT `component_id` (e.g. `parcel-1`) to `GET /components/{id}/artifact`,
but the backend route declared `Path(id): Path<Uuid>` — so the fallback path
in `fetchComponentAsset` would 400 on every call, making the data-plane
artifact fallback unreachable.

## Implementation

**Backend — `terra-core/api/src/routes/components.rs`:**
- Changed `get_component_artifact` signature from `Path(id): Path<Uuid>` to
  `Path(id): Path<String>`.
- When the path segment parses as a UUID, it is used directly.
- When it is not a UUID, it is treated as a `component_id` text and resolved
  to the package UUID via `SELECT id FROM component_packages WHERE component_id = $1`.
- The resolved UUID is then used for `fetch_component`, `fetch_artifact`, and
  error messages (the original text is preserved in the not-found error).
- Added test `artifact_retrieval_by_component_id_text` that submits a
  component, uploads an artifact, and retrieves it by `component_id` text.

**Frontend — `terra-web/src/lib/runtime.test.ts`:**
- `fetchComponentArtifact` test now asserts the fetch URL uses the text
  `component_id` as-is (`/components/comp-001/artifact`), not a UUID.
- No runtime.ts source change needed — `fetchComponentArtifact` already passed
  `componentId` text; the backend now accepts it.

## Verification

| Check | Result |
|---|---|
| `cargo check -p terra-api` | clean |
| `cargo clippy -p terra-api -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| `tsc -b --noEmit` | clean |
| `vitest run src/lib/runtime.test.ts` | 26/26 |

## Remaining RFC-016 Phase 2+ code items (unblocked agent work)

- `SecurityRequest.subject` still `Pubkey::default()` — fill via instruction param
- `PrivacyState` writes on finalize
- Provider proof-of-possession audits
- Client-side digest verification vs signed manifest (already wired in
  `fetchComponentArtifact`)
- `POST /components` auth
- `complete_assignment` auth

These remaining items are backend (Rust) work; some require heavier builds.
