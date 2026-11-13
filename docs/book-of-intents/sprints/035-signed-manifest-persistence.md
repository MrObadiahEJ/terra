# Sprint 035 — Persist and Retrieve Signed Manifests by Content Address

**Date:** 2026-11-13 · **Status:** done, committed (`795c60d`)
**Gate:** `cargo fmt --all -- --check`; `cargo clippy -- -D warnings`;
`cargo test -p terra-api` — **141 passed** (2 new)

## Trigger

Sprint 032 created the signed manifest format and Sprint 033 exposed stateless
verification, but the API still could not keep a verified envelope or fetch one
by digest. Chapter 12 §12.8 and `work/CURRENT.md` listed signed persistence and
content-addressed retrieval as the next unblocked slice. ADR-020 had already
fixed identity (payload SHA-256) and authenticity (Ed25519), so no protocol
decision was reopened.

## Decision (ADR-021)

- The API accepts **pre-signed** envelopes. It holds no publisher key; signing
  stays external (offline signing or a future KMS/HSM-backed publisher service).
- Storage is keyed by `(content_address, publisher)`. Identity is the payload
  digest, so one payload may carry multiple publisher attestations without
  rewriting its address.
- Retrieval re-parses and re-verifies the stored envelope; a digest mismatch is
  a server error rather than a client error.
- Mutable `world_manifests` / `manifest_objects` stay unchanged for the existing
  CRUD path in this sprint.

## Implementation

### Migration `0044_signed_manifests.sql`

`signed_manifests(content_address, publisher, region_id, region_name, version,
region_cell, signature, envelope JSONB, created_at)` with a composite primary
key and indexes on `(region_id, version)` and `region_cell`. The migration is
additive.

### `POST /manifests/signed`

Verifies the envelope (payload invariants, publisher key, signature, canonical
digest), serializes it to JSONB, and inserts with
`ON CONFLICT (content_address, publisher) DO NOTHING`. Returns `201` on first
store and `200` on an idempotent repeat, with the digest, publisher, region,
and version.

### `GET /manifests/signed/{address}`

Parses the `sha256:<64 hex>` path segment, fetches the deterministic first
publisher by address, deserializes the envelope, recomputes the digest, verifies
the signature, and returns the envelope. Malformed addresses are `400`; a
missing artifact is `404`; a stored artifact that fails verification is `500`.

The API does not require a signing key for either endpoint.

## Verification

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | clean |
| `cargo clippy -- -D warnings` | clean workspace-wide |
| `cargo test -p terra-api` | **141 passed** (2 new: tampered store rejected before insert; malformed address rejected) |
| `cargo test -p terra-geo --lib` | **48 passed, 2 ignored** |

The DB-backed store/retrieve roundtrip is not exercised locally because no
PostgreSQL instance is available on this machine. CI applies migrations before
`cargo test -p terra-api`, so the migration syntax is checked there; the unit
tests cover the pre-database rejection paths. No BPF/SBF build was run locally.

## What is still missing

- A DB-backed integration test for the idempotent store and retrieval path
  (needs a test database or a transactional test fixture).
- Content-addressed storage for **component bytes** (the manifest references
  storage URLs; ADR-020's `ContentAddress::verify_bytes` is not yet used on the
  data plane).
- Discovery by region/cell over stored signed manifests (the columns exist, but
  no list endpoint is exposed).
- Terra-operated publisher key custody and rotation (ADR-021 defers this).
- Scene-path quadtree use, frustum/occlusion culling, and live-cluster
  verification (B6 devnet funding).

## Book updates

ADR-021 added; Chapter 12 §12.8/§12.13, Chapter 13 §13.8, Chapter 14, REQ-027,
`IMPLEMENTATION_STATUS.md` (B16/B20/C5), `work/CURRENT.md`, `CHANGELOG.md`,
`SUMMARY.md`, and `BOOK_OF_INTENTS.md` updated.
