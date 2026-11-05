# Chapter 3 — Multi-Layer 3D Spatial Intelligence

**Status:** `IMPLEMENTED AND VERIFIED` for on-chain 3D model registration,
versioned geometry, elevation rules, canonical 2D geometry digest, and
spatial-account mirror; `IMPLEMENTED, NOT FULLY VERIFIED` for the off-chain
reconstruction worker pipeline, distributed multi-engine reconstruction,
component packaging, assembly, synchronized runtime, and progressive
visualization (Chapters 11, 12, 13 — implemented Sprint 025, 2026-11-02/03;
live-cluster verification pending on B6 devnet funding); `SPECIFIED ONLY`
for institutional identity / residency / liens (Chapters 2, 6, 8).

**Evidence:** RFC-013 Phases C/D/E; `spatial_asset.rs`, `three_d_model.rs`,
`spatial_snapshot.rs`; migrations 0029–0043; stage3 BPF suites; commit
`775e20a` (`terra-geom-v1` digest, 2D, frozen); `geo-engine/src/canonical3d.rs`
(`terra-geom-v2` digest, 3D, **integrated** Sprint 025 per ADR-016); the
RFC-015 conformance validator in `geo-engine/src/component_package.rs`.

**See:** Chapter 11 (Distributed Spatial Reconstruction and Assembly), Chapter
12 (World Runtime and Synchronized Spatial Rendering), Chapter 13 (Global
Spatial Data Delivery and Progressive Visualization), RFC-013, RFC-014,
`docs/canonical-geometry-digest.md`, `RFC-015` (Spatial Component Package — not
yet written).

---

## 3.1 The problem

2D parcel boundaries cannot represent stacked cities, underground structures,
vertical farming, or volumetric rights. Fragmented reconstruction data (LiDAR,
drones, photogrammetry) has no tamper-evident link to on-chain ownership —
Failure 2 in [Chapter 1](01-core-identity-and-philosophy.md). Terra must
represent **volumetric** space and **verifiable construction progress**, not just
footprints — and it must do so without forcing every participant to use one
reconstruction engine, one storage provider, or one renderer.

---

## 3.2 Desired outcome

Physical entities have stable identifiers, versioned 2D/3D geometry, elevation
semantics, temporal history, and cryptographic commitments — so applications can
reason about volumetric space and verifiable construction progress. Separately
produced components connect through common manifests and spatial relationships
into one synchronized, progressively visualized world representation (Chapters
11–13).

---

## 3.3 What is built (`IMPLEMENTED AND VERIFIED`)

| Piece | Evidence |
|---|---|
| Spatial assets with append-only geometry versions | `init_spatial_asset`, `append_geometry_version`, `append_geometry_version_from_evidence`, `verify_geometry_version` (`spatial_asset.rs` L266–518) |
| 3D model registration with format label | `register_3d_model`, `verify_3d_model` (`three_d_model.rs` L121–190) |
| Spatial snapshots | `create_spatial_snapshot` (`spatial_snapshot.rs` L107) |
| Elevation envelope rules | `elevation_range_ok` / `elevation_none_is_zero` / `elevation_matches_dimension` / `dimension_within_asset` (`spatial_asset.rs` L179–246); version dimension ≤ asset dimension |
| Canonical geometry digest | `terra-geom-v1` (commit `775e20a`); `docs/canonical-geometry-digest.md`; `terra-core/geo-engine/src/canonical.rs` |
| BPF lifecycle tests | `stage3_spatial_asset_lifecycle`, `stage3_geometry_version_verification`, `stage3_evidence_linked_geometry`, `stage3_geometry_quorum_config`, `stage3_3d_model_lifecycle`, `stage3_spatial_snapshot_lifecycle` |
| Off-chain mirror | migration `0030_spatial_accounts_mirror.sql`; `/spatial-accounts/*` REST |

**Canonicalization honesty:** the geometry hash commits to a *representation*
under documented normalization rules. It does **not** prove the real-world
boundary is correct — that remains a verification-consensus question (Chapter 1
§1.3).

**3D digest status:** `terra-geom-v1` is frozen and 2D-only (rejects third
ordinates) — it remains the digest for all existing parcel hashes. `terra-geom-v2`
(`geo-engine/src/canonical3d.rs`, 3D, requires `elevation_ref`, `vertical_units`,
`precision`) was **integrated into production** in Sprint 025 (2026-11-02) per
ADR-016 and is the digest consumed by the RFC-015 conformance validator
(`geo-engine/src/component_package.rs`). It is additive, not a replacement:
v1 parcel hashes are unaffected.

---

## 3.4 The reconstruction worker (`PARTIALLY IMPLEMENTED`)

Migration `0031`; `routes/reconstruction.rs`:

- `POST /jobs/lease` — `SKIP LOCKED` + 15-minute lease + expired reclaim.
- `POST /jobs/{id}/progress` — heartbeat.
- `POST /jobs/{id}/complete` — PostGIS convex hull from ≥3 capture points
  (clipped to parcel) labeled `capture-hull`, else `parcel-footprint-fallback`;
  SHA-256 geometry hash over exact GeoJSON bytes; writes `three_d_models`;
  produces `anchor_prep` with real PDAs derived from parcel on-chain hex.
- `GET /jobs/{id}/draft`, `POST /drafts/{id}/anchor`.

**Gap:** `anchor_prep` builds the transaction but has never been executed on a
live cluster — blocked on devnet funding (Q-001 / RISK R-001). Until then,
end-to-end reconstruction anchoring is `PARTIALLY IMPLEMENTED`.

**Multi-engine status:** None. The reconstruction API is a single worker path.
There is no worker registration, no engine negotiation, no component package
conformance check, and no assembly system connecting independently produced
components (see Chapters 11–13).

---

## 3.5 AI-assisted reconstruction (partial)

AI/draft outputs are labeled (`capture-hull` vs fallback) so a consumer can
always tell measured geometry from interpolated geometry. Model provenance
(model, version, confidence) is **not yet recorded** — RISK R-014 / REQ-024.
Migration `0034_ai_provenance.sql` adds `provenance_kind` plus model
provider/name/version/confidence fields to geometry drafts. The protocol must
never let an AI output silently become verified geometry.

---

## 3.6 Spatial object identity and relationships

Terra must distinguish four kinds of identity that are easily conflated:

- **Physical identity** — the real-world object (a parcel, a building, a roof).
- **Legal identity** — ownership, rights, easements (on-chain `Rights PDA`,
  not a geometry column).
- **Geometry identity** — a specific version of a shape (`GeometryVersion`,
  `geometry_hash`).
- **Digital file identity** — a stored artifact (mesh, point cloud, BIM file).

A building must retain its **physical identity** when its roof is reconstructed,
a floor is added, or one component is replaced. A subdivision must preserve the
history and relationship between the original parcel and its resulting parcels.

The relationship graph must support: parent-child, spatial containment,
adjacency, intersection, connection, dependency, shared boundaries, ownership or
legal assertions (as separate information), and version lineage. Relationships
must be classified as geometric, semantic, administrative, or legal.

Do not assume every spatial object is a simple parent of another object:

- A building can span multiple parcels.
- A road can cross several administrative areas.
- A utility can connect many buildings.
- A parcel can contain multiple buildings.
- A building can have several independently reconstructed components.

---

## 3.7 Coordinate frames and geometry standards

- **Horizontal CRS:** EPSG:4326 (`x` = longitude, `y` = latitude). Other
  horizontal CRSes are rejected.
- **2D quantization:** `1e-7` degrees (~1.1 cm at the equator), rounded
  half-away-from-zero. Third ordinates are rejected by `terra-geom-v1`.
- **Ring normalization:** consecutive duplicate vertices removed; closing
  vertex implicit; degenerate rings dropped; RFC 7946 orientation (exterior
  counter-clockwise, holes clockwise); rings rotated to start at the
  lexicographically smallest `(lon, lat)` vertex; children sorted and
  de-duplicated.
- **3D extension:** `terra-geom-v2` (experimental, `terra-3d-spatial-pipeline`
  branch) adds `crs`, `elevation_ref` (`EGM2008` / `WGS84-ELLIPSOID` / `local`
  / `unknown`), `vertical_units` (`m` / `ft`), `precision`, and optional third
  ordinates quantized to `1e-5` units. Domain-separated from v1 so digests
  cannot collide.
- **Do not use a 2D geometry digest for arbitrary 3D models.** Keep
  `terra-geom-v1` stable for existing parcel hashes.
- **Local frames:** local ENU coordinate frames where applicable, with an
  explicit local-to-global transform stored per component.

---

## 3.8 Geometry and model versioning

A physical entity may have many representations: an original footprint, a
simplified 3D model, a detailed roof reconstruction, a corrected facade, a new
interior representation, a new terrain alignment. These must not overwrite one
another without history:

- Component versions are **immutable**.
- An **explicit current-version pointer** identifies the accepted version.
- **Version dependencies** record which component versions an assembly depends on.
- **Parent assembly versions** and **supersession relationships** are tracked.
- Validation status transitions:
  `pending → technically_valid → independently_valid → rejected`.
- Rejected candidates are retained, not deleted.

A new component version must not automatically become the accepted current
version. It must pass technical and validation conditions first.

---

## 3.9 AI provenance

Migration `0034_ai_provenance.sql` records `provenance_kind` plus model
provider/name/version/confidence on geometry drafts, with fail-closed validation.
Every reconstruction component must carry provenance classification so consumers
can distinguish imported, observed, surveyed, reconstructed, inferred, and
manually corrected geometry (Chapter 13 §13.5). Model-provenance fields must be
complete before AI outputs feed any validated geometry (RISK R-014 resolved by
migration, but live validation must be maintained).

---

## 3.9 Acceptance criteria (to reach full `IMPLEMENTED AND VERIFIED`)

- [ ] Devnet: worker lease → complete → anchor transaction executes on-chain.
- [ ] Model provenance fields (model/version/confidence) recorded per draft.
- [ ] Multi-layer nesting (building → floor → room) tested against elevation
      envelope rules.
- [ ] `terra-geom-v2` reviewed and either integrated into production `canonical`
      or formally deprecated; decision recorded in `ARCHITECTURAL_DECISIONS.md`.
- [ ] Chapter 11 `SpatialAsset`/`GeometryVersion` anchor mapping: document how a
      `Spatial Component Package` (component_id, physical_entity_id,
      geometry_digest, storage_reference) maps to an on-chain `SpatialAsset`
      PDA + `GeometryVersion` row, including which fields live on-chain vs
      off-chain.
- [ ] Chapter 13 → frontend linkage: document how `LOD_LEVELS` /
      `LOD_TARGET_HEIGHT` (frontend `lib/lod.ts`) and the terrain provider
      (`lib/cesiumTerrain.ts`) map to Chapter 13 lazy-loading levels L0–L4 and
      the storage-provider discovery contract.
