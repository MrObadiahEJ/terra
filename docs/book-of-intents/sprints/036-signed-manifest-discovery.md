# Sprint 036 — Discover Persisted Signed Manifests by Region and Cell

**Date:** 2026-11-13 · **Status:** done, committed (`638912a`)
**Gate:** `cargo fmt --all -- --check`; `cargo clippy -- -D warnings`;
`cargo test -p terra-api` — **143 passed**

## Trigger

Sprint 035 stored signed `terra.manifest/v1` envelopes and made them
retrievable by content address, but a client had no way to find the right
address in the first place. The `signed_manifests` table already carried
`region_id`, `region_cell`, `version`, and `publisher` columns with indexes;
Sprint 035's "still missing" list named the list endpoint. Chapter 12 §12.8 and
`work/CURRENT.md` tracked the same gap.

## Implementation

`GET /manifests/signed?region_id=&cell=&limit=`

- Filters are optional and composable. `region_id` matches the stored region
  string; `cell` matches an **exact** `q/level/x/y` quadtree cell.
- The `cell` value is parsed with `QuadCell::from_str` before the query, so a
  malformed cell is `400 Bad Request` rather than an empty result. This mirrors
  the validation already used by `GET /spatial/cell`/`/spatial/cover`.
- `limit` defaults to 20 and is clamped to 1–100.
- Ordering is deterministic: `created_at DESC, content_address ASC`.
- The response is metadata only — `content_address`, `publisher`, `region_id`,
  `region_name`, `version`, `region_cell`, `created_at`. Envelope bodies are
  deliberately not returned; callers follow the content address to
  `GET /manifests/signed/{address}`, which re-verifies before returning.

The handler is `list_signed_manifests` in `api/src/routes/manifests.rs`. No
migration was required: Sprint 035 already added the region and cell indexes.

## Verification

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | clean |
| `cargo clippy -- -D warnings` | clean workspace-wide |
| `cargo test -p terra-api` | **143 passed** (1 new unit test) |
| DB-backed roundtrip | now also asserts the stored manifest appears in the region/cell listing (runs in CI under `DATABASE_URL`) |

`list_endpoint_rejects_malformed_cell_before_querying` runs without a database.
The listing assertion was added inside the existing env-gated roundtrip test, so
it exercises real PostgreSQL in CI and skips cleanly on a machine without
`DATABASE_URL`. No BPF/SBF build was run locally.

## What is still missing

- Pagination beyond a clamped `limit` (no cursor/offset yet).
- Prefix/parent-cell discovery (an exact-cell match only; clients can already
  compute covering cells with `GET /spatial/cover`).
- Content-addressed storage for **data-plane component bytes**.
- Terra-operated publisher key custody (ADR-021).
- Scene-path quadtree use, frustum/occlusion culling, live-cluster verification
  (B6 devnet funding).

## Book updates

Chapter 12 (topology, §12.8, file table, closing status), Chapter 13 file table,
Chapter 14 implementation and no-funding lists, REQ-027,
`IMPLEMENTATION_STATUS.md` (B16/B20/C5), `ROADMAP.md`, `work/CURRENT.md`,
`CHANGELOG.md`, `SUMMARY.md`, `BOOK_OF_INTENTS.md`, ADR-021 evidence, and the
Sprint 035 follow-up note.