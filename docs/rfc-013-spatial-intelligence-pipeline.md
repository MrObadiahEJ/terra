# RFC-013 — Spatial intelligence pipeline (Vision Stage 3)

- **Status:** Phase A implemented on `dev` (2026-09-28); **Phase B implemented (2026-10-01)** — Stage 3 marked 🟡 In progress
- **Depends on:** [RFC-012](rfc-012-global-physical-digital-trust-architecture.md) (architecture contract: §5 entity catalog, Design Rules 9 & 10, core invariant §0)
- **Implements:** `SpatialAsset`, `GeometryVersion` (Phase A) · evidence-anchored append, elevation provenance, validator-quorum verification (Phase B) · plans `ThreeDModel`, `SpatialSnapshot` (Phase C — already named in RFC-012 §5)
- **Related:** [`docs/VISION.md`](VISION.md) Domain 5 (2D → 3D → 4D) and the stage map §13

---

## 0. Core invariant (inherited, unchanged)

RFC-012 §0 applies verbatim: **AI never gets unilateral authority to turn an
observation into canonical ownership.** Geometry follows the same chain —
`observation → evidence → validation → consensus → state transition`. A
segmentation model, photogrammetry run or LiDAR pass may *anchor a claim*
(a geometry hash + provenance); only a registered validator's verification
turns that claim into a verified fact, and rights still move through the
existing validation flow. This RFC never short-circuits the evidence stack.

## 1. Why

The parcel baseline is 2D (`Parcel.geometry_hash`), but the physical world is
3D and changes over time: terrain, buildings, floors, air rights, underground
structures. Vision Stage 3 grows the on-chain model from a flat parcel to a
**`SpatialAsset` with X, Y, Z and TIME** — versioned geometry history whose
hashes anchor what the off-chain GIS stack already knows (PostGIS polygons,
elevation, 3D models).

## 2. Target pipeline

```
raw observation          off-chain GIS / AI                 on-chain anchor
─────────────────        ─────────────────────────          ────────────────────────
device capture      →    segmentation / photogrammetry  →   GeometryVersion (hash,
(GNSS, drone, LiDAR,     / LiDAR processing, PostGIS        source, dimension,
satellite — Phase 9)     geometry, elevation raster         storage_reference)
                                 │                                   │
                                 ▼                                   ▼
                         canonical GeoJSON digest            validator verify
                         (= geometry_hash)                   (claim → fact)
                                                                 │
                                                                 ▼
                                                     SpatialAsset history (4D)
                                                     "what did the geometry look
                                                      like at time T?"
```

**Design Rule 10 (RFC-012) is the split:** geometry, rasters and 3D models
stay **off-chain**; the chain stores only `geometry_hash`, provenance codes,
version indexes and `storage_reference` pointers.

## 3. Entities (Phases A + B, delivered)

### `SpatialAsset`

| | |
|---|---|
| PDA | `["spatial_asset", parcel]` — exactly one per `Parcel` |
| Fields | `parcel`, `authority` (init registrar), `dimensionality` (0=2D, 1=2.5D, 2=3D), `elevation_min_mm`/`elevation_max_mm` (i32, may be negative), `geometry_version_count` (append cursor), `latest_geometry` (hash), timestamps |
| Creation | `init_spatial_asset(dimensionality, elevation_min_mm, elevation_max_mm)` — **permissionless** (rent-paid anchor; grants no rights), parcel must exist |
| Events | `SpatialAssetCreated` |

### `GeometryVersion`

| | |
|---|---|
| PDA | `["geometry_version", asset, version_le_bytes(u32)]` — index = `asset.geometry_version_count` at append time |
| Fields | Phase A: `asset`, `parcel`, `version`, `geometry_hash`, `source`, `dimension`, `storage_reference` (≤128), `submitted_by`, `submitted_at`, `verified`, `verified_by`, `verified_at`. **Phase B:** `evidence_manifest` (Pubkey, `Pubkey::default()` = plain append), `elevation_min_mm`/`elevation_max_mm`, `elevation_source`, `required` (quorum threshold snapshot), `attestors` (`[Pubkey; 8]`), `attest_count` |
| Creation | `append_geometry_version(geometry_hash, source, dimension, storage_reference, elevation_min_mm, elevation_max_mm, elevation_source)` — **permissionless** rent-paid append (exactly the evidence-manifest artifact model: an anchored *claim*); append-only, immutable, no delete path. **Phase B:** `append_geometry_version_from_evidence(…same args)` anchors the version *to* an `EvidenceManifest` (task must be open, manifest non-empty) with no seed instruction args — the manifest/task PDAs are self-derived from the manifest's own fields (AddEvidenceArtifact precedent) |
| Verification | **Phase B:** `verify_geometry_version()` — registered validator signer (`profile.wallet == signer`), **quorum attestation**: each validator appends to `attestors` (dedup → `GeometryAlreadyAttested`); the version flips `verified` when `attest_count >= required`. `required` is snapshotted at append time from the global `QuorumConfig` `(parcel_type=0, region=[0,0])` passed in `remaining_accounts` (same lookup as `create_claim`), falling back to `DEFAULT_GEOMETRY_QUORUM = 2` |
| Events | `GeometryVersionAppended`, `GeometryVersionAttested` (Phase B), `GeometryVersionVerified` |

Append-only discipline: the version index lives in the PDA seeds and is read
from the asset cursor, so a stale or forged index fails Anchor seed
validation; `MAX_GEOMETRY_VERSIONS = 64` bounds state growth (mirrors
`MAX_MANIFEST_ARTIFACTS`).

## 4. Enums & limits

```text
spatial_dimension: D2=0, D2_5=1, D3=2 (MAX 2)
geometry_source:   MANUAL=0, SURVEY=1, DRONE_PHOTO=2, SATELLITE=3,
                   LIDAR=4, PHOTOGRAMMETRY=5, AI_SEGMENTATION=6, API_IMPORT=7
elevation_source (Phase B): NONE=0, SURVEY=1, DEM=2, LIDAR=3,
                   PHOTOGRAMMETRY=4, MANUAL=5 (MAX 5)
MAX_GEOMETRY_VERSIONS = 64        storage_reference ≤ 128 chars (shared with evidence)
DEFAULT_GEOMETRY_QUORUM = 2       MAX_VALIDATORS = 8 (attestors array)
```

Guards: version `dimension` must be ≤ the asset's `dimensionality`
(a 3D version may not attach to a 2D asset); elevation envelope must be
non-inverted (`min ≤ max`); geometry hash must be non-zero (canonical digest).

**Phase B elevation invariants** (all append paths):
1. `elevation_source` code ≤ `MAX` (else `InvalidElevationSource`);
2. envelope non-inverted (else `InvalidElevationRange`);
3. `NONE ⇒ (min, max) == (0, 0)` — "no elevation" may not smuggle Z bounds;
4. **provenance ⇔ Z-dimension**: `elevation_source == NONE` **iff** version is
   flat `D2`; any non-NONE provenance requires `D2_5`/`D3` (and vice versa);
5. envelope must fit inside the asset's own bounding box
   (`asset.elevation_min ≤ version.min`, `version.max ≤ asset.elevation_max`,
   else `InvalidElevationRange` 6235).

## 5. Error codes (appended, 6231–6240)

| Code | Name | Raised when |
|------|------|-------------|
| 6231 | `InvalidSpatialDimension` | dimension out of range, or version exceeds asset dimensionality |
| 6232 | `InvalidGeometrySource` | unknown provenance source |
| 6233 | `GeometryVersionMismatch` | verified entry belongs to a different asset |
| 6234 | `GeometryVersionsFull` | cursor at `MAX_GEOMETRY_VERSIONS` |
| 6235 | `InvalidElevationRange` | `elevation_min_mm > elevation_max_mm`, or version envelope outside the asset envelope |
| 6236 | `GeometryAlreadyVerified` | verify after the quorum already flipped |
| 6237 | `InvalidElevationSource` *(Phase B)* | unknown elevation source code; `NONE` with nonzero envelope; provenance ⇔ dimension mismatch |
| 6238 | `GeometryAlreadyAttested` *(Phase B)* | same validator attests twice |
| 6239 | `EmptyEvidenceManifest` *(Phase B)* | `append_geometry_version_from_evidence` with `artifact_count == 0` |
| 6240 | `GeometryQuorumFull` *(Phase B)* | attestation array full (`attest_count == MAX_VALIDATORS`) before quorum |

Reuses: `EmptyGeometryHash` (6002), `EmptyStorageReference` (6134),
`NotAuthorized` (6010), `TaskAlreadyFinalized` (6174, evidence-anchored append
on a terminal task), `InvalidRequiredAttestations` (quorum config bounds).

## 6. Off-chain integration (existing, unchanged)

The GIS layer already ships: PostGIS `parcels.geometry` (+ `parcel_spatial_stats`
views), roads/POIs/pilot zones fusion (`0002`), spatial REST endpoints
(`routes/spatial.rs`, `routes/fusion.rs`), globe UI (Cesium + 2D Leaflet
fallback, draw-and-register flow). Phase A's `storage_reference` points at
exactly those canonical documents; `geometry_hash` mirrors their canonical
digest. API mirror tables + REST for `spatial_asset`/`geometry_version`
accounts are Phase E (see below).

## 7. Phase plan

| Phase | Scope | Status |
|-------|-------|--------|
| **A** | On-chain `SpatialAsset` + `GeometryVersion` (init / append / verify, 3 ixs, 2 accounts, 3 events, 6 errors), unit + BPF tests, IDL, docs | ✅ **Delivered 2026-09-28** |
| **B** | Geometry linked to the evidence stack: `append_geometry_version_from_evidence` (version anchored to an `EvidenceManifest`), elevation provenance (`elevation_source` + 5 invariants), validator-quorum verification (`attestors`/`attest_count`/`required` snapshot via global `QuorumConfig`), +1 ix, +1 event, +4 errors (6237–6240) | ✅ **Delivered 2026-10-01** |
| **C** | `ThreeDModel` account (off-chain model ref + hash, DR10) and `SpatialSnapshot` point-in-time reads — the 4D layer of RFC-012 §5 | ⬜ |
| **D** | AI/GIS worker integration (photo/LiDAR → geometry draft → anchor), provenance attribution, globe rendering of version history | ⬜ |
| **E** | API mirror + REST routes for spatial accounts; devnet deployment | ⬜ |

## 8. Testing (Phases A + B)

- Unit (`spatial_asset.rs`, 11): dimension/source enums, elevation range,
  NONE-zero envelope, provenance ⇔ dimension biconditional, envelope-fits-
  asset, quorum bounds (1..=8 + fallback 2), attestor dedup list, cap
  arithmetic — registry lib **142/142**
- BPF (`tests/integration.rs`): `stage3_spatial_asset_lifecycle` (init happy
  + duplicate PDA + guards 6231/6235/6232/6002/6134 + **Phase B elevation
  guards 6237 ×4 and envelope 6235** + appends + field asserts + cap 6234 via
  cursor poke), `stage3_geometry_version_verification` (non-validator reject,
  **quorum flow: attest 1 → not verified, re-attest 6238, attest 2 → flips,
  verified_by = completing validator, double-verify 6236, cross-asset 6233**),
  `stage3_evidence_linked_geometry` (**new** — task + manifest + GEOMETRY
  artifact, anchored append asserts `evidence_manifest` link, empty manifest
  6239, cancelled task 6174), `stage3_geometry_quorum_config` (**new** —
  global `QuorumConfig (0,[0,0])` → `required = 3`: 1st/2nd attest not
  verified, 3rd flips) — integration **305/305**
- IDL synced: **162 / 67 / 149 / 241** (`terra-web/src/idl/`)

## 9. Open questions (Phase B+)

1. Lifecycle of `SpatialAsset` itself (does it need DRAFT/ARCHIVED status —
   `asset.authority` exists for this, unused in Phase A)?
2. ~~Permissionless append economics: rent-paid + cap bounds spam, but should
   verified-only versions rank higher in reads?~~ **Resolved (Phase B):**
   verification is now quorum-weighted — `required` snapshots the configured
   threshold at append time; reads can rank by `attest_count`/`required`.
3. ~~Elevation source provenance (DEM tiles vs survey) and who may attest
   it.~~ **Resolved (Phase B):** `elevation_source` enum (NONE/SURVEY/DEM/
   LIDAR/PHOTOGRAMMETRY/MANUAL) with the provenance ⇔ dimension invariant;
   attestation is validator-quorum based (same flow as the fact itself).
4. Canonicalization rule for `geometry_hash` (GeoJSON ring order, precision,
   CRS — document the exact digest recipe next to the evidence-stack digest).
5. Snapshot semantics: does `SpatialSnapshot` copy indexes at time T, or is a
   version-range query enough?

## 10. Handoff

Start at [RFC-012 §8.1](rfc-012-global-physical-digital-trust-architecture.md)
for the architecture contract and devnet checklist, then pick a Phase C–E
slice from §7. Stage 3 remains 🟡 until the AI/GIS loop (D) runs end-to-end;
Stages 4–5 build on `ThreeDModel`/`SpatialSnapshot` (C).
