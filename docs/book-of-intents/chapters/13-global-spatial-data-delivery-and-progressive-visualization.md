# Chapter 13 — Global Spatial Data Delivery and Progressive Visualization

**Status:** `IMPLEMENTED, NOT FULLY VERIFIED` — the specification (this chapter)
was written first (Sprint 024, commit `d293b0e`); the code followed immediately
after (2026-11-02/03). Progressive lazy loading by LOD 0–4, provider discovery +
fallback, and the runtime scene graph below are now backed by real routes,
migrations, and a frontend runtime client. **Not yet exercised against a live
cluster** — B6 devnet funding blocks that (see `IMPLEMENTATION_STATUS.md`).

**See:** Chapter 12, RFC-013, RFC-014, Chapter 3, Chapter 10.

---

## 13.1 Why this chapter exists

Terra's frontend renders a Cesium globe and defines `LOD_LEVELS` /
`LOD_TARGET_HEIGHT` constants. As of 2026-11-02/03 it now has:

- progressive lazy loading driven by LOD 0–4 (`api/src/routes/runtime.rs` `/runtime/lod`,
  `/runtime/scene/{id}/{lod}` + `terra-web/src/lib/runtime.ts` `lodFromHeight`),
- provider discovery and fallback (`api/src/routes/providers.rs` + `runtime.ts`
  asset retrieval with provider fallback),
- a runtime scene graph with loading-state and validation-state tracking
  (`migrations 0039_runtime_scene_graphs`, `0043_scene_node_validation_state`),
- tier-based building selection (`TIER_SETTINGS.buildingCap`,
  `selectBuildingsForTier`, `distanceDisplayCondition` in `lod.ts` / `perfTier.ts`).

Still missing (tracked separately, not by this chapter): a spatial tiling scheme,
3D Tiles or glTF streaming pipeline, frustum/occlusion culling, and live-cluster
verification of every item above (B6 devnet funding blocks it).

This chapter documents how Terra retrieves only the data needed for the current
view, progressively, from anywhere in the world.

---

## 13.2 Global lazy loading levels

The runtime retrieves only what is needed for the current view, driven by camera
distance, screen-space error, visibility, user interaction, LOD availability,
network conditions, and device capabilities.

| Level | Scope | Loads |
|---|---|---|
| Level 0 — Global overview | Whole globe | Globe representation, major boundaries, low-resolution terrain, global spatial index, region manifests |
| Level 1 — Regional view | Region approaching | Regional manifests, terrain tiles, administrative boundaries, major infrastructure, simplified buildings |
| Level 2 — City / district | District | District manifests, road networks, parcel boundaries, building footprints, simplified models, terrain/infrastructure |
| Level 3 — Parcel / building | Parcel/building | Detailed parcel geometry, building components, roof/facade models, higher-resolution terrain, component metadata, evidence/validation info where authorized |
| Level 4 — Interior / detail | On request | Floors, apartments, rooms, interior models, utilities, detailed meshes, textures, point clouds, BIM data |

A client viewing a city must not download every building's detailed geometry.
A client viewing a specific apartment must not download every component of the
entire city.

## 13.2.1 Diagram — global lazy loading levels

```mermaid
flowchart TB
    subgraph L0["Level 0 — Global overview"]
        L0a[Globe representation]
        L0b[Major boundaries]
        L0c[Low-res terrain]
        L0d[Global spatial index]
        L0e[Region manifests]
    end

    subgraph L1["Level 1 — Regional view"]
        L1a[Regional manifests]
        L1b[Terrain tiles]
        L1c[Admin boundaries]
        L1d[Major infrastructure]
        L1e[Simplified buildings]
    end

    subgraph L2["Level 2 — City / district"]
        L2a[District manifests]
        L2b[Road network]
        L2c[Parcel boundaries]
        L2d[Building footprints]
        L2e[Simplified models]
    end

    subgraph L3["Level 3 — Parcel / building"]
        L3a[Detailed parcel geometry]
        L3b[Roof / facade models]
        L3c[Building components]
        L3d[Higher-res terrain]
        L3e[Evidence / validation info]
    end

    subgraph L4["Level 4 — Interior / detail"]
        L4a[Floors / apartments / rooms]
        L4b[Interior models]
        L4c[Utilities]
        L4d[Detailed meshes / textures]
        L4e[Point clouds / BIM data]
    end

    L0 --> L1 --> L2 --> L3 --> L4
    
    style L0 fill:#e1f5e1
    style L1 fill:#d4e6f1
    style L2 fill:#d4e6f1
    style L3 fill:#fff3cd
    style L4 fill:#fff3cd

---

## 13.3 Spatial request resolution

For every view request, the runtime calculates:

- Camera location.
- Geographic bounding area / visible frustum.
- Required spatial partitions (via the spatial index, not local filtering).
- Visible object IDs.
- Required component versions.
- Required LOD.
- Expected data size, priority, network and memory constraints.

The system should support both **geographic tile retrieval** and
**direct object/component retrieval**.

---

## 13.4 Data discovery across the world

Spatial data may be stored in any region or on any server. Provider discovery
identifies:

- Available replicas and storage locations.
- Provider capabilities, geographic proximity, network latency, reliability.
- Access restrictions, supported transfer protocols, asset version, integrity
  verification.

Selection considers exact-version availability, authorization, latency,
availability, transfer cost, reliability, data-locality requirements, privacy,
and jurisdictional restrictions. Fallback retrieval is defined when the first
provider fails.

---

## 13.5 Progressive rendering

Visualization begins as soon as useful information is available:

1. Display a parcel boundary.
2. Display existing terrain.
3. Display an approximate building footprint or volumetric preview.
4. Display a low-resolution building representation.
5. Load detailed roof and facade components.
6. Load textures.
7. Load interior components only when requested.
8. Replace preliminary geometry with accepted higher-quality versions.

Every visual representation must clearly indicate its status:

- Imported / Observed / Surveyed / Reconstructed / Inferred / Manually corrected /
  Technically validated / Independently validated / Legally recognized (where
  applicable).

**Do not visually communicate that an inferred model is verified physical
reality.**

---

## 13.6 LOD and streaming

Specify:

- Geometry simplification, mesh compression, texture compression.
- Point-cloud decimation, spatial tiling.
- View-dependent loading, frustum culling, occlusion culling.
- Distance-based LOD, screen-space error.
- Cache prioritization, asset prefetching, memory limits.
- Mobile fallback, low-bandwidth fallback, cancellation and retry.

Format evaluation:

| Data type | Recommended format | Notes |
|---|---|---|
| Large geospatial 3D scenes | OGC 3D Tiles | Evaluate for streaming |
| Interoperable model assets | glTF / GLB | Evaluate |
| Point clouds | COPC / LAZ | Evaluate |
| Raster / terrain | Cloud Optimized GeoTIFF | Evaluate |
| BIM | IFC | Evaluate |
| Vector data | GeoJSON | Where appropriate |

**Do not use one format for every data type.**

---

## 13.7 Device-adaptive rendering

Terra must support low-end Android, modern mobile, desktop, professional
workstations, web browsers, and future XR. A device declares its capabilities:
CPU, GPU, memory, storage, network quality, supported formats, rendering
capabilities.

Define capability profiles and adaptive rendering policies:

- Low-end phone: navigate a simplified representation without loading every
  texture or high-resolution mesh.
- Powerful workstation: request more detailed representations.

The data model must remain consistent even when rendering quality differs.

---

## 13.8 Connection to existing code

| Existing capability | Reused by this chapter | Gap |
|---|---|---|
| Frontend `LOD_LEVELS` / `LOD_TARGET_HEIGHT` (8 levels, height breaks) | LOD level definitions | `lodFromHeight` + `TIER_SETTINGS.buildingCap` + `selectBuildingsForTier` now drive progressive tier loading; still no spatial tiling, 3D Tiles, or frustum/occlusion culling |
| `cesiumTerrain.ts` (single terrain provider + retry) | Terrain loading pattern | Provider discovery + fallback now live via `routes/providers.rs` + `runtime.ts`; single terrain provider remains |
| `story/roads.ts` (`buildRoadRoute`, `snapToRoadNetwork`) | Client-side routing baseline | Server-side `/geo/route` absent (Q-007 closed as sufficient) |
| `routes/stories.rs` (`/replay`, `/replay.geojson`, `/schema/replay`) | Read-only replay baseline | No progressive 3D streaming |
| `routes/spatial.rs` (`/parcels/near`) | Spatial query baseline | No hierarchical tiles, no object streaming |
| `routes/geo.rs` (nearest-roads, pois, buildings, stats) | Geo read model | No 3D mesh retrieval, no LOD streaming |
| `api/src/routes/runtime.rs` + migrations 0039/0043 | **New (2026-11-02)** — `/runtime/scene/{id}/{lod}`, `/runtime/assets/{id}` (provider fallback), `/runtime/lod` (LOD 0–4), `/runtime/cache` | — |
| `api/src/routes/providers.rs` + migration 0037 | **New (2026-11-02)** — provider discovery + fallback + replication | — |
| `terra-web/src/lib/runtime.ts` + `runtime.test.ts` | **New (2026-11-02/03)** — frontend runtime client: LOD 0–4, provider fallback, asset retrieval, validation-state tracking | — |

The specification is implemented. Remaining work is the spatial tiling / 3D Tiles
pipeline, frustum/occlusion culling, and live-cluster verification (B6 devnet
funding).