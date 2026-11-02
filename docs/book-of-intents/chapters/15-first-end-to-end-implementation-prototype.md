# Chapter 15 — First End-to-End Implementation Prototype

**Status:** `SPECIFIED ONLY` — specification, acceptance criteria, and acceptance tests; no implementation code yet.

**See:** RFC-013 (spatial intelligence pipeline), RFC-014 (derived compute), RFC-015 (spatial component package), RFC-016 (security), Chapters 3, 10, 11, 12, 13.

---

## 15.1 Purpose

Before proposing a global production network, Terra must demonstrate one working end-to-end example that proves the architecture. This chapter specifies that prototype: one parcel, one building, independently reconstructed components from multiple workers, stored separately, connected through manifests and spatial relationships, retrieved dynamically, assembled into a synchronized representation, progressively visualized, independently validated, and continuously updated.

The prototype proves the architecture, not just displays a 3D model. It must run on modest hardware; GPU-intensive AI is optional.

---

## 15.2 Prototype scope

### A. Physical area

One parcel containing one building. The parcel is approximately 500–2000 m². The building is a single-family dwelling with one roof, one or more facades, a foundation, and a footprint.

### B. Independent components (minimum five)

| Component | Description | Worker |
|-----------|-------------|--------|
| Parcel boundary | 2D parcel polygon from survey/GIS | Worker A |
| Terrain | Elevation surface (2.5D) covering the parcel | Worker B |
| Building footprint | 2D/2.5D building outline | Worker A |
| Roof | Volumetric roof structure (mesh) | Worker B |
| Facade | One or more facade components (mesh) | Worker C |

Each component is independently produced, independently stored, and independently versioned.

### C. Distributed processing

At least two independent computers or workers:

- **Worker A** processes the parcel boundary and building footprint (GIS-based, CPU-only).
- **Worker B** processes the terrain and roof (photogrammetry/LiDAR, GPU-optional).
- **Worker C** (optional) processes facade components (photogrammetry, GPU-optional).

A separate assembly process registers and aligns the components into a coherent building representation.

### D. Spatial object structure

Each component must have:

| Field | Description |
|-------|-------------|
| Stable identity | Component ID (UUID) |
| Version | Component version (immutable) |
| Coordinates | CRS (EPSG:4326), elevation reference, vertical units |
| Transform | Local-to-global transform matrix |
| Provenance | Imported/observed/surveyed/reconstructed/inferred |
| Geometry hash | terra-geom-v1 or terra-geom-v2 digest |
| Accuracy/uncertainty | Expected geometric error in vertical units |
| Storage reference | Retrieval URI (object storage) |

### E. Central coordination

A **World Representation Manifest** (Chapter 12) identifies the component versions and their relationships. The manifest:

- Identifies the geographic region (the parcel)
- Lists spatial objects (parcel, building, terrain, roof, facade)
- Records their stable IDs and current component versions
- Records assembly versions and dependencies
- Provides storage references (retrieval URIs)
- Lists available LODs and validation states
- Records update history
- References parent and neighboring manifests

### F. Assembly

The assembly system (Chapter 11) aligns the components:

1. Registers each component with its spatial transform
2. Detects overlaps and boundary mismatches
3. Resolves registration conflicts (preserves both candidates if merge is unsafe)
4. Produces an Assembly Manifest identifying exact component versions
5. Produces a reproducible building assembly

### G. Runtime

The client (Chapter 12/13):

1. Discovers the World Representation Manifest
2. Resolves component versions
3. Discovers storage providers (with fallback)
4. Retrieves components independently
5. Combines them into one scene
6. Applies coordinate transformations
7. Assembles the scene graph
8. Renders available components

### H. Progressive visualization

The client:

1. Displays the parcel boundary immediately (Level 0/1)
2. Shows a preliminary building (simplified volumetric, Level 2)
3. Loads available components (roof, facade, Level 3)
4. Replaces previews with higher-detail assets
5. Shows missing or unverified components distinctly (orange outline vs cyan fill)
6. Supports basic camera and navigation workflow

### I. Updating

Replace one roof component without reconstructing the whole building:

1. Worker B produces a new roof component candidate
2. The new component is validated (technical checks)
3. Affected assembly dependencies are identified
4. A new assembly candidate is created
5. Only affected relationships are recomputed
6. A new manifest version is published
7. Connected clients transition safely to the new version
8. The previous assembly version is retained (reproducible)

### J. Economic accounting

Record:

| Item | Description |
|------|-------------|
| Task specification | What was requested, accuracy thresholds, deadline, budget |
| Assigned workers | Which workers were assigned to which components |
| Resource usage | CPU/GPU/storage/bandwidth receipts per worker |
| Cost | Total cost per component and total |
| Escrow/payment state | Funds reserved, released, or refunded |
| Accepted outputs | Which component versions were accepted |
| Validation results | Technical validation, independent validation |
| Contributor attribution | Which worker contributed which component |

---

## 15.3 Prototype architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Worker A       │     │  Worker B       │     │  Worker C       │
│  (parcel,       │     │  (terrain,      │     │  (facade)       │
│   footprint)    │     │   roof)         │     │                 │
└─────────┬───────┘     └─────────┬───────┘     └─────────┬───────┘
          │                       │                       │
          ▼                       ▼                       ▼
┌─────────────────────────────────────────────────────────────────┐
│  Component Packages (RFC-015) — independently stored            │
│  • component package A: parcel boundary (GeoJSON)               │
│  • component package B: terrain (point cloud/mesh)              │
│  • component package C: roof (GLB mesh)                         │
│  • component package D: facade (GLB mesh)                       │
└─────────────────────────────────────────────────────────────────┘
          │                       │                       │
          ▼                       ▼                       ▼
┌─────────────────────────────────────────────────────────────────┐
│  World Representation Manifest (Chapter 12)                     │
│  • region: parcel-001                                           │
│  • objects: parcel, building, terrain, roof, facade             │
│  • current component versions                                   │
│  • assembly versions                                            │
│  • storage references                                           │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────────┐
│  Assembly System (Chapter 11)                                   │
│  • component registration                                       │
│  • alignment + registration transforms                          │
│  • conflict detection                                           │
│  • assembly manifest                                            │
└─────────────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────────┐
│  Runtime (Chapter 12/13)                                        │
│  • spatial index → manifest resolution                          │
│  • provider discovery + fallback                                │
│  • asset retrieval + digest verification                        │
│  • scene graph assembly                                         │
│  • progressive LOD rendering                                    │
└─────────────────────────────────────────────────────────────────┘
```

---

## 15.4 Acceptance criteria

The prototype is complete when all of the following are demonstrated:

| # | Criterion | Verification method |
|---|-----------|---------------------|
| A1 | Two different workers produce valid component packages | Two workers submit packages via `POST /components`; both pass RFC-015 conformance |
| A2 | Packages use different internal engines | One worker uses photogrammetry, another uses LiDAR processing; both satisfy the same output specification |
| A3 | Coordinate frames resolve correctly | All components use EPSG:4326 with declared elevation reference; transforms map local→global |
| A4 | Components assemble into a common scene | Assembly system produces a versioned Assembly Manifest; all five components are referenced |
| A5 | Assembly manifest identifies exact component versions | Each component version in the manifest is immutable and addressable |
| A6 | Runtime retrieves components from separate storage providers | Runtime fetches components from at least two different `storage_providers` rows |
| A7 | Runtime does not download unrelated geographic data | Runtime fetches only components for the target parcel, not neighboring parcels |
| A8 | Viewer displays low-detail before high-detail | LOD 0 (parcel boundary) renders before LOD 3 (roof/facade meshes) |
| A9 | Viewer shows missing/preliminary/validated distinctly | Unfetched components render as orange outlines; fetched as cyan volumes; validated as green |
| A10 | Component update does not rebuild unrelated components | Replacing the roof component creates a new assembly version without touching terrain/facade |
| A11 | Previous assembly is reproducible from its manifest | The previous manifest version resolves to the same component versions |
| A12 | Provider resource usage is recorded and checked | Each worker submits a `resource_receipt` with CPU/GPU/storage/bandwidth values |
| A13 | Payment depends on accepted work, not submission | Payment is released only after validation passes (`independently_valid`) |
| A14 | Storage-provider failure does not blank out a region | If one provider is unavailable, the runtime falls back to another replica |
| A15 | Privacy restrictions enforced at retrieval | A `PRIVATE` component is not returned by `GET /components/{id}` without authorization |

---

## 15.5 Acceptance tests

### AT-1: Component submission and dedup

1. Worker A submits a parcel boundary component package via `POST /components`.
2. Worker A submits the same component again with the same `processing_key`.
3. The API returns the same `component_id` with `created=false` (dedup).
4. The `derived_jobs` row has `status='pending'` and `created_by` set.

### AT-2: Multi-engine compatibility

1. Worker A submits a roof component from photogrammetry (mesh, `model/gltf-binary`).
2. Worker B submits a roof component from LiDAR (point cloud, `application/vnd.cesium+json`).
3. Both packages pass RFC-015 conformance (digest, CRS, elevation, bounding volume).
4. Both are stored as separate component versions.

### AT-3: Assembly and registration

1. Submit parcel, terrain, roof, and facade components.
2. Create an assembly via `POST /assemblies`.
3. Add each component via `POST /assemblies/{id}/objects` with relationship types.
4. The assembly manifest identifies all five component versions.
5. The assembly status is `pending` (awaiting validation).

### AT-4: Runtime scene resolution

1. Create a `World Representation Manifest` via `POST /manifests` for the parcel region.
2. Add the assembly's components via `PUT /manifests/{id}/objects`.
3. Call `GET /runtime/scene/{manifest_id}` at LOD 0.
4. The response includes parcel and terrain components (LOD 0).
5. Call `GET /runtime/scene/{manifest_id}/3` at LOD 3.
6. The response includes roof and facade components (LOD 3).

### AT-5: Provider fallback

1. Register two storage providers via `POST /providers`.
2. Register replicas for the same component on both providers via `POST /providers/{id}/replicas`.
3. Mark the first provider unavailable (`is_available=false`).
4. Call `GET /runtime/assets/{component_id}` — the API returns the second provider's replica.

### AT-6: Incremental update

1. Create assembly version 1 with roof version 1.
2. Submit roof version 2 (new geometry).
3. Create assembly version 2 with roof version 2.
4. Assembly version 1 is retained and reproducible.
5. Assembly version 2 references the new roof version; terrain/facade versions are unchanged.

### AT-7: Economic accounting

1. Create a reconstruction task with budget and accuracy thresholds.
2. Assign workers A and B to components.
3. Workers submit resource receipts via `POST /assignments/{job_id}/complete`.
4. Payment is released only after validation passes (`independently_valid`).
5. Contributor attribution is recorded per component.

---

## 15.6 Smallest practical implementation stack

| Layer | Component | Technology |
|-------|-----------|------------|
| On-chain | SpatialAsset, GeometryVersion, ThreeDModel | Solana/Anchor (terra_registry) |
| Backend API | Component packages, manifests, assemblies, providers, runtime | Rust/Axum + Postgres (terra-api) |
| Frontend | Runtime client, scene graph rendering, worker console | TypeScript/Cesium (terra-web) |
| Storage | Component packages, model assets | Object storage (S3-compatible) |
| Compute | Workers A, B, C | Any (CPU-only for parcel/footprint; GPU-optional for roof/facade) |

The prototype runs on modest hardware: a single machine can host all three workers sequentially; GPU-intensive AI is optional (photogrammetry can use CPU-only pipelines).

---

## 15.7 Code areas requiring implementation

| Code area | Change | Depends on |
|-----------|--------|------------|
| `terra-core/api/src/routes/derived.rs` | Add worker lease/claim for derived jobs | RFC-015 component package |
| `terra-core/api/src/routes/workers.rs` | Already implemented (worker registration, job assignment) | — |
| `terra-web/src/pages/WorkerConsolePage.tsx` | Already implemented (worker registration UI) | — |
| `terra-web/src/pages/GlobePage.tsx` | Already implemented (runtime scene rendering) | RFC-015 component package |
| `terra-web/src/pages/` | New: Derived jobs page (worker claims derived jobs) | RFC-015 component package |
| `terra-core/programs/terra_registry/src/task_economics.rs` | Extend for reconstruction task types (not verification-only) | Founder decision (economic model) |
| `terra-core/programs/terra_registry/src/spatial_asset.rs` | Already implemented (versioned geometry anchors) | — |

---

## 15.8 Exit criteria

The prototype is complete when:

1. All 15 acceptance criteria (A1–A15) are demonstrated.
2. All 7 acceptance tests (AT-1–AT-7) pass.
3. The on-chain program compiles and BPF tests pass.
4. The API test suite passes (133/133).
5. The frontend test suite passes (83/83).
6. The Vite build succeeds.
7. The prototype is documented in this chapter with source code references.
