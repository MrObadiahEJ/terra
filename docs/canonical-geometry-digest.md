# Canonical geometry digest (`terra-geom-v1`)

**Status:** implemented on `dev` (2026-10-16) — resolves [RFC-013 §9 open question 4](rfc-013-spatial-intelligence-pipeline.md#9-open-questions-phase-b)
**Reference implementation:** `terra-core/geo-engine/src/canonical.rs` (`terra-geo` crate)
**Consumers:** `terra-api` `POST /reconstruction/drafts` (`canonical_geometry_hash`), any validator or external service re-deriving the digest from the stored document
**Published vectors:** `terra-core/geo-engine/testdata/geometry_digest_v1.json`

---

## 1. Why this exists

`geometry_hash` is the identifier Terra anchors on-chain for a `GeometryVersion`
(RFC-013 §5, `append_geometry_version`). The hash is computed off-chain — by the
Rust worker, possibly also by a PostGIS draft, a photogrammetry service or a
validator re-deriving it — and must be reproducible by every party from the
stored GeoJSON document alone. A digest over raw document bytes does not satisfy
this: the same parcel encoded with a different key order, ring start, ring
orientation, coordinate precision or multi-geometry ordering would produce a
different hash, and two honest implementations would disagree on-chain.

`terra-geom-v1` defines a canonical form such that the digest is a pure function
of the **geometry**, independent of the JSON encoding.

> Before this change, `canonical_geometry_hash` was `hex(SHA-256(geojson_bytes))`
> over the raw `ST_AsGeoJSON` output — a hash of an encoding, not of a geometry.
> The name promised canonicalization the code did not perform; this document and
> `canonical.rs` close that gap.

## 2. Digest recipe

```
digest = SHA-256( "terra-geom-v1" ‖ canonical_geojson )
```

- `"terra-geom-v1"` is a domain-separation **and** version tag, hashed ahead of
  the document (mirroring the evidence-stack / `access_digest` construction in
  `geo-engine/src/reach.rs`). A future `terra-geom-v2` coexists without ambiguity
  because the tag is part of the preimage.
- `canonical_geojson` is the UTF-8 canonical document described in §3.
- The digest is returned as 32 bytes; `geometry_digest_hex` renders lowercase hex
  (64 characters) for `AnchorArgs.geometry_hash`.

## 3. Canonical form

Input is a GeoJSON **geometry**, or a `Feature` (its `geometry` member is used;
a `null` geometry is rejected). Supported types: `Point`, `MultiPoint`,
`LineString`, `MultiLineString`, `Polygon`, `MultiPolygon`.

### 3.1 CRS

Fixed to **EPSG:4326** (`x` = longitude, `y` = latitude). A `crs` member naming
anything other than 4326 / CRS84 is rejected; a missing member is assumed 4326
(RFC 7946 default). The canonical output never carries a `crs` member.

### 3.2 Coordinates

- **2D only.** A third ordinate (altitude) is rejected.
- Quantized to `1e-7` degrees (≈ 1.1 cm at the equator), **rounded
  half-away-from-zero**, stored as signed integers internally.
- Emitted with exactly **seven fractional digits**, fixed-point (never scientific
  notation). `-0.0000000` is normalized to `0.0000000`.

### 3.3 Rings

- Consecutive duplicate vertices are removed.
- The closing vertex is **implicit**: a ring is not closed in the canonical
  output (last vertex ≠ first).
- A ring with fewer than 3 distinct vertices, or zero area, is dropped as
  degenerate.

### 3.4 Orientation and starting vertex

Ring orientation is forced to RFC 7946 — exterior rings counter-clockwise, holes
clockwise — and each ring is rotated to start at its lexicographically smallest
`(lon, lat)` vertex. (Shoelace sign on the quantized integer ring.)

### 3.5 Ordering

- **Polygon:** shell first, then holes sorted ascending.
- **MultiPolygon / MultiLineString / MultiPoint:** children sorted ascending and
  de-duplicated.

### 3.6 Serialization

Canonical GeoJSON with fixed key order (`type` before `coordinates`) and no
insignificant whitespace:

```json
{"type":"Polygon","coordinates":[[[8.5000000,47.4000000],[8.6000000,47.4000000],[8.6000000,47.5000000],[8.5000000,47.5000000]]]}
```

## 4. Invariants

For geometries `A`, `B`:

1. **Encoding-independent.** Key order, whitespace, coordinate formatting,
   ring start vertex, ring orientation and multi-geometry ordering do not change
   the digest.
2. **Idempotent.** `canonicalize(canonicalize(A)) == canonicalize(A)`, and the
   digest of the canonical document equals that of the input.
3. **Sensitive.** Any real change to the geometry (a vertex moved by more than
   the quantization step, a shell/hole added, a different CRS) changes the
   digest.

These are asserted in `canonical.rs` tests and frozen against the vectors in
`testdata/geometry_digest_v1.json` (`published_vectors_match`).

## 5. Versioning

Change the tag (`terra-geom-v2`) for any change to §3 that would alter existing
digests — quantization step, orientation rule, ordering, key order, supported
dimensions. After such a change, refresh the vectors:

```sh
cd terra-core
cargo test -p terra-geo regenerate_vectors -- --ignored --nocapture
```

and update the `expected` digests in `testdata/geometry_digest_v1.json`. The file
is the cross-implementation contract: a non-Rust implementation is correct when
it reproduces every vector.

## 6. Open items

- `MultiPoint` is canonicalized (sorted, de-duplicated) but has no published
  vector yet; add one when a producer emits it.
- Precision is a fixed `1e-7` for all geometry types; if survey-grade input needs
  finer resolution, that is a `v2` decision, not a local one.
