# RFC-015 — Spatial Component Package (Vision Stage 4 boundary)

- **Status:** `IMPLEMENTED, NOT FULLY VERIFIED` — the schema is specified here and
  the conformance validator is implemented (`terra-core/geo-engine/src/component_package.rs`,
  `validate()` + `ComponentPackage`); the API surface is implemented
  (`api/src/routes/components.rs` + migrations 0035/0041). Not yet exercised
  against a live cluster — B6 devnet funding blocks that.
- **Depends on:** RFC-012 (architecture contract), RFC-013 (spatial intelligence
  pipeline, `terra-geom-v1` digest), Chapter 11 (distributed reconstruction &
  assembly), Chapter 12 (world runtime & synchronized rendering),
  Chapter 13 (global spatial data delivery & progressive visualization).
- **Implements:** REQ-026 (distributed spatial component package & assembly).
- **Related:** `docs/canonical-geometry-digest.md` (`terra-geom-v1`),
  `docs/book-of-intents/chapters/11-distributed-spatial-reconstruction-and-assembly.md`,
  `docs/book-of-intents/chapters/12-world-runtime-and-synchronized-rendering.md`.

---

## 0. Core invariant

A Spatial Component Package is a **description of a spatial representation**,
not a claim that the representation matches physical reality. The package
carries provenance, accuracy, uncertainty, and validation state so consumers
can distinguish measured geometry from inferred or AI-generated geometry. Only
a registered validator's verification turns a component into a verified fact —
this RFC never short-circuits the evidence stack (RFC-013 §0, RFC-012 §0).

---

## 1. Why

Terra's on-chain model (`SpatialAsset`, `GeometryVersion`, `ThreeDModel`)
anchors geometry hashes and model references, but it has **no schema for the
off-chain component file itself**. Today:

- `ThreeDModel` records a `uri` and `content_hash` but no coordinate frame,
  elevation reference, transform, engine identity, accuracy, uncertainty, or
  dependency information.
- The reconstruction API produces `geometry_drafts` with a SHA-256 hash over
  exact GeoJSON bytes, but no versioned package that multiple independent
  workers can exchange.
- There is no conformance test, so any engine exporting GLB or GeoJSON would
  be indistinguishable from a Terra-compatible engine.

Independent reconstruction workers cannot interoperate without a common package
schema. RFC-015 defines that schema and its conformance rules.

---

## 2. Target pipeline

```
Worker A (photogrammetry)   Worker B (LiDAR)           Worker C (CAD/BIM)
─────────────────           ───────────                ────────────────
component package           component package          component package
(RFC-015 schema)            (RFC-015 schema)           (RFC-015 schema)
         │                       │                       │
         └───────────┬───────────┴───────────┬───────────┘
                     ▼                       ▼
            Component Registry (RFC-015 conformance)
                     │
                     ▼
            Assembly System (Chapter 11)
              registration + alignment
                     │
                     ▼
            Assembly Manifest (versioned)
                     │
                     ▼
            World Representation Manifest (Chapter 12)
                     │
                     ▼
            Runtime: discover → fetch → verify → transform → render (Chapter 12/13)
```

---

## 3. Spatial Component Package schema

Every independently produced component must be delivered as a versioned
package. The package is a JSON document with the following fields:

| Field | Type | Required | Description |
|---|---|---|---|
| `component_id` | string (UUID) | yes | Stable identifier of this component |
| `physical_entity_id` | string (UUID) | yes | Stable identifier of the real-world object (parcel, building, terrain tile) |
| `parent_id` | string (UUID) | no | Parent component/object in the relationship graph |
| `related_ids` | string[] | no | Related component/object IDs (adjacency, intersection, connection) |
| `component_type` | string | yes | `footprint`, `roof`, `facade`, `terrain`, `interior`, `utility`, `vegetation`, `water` |
| `geometry_artifact` | string (URI) | yes | Off-chain model file URI (mesh, point cloud, BIM footprint) |
| `format` | string | yes | MIME/type + version, e.g. `model/gltf-binary;version=2.0`, `application/vnd.cesium+json;version=1.0` |
| `format_version` | string | yes | Format schema version |
| `geometry_digest` | string (hex-64) | yes | `terra-geom-v1` (2D) or `terra-geom-v2` (3D) canonical digest |
| `digest_algorithm_version` | string | yes | `terra-geom-v1` or `terra-geom-v2` |
| `crs` | string | yes | Horizontal CRS, normalized to `EPSG:4326` (RFC 7946 / CRS 84 aliases accepted) |
| `elevation_ref` | string | yes | `EGM2008` (geoid MSL), `WGS84-ELLIPSOID` (GNSS ellipsoid height), `local` (site-relative), `unknown` |
| `vertical_units` | string | yes | `m` (metres) or `ft` (feet) |
| `local_frame` | object | no | Local ENU frame: `{ origin: [lon, lat], rotation_deg, scale }` |
| `global_transform` | array[16] | no | Column-major 4x4 local→global transform matrix |
| `bounding_volume` | object | yes | `{ min: [lon, lat, z], max: [lon, lat, z] }` in declared CRS + vertical units |
| `spatial_extent` | object | yes | GeoJSON Geometry describing the component's footprint |
| `resolution` | number | yes | Nominal ground sample distance in `vertical_units` |
| `lod` | integer | yes | Level of detail (0 = coarsest, higher = finer) |
| `source_evidence_refs` | string[] | yes | References to source evidence (capture sessions, survey records, GIS imports) |
| `reconstruction_engine_id` | string | yes | Identity of the reconstruction engine that produced this component |
| `reconstruction_engine_version` | string | yes | Engine version |
| `processing_configuration` | object | no | Engine-specific processing parameters (for reproducibility) |
| `creation_timestamp` | string (ISO-8601) | yes | When the component was produced |
| `resource_usage_receipt` | object | no | Metered CPU/GPU/memory/storage/bandwidth + cost (Chapter 10 §37) |
| `accuracy` | number | no | Expected geometric error in `vertical_units` |
| `uncertainty` | number | no | Uncertainty bound in `vertical_units` |
| `coverage` | object | no | `{ area_m2, completeness_pct }` |
| `confidence` | number | no | Engine-reported confidence in [0,1] — description, not verification |
| `provenance_classification` | string | yes | `imported`, `observed`, `surveyed`, `reconstructed`, `inferred`, `manually_corrected` |
| `dependencies` | string[] | no | Component IDs this component depends on |
| `overlap_ids` | string[] | no | Component IDs whose geometry overlaps this one |
| `privacy_classification` | string | yes | `public`, `restricted`, `private` |
| `access_policy` | object | yes | Capability-based access rules (Chapter 13 §13.8 / Chapter 10 §44) |
| `validation_state` | string | yes | `pending`, `technically_valid`, `independently_valid`, `rejected` |
| `version_lineage` | object | yes | `{ previous_version_id, superseded_by, assembly_version }` |

---

## 4. Conformance rules

An engine is **not** Terra-compatible merely because it exports a GLB or
GeoJSON file. A package passes conformance only if:

1. **Digest integrity:** `geometry_digest` equals the canonical digest
   (`terra-geom-v1` or `terra-geom-v2`) computed over the geometry in
   `geometry_artifact`, using the declared `digest_algorithm_version`.
2. **CRS validity:** `crs` normalizes to `EPSG:4326`; other horizontal CRSes are
   rejected.
3. **Elevation reference:** `elevation_ref` is one of `EGM2008`,
   `WGS84-ELLIPSOID`, `local`, `unknown`. Mixing ellipsoid and MSL heights is
   detectable and flagged.
4. **Vertical units:** `vertical_units` is `m` or `ft`; horizontal units are
   fixed degrees by the CRS.
5. **Position validity:** X/Y quantized to `1e-7` degrees; Z finite and
   `|z| ≤ 100000` in `vertical_units`; positions carry 2 or 3 ordinates only.
6. **Bounding volume consistency:** `bounding_volume` encloses
   `spatial_extent`; transform (if present) maps local frame to global frame
   within declared tolerance.
7. **Provenance completeness:** AI-kind packages (`provenance_classification == reconstructed` with AI engine) must declare `reconstruction_engine_id`, `reconstruction_engine_version`, and `confidence` ∈ [0,1]. Deterministic packages must not carry AI model metadata.
8. **Privacy:** `privacy_classification` and `access_policy` are present and internally consistent.

The conformance test is a standalone validator (Rust or TS) that rejects any
package violating these rules. It is the gate before a component can enter the
Component Registry (Chapter 11 §11.3).

---

## 5. Connection to existing code

| Existing capability | Reused by this RFC | Gap |
|---|---|---|
| `ThreeDModel` (`three_d_model.rs`, `register_3d_model`, `verify_3d_model`) | Model artifact reference pattern (`uri`, `content_hash`) | No component schema; no engine identity, accuracy, uncertainty, dependency, or version-lineage fields |
| `GeometryVersion` (`spatial_asset.rs`, `append_geometry_version`) | On-chain version anchor for `geometry_digest` | No off-chain package submission or conformance check |
| `routes/reconstruction.rs` (`jobs`, `complete → draft → anchor`) | Single-worker reconstruction flow baseline | No multi-worker package submission, no engine negotiation, no assembly |
| `task_economics.rs` (`FeePolicy`, `CoverageIncentive`, `TaskEscrow`, `RewardAllocation`) | Payment/subsidy primitives | Applies to verification tasks, not reconstruction tasks; needs new task types |
| `canonical.rs` (`terra-geom-v1`) | 2D geometry digest baseline | Rejects 3D ordinates; no 3D digest canonicalization in production code |
| `canonical3d.rs` / `terra-geom-v2` (branch-only, experimental) | Candidate 3D digest | Not merged; experimental (ADR-016) |

---

## 6. Open decisions (founder review)

- Whether `terra-geom-v2` is promoted to production digest (integrate-or-deprecate; ADR-016).
- Whether the conformance validator lives in `terra-core/geo-engine` or as a standalone service.
- Whether `component_id` is a content-addressed hash (derived from package bytes) or a separately assigned UUID.
