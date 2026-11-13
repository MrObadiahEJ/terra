# Sprint 033 — API Discovery Cells and Signed Manifest Verification

**Date:** 2026-11-13 · **Status:** done, committed (`496ec49`)
**Gate:** `cargo check -p terra-api`; `cargo test -p terra-api endpoint` —
**6 passed** (133 filtered; new tests only); rustfmt clean on changed files

## Trigger

Sprint 032 delivered the quadtree and signed-manifest libraries but left them
unwired. Chapter 12 §12.13 and `work/CURRENT.md` listed API integration as the
next unblocked slice. This sprint exposes the two safest, stateless surfaces
first: cell lookup and manifest verification. No database migration, signing
key custody, or storage change was attempted.

## Endpoints

### `GET /spatial/cell?lon=&lat=&level=`

Returns the quadtree cell containing a point:

```json
{
  "id": "q/8/…/…",
  "level": 8,
  "x": …,
  "y": …,
  "bounds": { "min_lon": …, "min_lat": …, "max_lon": …, "max_lat": … }
}
```

- Level defaults to 12; out-of-range coordinates/levels return `400`.
- Uses `terra_geo::QuadCell::from_point`; no database access, so it works with
  an empty geo mirror.

### `GET /spatial/cover?min_lon=&min_lat=&max_lon=&max_lat=&level=`

Returns `{ level, count, cells }` in south-to-north, west-to-east order.

- Level defaults to 10; invalid bounds and covering requests above the
  library's 65,536-cell cap return `400`.

### `POST /manifests/verify`

Accepts a `terra.manifest/v1` envelope (`SignedManifest` JSON) and returns:

```json
{
  "valid": true,
  "content_address": "sha256:…",
  "region_id": "…",
  "version": 1,
  "object_count": 2
}
```

Verification covers payload invariants, publisher key parsing, Ed25519
signature, and the canonical payload content address. It persists nothing and
does not require an API signing key. A tampered payload returns `400`.

## Verification

| Check | Result |
| --- | --- |
| `cargo check -p terra-api` | passes in 5 m 08 s (warm target; existing warnings unchanged) |
| `cargo test -p terra-api endpoint` | **6 passed, 133 filtered** — 4 cell/cover tests + 2 manifest-verification tests; test build 6 m 10 s |
| `cargo test -p terra-geo --lib` | **48 passed, 2 ignored** (unchanged) |
| `rustfmt --check` on changed files | clean |

The targeted API run intentionally avoided the full 133-test suite and any
database-backed test; the machine is the constraint. No integration test hits
a live cluster.

## What is still missing

- The API does not **sign** or persist signed envelopes; `world_manifests`
  remains mutable PostgreSQL rows with no publisher/signature/content-address
  columns.
- No content-addressed storage/retrieval endpoint. `POST /manifests/verify`
  proves an envelope but does not fetch or store the referenced components.
- `GET /spatial/cell`/`cover` are discovery primitives; the scene request path
  and `routes/manifests.rs` do not derive cells from camera/query bounds yet.
- Signing-key custody is an operational decision that should not be improvised
  in a request handler.
- No live-cluster manifest transition, multi-provider retrieval, or failover
  (B6 devnet funding blocks it).

## Book updates

`work/CURRENT.md` next-up item narrowed to signed persistence + content-addressed
storage + scene-path wiring; Chapter 12 §12.8/§12.13, Chapter 13 §13.3/§13.8,
Chapter 14, REQ-027, `IMPLEMENTATION_STATUS.md` (B16/B20/C4/C5), `CHANGELOG.md`,
`SUMMARY.md` and `BOOK_OF_INTENTS.md` updated.
