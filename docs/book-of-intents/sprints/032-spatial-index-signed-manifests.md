# Sprint 032 — Spatial Index Decision and Signed Content-Addressed Manifests

**Date:** 2026-11-13 · **Status:** done, committed (`db22de0`, `40b6633`)
**Gate:** `cargo check -p terra-geo`; `cargo test -p terra-geo --lib` —
**48 passed, 2 ignored** (15 new tests); `cargo fmt -p terra-geo -- --check` clean

## Trigger

Chapter 12 §12.4 left the hierarchical spatial index as an open decision
(“quadtree/S2/H3 decision pending”), and §12.8 listed signed, content-addressed
manifest retrieval as missing. `work/CURRENT.md` placed this after the
founder/devnet items because it is pure library work that needs no SOL, no
cluster, and no frontend build.

The goal of this sprint was the smallest honest foundation: decide the
control-plane partition structure, implement it, and make manifest identity and
authenticity real — not just documented.

## Decision (ADR-020)

- **Control-plane 2D discovery:** a deterministic **quadtree over EPSG:4326**.
  Cell IDs are stable strings `q/{level}/{x}/{y}`; level 0 is the world and
  level *n* has `2^n × 2^n` cells. `x` grows east, `y` grows north.
- **S2/H3:** not adopted for control-plane partitions now. They remain the
  preferred adapter later for streaming visualization tiles; adding them must
  not change stored manifest references because cell IDs stay opaque strings.
- **3D lookup:** octree/BVH remains the structure for buildings, point clouds,
  and component-level culling. The quadtree is not a 3D index.
- **Manifest identity:** SHA-256 of the exact serialized payload bytes
  (`sha256:<hex>`). Retrieval can ask for that address, hash the received bytes,
  then parse.
- **Manifest authenticity:** Ed25519 signature over
  `terra.manifest.signature.v1\0 || payload_digest`, with the publisher key
  carried as a 32-byte hex string. Signing the digest avoids a JSON
  canonicalization dependency while still making any payload change invalidate
  the signature.
- **Separation preserved:** geographic partitions, spatial objects, storage
  chunks, visualization tiles, and assembly manifests remain distinct concepts.

Recorded in `ARCHITECTURAL_DECISIONS.md` as ADR-020.

## Implementation

### `geo-engine/src/spatial_index.rs`

- `GeoBounds` (EPSG:4326, validated, antimeridian crossing must be split by the
  caller) with contain/intersect/enclose helpers.
- `QuadCell` with `root`, `at`, `from_point`, `parent`, `children`, `bounds`,
  `contains`, `encloses`, `intersects`, `Display`, `FromStr`, and string
  serialization. Maximum level 30 (~3.7 cm at the equator) keeps `x`/`y` in
  `u32`.
- `covering_cells(bounds, level)` returns intersecting cells in south-to-north,
  west-to-east order and rejects queries above 65,536 cells.
- Boundary rule is deliberately conservative: point lookup uses `floor` and
  clamps `180°`/`90°` into the last cell; bbox covering includes cells that only
  touch the query edge, so an object exactly on a boundary cannot be missed.

### `geo-engine/src/manifest.rs`

- `ContentAddress` — SHA-256 parse/format (`sha256:` prefix) and
  `verify_bytes()` for content-addressed retrieval.
- `ManifestPayload` — schema `terra.manifest/v1`, region ID/name, quadtree
  region cell, monotonic version, optional parent and neighbor manifest
  addresses, object references, optional description, and Unix timestamp.
- `ManifestObject` — object ID, component version, optional assembly version,
  component content digest, data-plane storage reference, bounds, available LODs
  (0–4), and `ValidationState` (`pending` / `technically_valid` /
  `independently_valid` / `rejected`).
- Validation rejects: wrong schema, empty region ID/name, version 0, empty
  object/component/storage fields, duplicate object IDs, duplicate neighbor
  addresses, duplicate or out-of-range LODs, invalid bounds, and object bounds
  that fall outside the manifest's region cell.
- `SignedManifest` — `sign`, `verify`, `content_address`, `to_json`,
  `from_json`; parsing through `from_json` verifies before returning.
- `ed25519-dalek` 2.2 was added to `terra-geo`; it was already in the workspace
  lockfile and needed by the API.

`component_package.rs` was rustfmt'd in a separate formatting-only commit
(`40b6633`).

## Verification

| Check | Result |
| --- | --- |
| `cargo check -p terra-geo` | passes (1 m 13 s cold-ish; ed25519/curve deps were already in the lockfile) |
| `cargo test -p terra-geo --lib` | **48 passed, 2 ignored** — 15 new: 7 spatial-index, 8 manifest |
| `cargo fmt -p terra-geo -- --check` | clean |

New tests cover ID roundtrips, edge clamping, parent/child symmetry, covering
order and limits, invalid inputs, signing/verification, tampered payloads,
wrong publisher keys, content-address verification, schema/region checks,
duplicate objects/neighbors/LODs, out-of-cell objects, and JSON roundtrips.

Not run: BPF/SBF builds, full-workspace tests, API tests, frontend gates. None
are needed for a pure `terra-geo` library slice, and this machine remains the
constraint.

## What is still missing

- The API still stores manifests as mutable PostgreSQL rows
  (`world_manifests`/`manifest_objects`) and does not serve signed envelopes or
  content addresses. Wiring the library into `routes/manifests.rs` and a
  retrieval endpoint is the next slice.
- The scene request path does not yet derive quadtree cells from camera bounds;
  `routes/spatial.rs` still uses the existing PostGIS proximity query.
- No live-cluster manifest version transition or multi-provider content
  retrieval (B6 devnet funding blocks it).
- S2/H3 tile adapters and 3D octree/BVH are intentionally deferred.

## Book updates

ADR-020 added; Chapter 12 §12.1/§12.4/§12.8/§12.13 and Chapter 13 §13.8 updated
to separate the implemented library foundation from the still-missing service
integration; REQ-027 evidence extended; `IMPLEMENTATION_STATUS.md` adds C4/C5 and
keeps B16 at `2 – IMPLEMENTED, NOT FULLY VERIFIED`; `work/CURRENT.md`,
`CHANGELOG.md`, `SUMMARY.md` and `BOOK_OF_INTENTS.md` updated.
