# Chapter 12 — World Runtime and Synchronized Spatial Rendering

**Status:** `SPECIFIED ONLY` — no implementation code yet.

**See:** Chapter 11, Chapter 13, RFC-013, RFC-014, `docs/canonical-geometry-digest.md`,
Chapter 3, Chapter 10.

---

## 12.1 Why this chapter exists

Terra's frontend can render a Cesium globe and load a small set of story
demos, but it does **not** have:

- a central coordination object / World Representation Manifest,
- a separation of control plane from data plane,
- a spatial index that lets a client discover which objects intersect a view,
- a synchronized runtime that merges components from multiple storage providers
  into one coherent scene,
- progressive lazy loading beyond hardcoded `LOD_LEVELS` constants,
- provider discovery, fallback, or content-addressed retrieval.

This chapter specifies the runtime architecture that makes separately stored
spatial objects feel like one coherent physical world.

---

## 12.2 Central coordination object: World Representation Manifest

Terra must **not** store the entire world's 3D geometry in one central database.
Instead it maintains a logical, versioned **World Representation Manifest** — a
reference structure that coordinates independently stored spatial objects.

A manifest must identify:

- The geographic region or world partition it represents.
- The spatial objects included and their stable IDs.
- Each object's current component version and assembly version.
- Spatial extents (bounding volumes).
- Dependencies between objects and components.
- Storage locations or retrieval references for each component.
- Available levels of detail.
- Validation states and access requirements.
- Update history.
- Parent and neighboring manifests.

**Decision required (founder):** whether this is one global root manifest with
hierarchical regional manifests, or a distributed registry with a logical global
root. Evaluate scalability and consistency implications before choosing
(see Part XII, Founder decisions).

## 12.2.1 Diagram — central coordination & decentralized storage

```mermaid
flowchart TB
    subgraph WM[World Representation Manifest<br>logical, versioned, references only]
        WM1[Region manifests]
        WM2[Object IDs + versions]
        WM3[Assembly relationships]
        WM4[Storage references]
        WM5[LODs + validation states]
    end

    subgraph REG[Spatial object registry<br>control plane]
        R1[Object identity]
        R2[Spatial index]
        R3[Version references]
        R4[Access authorization]
    end

    subgraph SP["Independent storage providers<br>(data plane — geometry, meshes, textures, point clouds)"]
        P1[Provider A: object storage]
        P2[Provider B: CDN]
        P3[Provider C: IPFS / distributed]
        P4[Provider D: commercial cloud]
    end

    subgraph RT[Runtime retrieval<br>client-side]
        RT1[Discover providers]
        RT2[Fetch components]
        RT3[Verify digests]
        RT4[Assemble scene graph]
    end

    WM --> REG
    WM --> SP
    REG --> RT
    SP --> RT
    RT --> WM[read manifest versions]
    
    style WM fill:#e1f5e1
    style RT fill:#d4e6f1
    style P1 fill:#fff3cd
    style P2 fill:#fff3cd
    style P3 fill:#fff3cd
    style P4 fill:#fff3cd

---

## 12.3 Control plane vs data plane

This distinction must be explicit.

### Control plane
- Object identity and spatial registry.
- World manifests, spatial indexing, version references.
- Assembly relationships and component discovery.
- Access authorization and storage-location discovery.
- Task coordination, update notifications, validation states, economic metadata.

### Data plane
- Geometry files, meshes, textures, point clouds, terrain tiles.
- BIM artifacts, photographs, derived spatial datasets, model packages.
- Cached assets and streaming content.

The control plane tells the runtime what exists, how objects relate, and where
data can be retrieved. The data plane supplies the actual content.

**Constraint:** The control plane must not become a bottleneck for every
rendered vertex or texture request. The data plane must support independent
storage, replication, caching, and retrieval.

## 12.3.1 Diagram — complete Terra architecture

```mermaid
flowchart TB
    subgraph Observation["Layer 1 — Physical Observation & Evidence"]
        O1[Smartphones / Cameras / Drones]
        O2[Satellite / LiDAR / GNSS]
        O3[Survey instruments / CAD / BIM]
        O4[GIS / government datasets]
    end

    subgraph Identity["Layer 2 — Spatial Identity & Structure"]
        I1[Stable object IDs]
        I2[Relationship graph]
        I3[Version lineage]
    end

    subgraph Reconstruction["Layer 3 — Spatial Processing & Reconstruction"]
        R1[Photogrammetry / SfM / LiDAR]
        R2[AI-assisted reconstruction]
        R3[GIS / CAD-BIM conversion]
    end

    subgraph Storage["Layer 4 — Spatial Component Storage (independent)"]
        S1[Provider A: object storage]
        S2[Provider B: CDN]
        S3[Provider C: IPFS / distributed]
    end

    subgraph Coordination["Layer 5 — Spatial Registry & Coordination"]
        C1[World Representation Manifest]
        C2[Spatial index]
        C3[Assembly relationships]
    end

    subgraph Assembly["Layer 6 — Spatial Assembly & World Organization"]
        A1[Component registration + alignment]
        A2[Conflict detection]
        A3[Incremental assembly]
    end

    subgraph Runtime["Layer 7 — Distributed Runtime & Synchronization"]
        RT1[Spatial discovery]
        RT2[Manifest resolver]
        RT3[Asset retrieval]
        RT4[Scene graph + LOD manager]
    end

    subgraph Visualization["Layer 8 — Visualization & Interaction"]
        V1[Cesium globe / Leaflet]
        V2[Progressive LOD rendering]
    end

    subgraph Validation["Layer 9 — Validation & Trust"]
        VAL1[Technical verification]
        VAL2[Domain validation]
        VAL3[Reputation / collateral]
    end

    subgraph Economics["Layer 10 — Economic Coordination"]
        EC1[Task escrow + settlement]
        EC2[Coverage investment]
        EC3[Staking / collateral]
    end

    Observation --> Identity
    Identity --> Reconstruction
    Reconstruction --> Storage
    Storage --> Coordination
    Coordination --> Assembly
    Assembly --> Runtime
    Runtime --> Visualization
    Validation -.-> Reconstruction
    Validation -.-> Assembly
    Economics -.-> Reconstruction
    Economics -.-> Runtime

    style C1 fill:#e1f5e1
    style RT1 fill:#d4e6f1
    style V1 fill:#d4e6f1

---

## 12.4 Spatial index and global partitioning

The runtime must efficiently determine which objects intersect a requested
geographic area without downloading all known objects.

Hierarchical spatial indexing must be specified. Candidates:

| Structure | Best for |
|---|---|
| Quadtree | 2D geographic discovery, global queries |
| Octree / BVH | 3D volumes, buildings, point clouds |
| S2 / H3 | Global discovery, hierarchical cells, streaming tiles |
| Geohash | Simple cell-based lookups |
| Hierarchical bounding-volume indexing | Component-level lookup, culling |

Do not adopt an indexing technology merely because it is popular. Define which
structures serve global discovery, geographic queries, 3D volumes, terrain
tiles, buildings, large point clouds, component-level lookup, and runtime
streaming.

**Relationships (distinct concepts):** geographic partitions, spatial objects,
spatial components, storage chunks, visualization tiles, and assembly manifests
are related but must not be confused.

---

## 12.5 Synchronized rendering runtime

A single rendered scene must be able to contain components retrieved from
different storage providers. The renderer must not care whether a component came
from Terra-operated storage, a community storage node, a government source, a
commercial cloud, a distributed object-storage network, or a local device cache.

Runtime sequence for a geographic view request:

1. Receive view request (camera position, extent).
2. Determine geographic extent and visible frustum.
3. Identify relevant spatial partitions via the spatial index.
4. Query the coordination layer for relevant object manifests.
5. Resolve required object and component versions.
6. Discover storage locations (provider discovery).
7. Check local and shared caches.
8. Fetch missing data from appropriate providers.
9. Verify content identity and version (digest check).
10. Apply coordinate transformations (local → global frames).
11. Resolve dependencies and assembly relationships.
12. Assemble the scene graph.
13. Render available components.
14. Continue fetching higher-resolution data when needed.
15. Synchronize updates without corrupting the current view.

---

## 12.6 Logical assembly vs physical storage

Do not require every component of a building, district, or city to be stored
together. A building may consist of:

- A foundation component stored by provider A.
- A roof component stored by provider B.
- A facade component stored by provider C.
- A terrain component stored by provider D.
- An interior component stored by provider E.

The World Representation Manifest and Assembly Manifest describe how these
components connect. The runtime retrieves only the components needed for the
current view and requested LOD. The user still experiences a unified building.

---

## 12.7 Scene graph

The scene graph is the runtime representation of assembled spatial objects. It
must represent: global root, geographic regions, spatial partitions,
administrative areas, terrain, parcels, buildings, building components, floors,
rooms, infrastructure, volumetric objects, relationships, and dependencies.

A node references its component and assembly versions rather than duplicating
all underlying geometry.

Define: parent-child relationships, transform inheritance, local and global
coordinates, visibility, LOD, dependencies, loading states, validation state,
privacy/access state, update state.

The scene graph is generated or resolved from manifests and spatial queries. It
is **not** the permanent source of truth — the manifest is.

---

## 12.8 Runtime communication

Use separate interfaces for separate concerns:

| Concern | Suggested mechanism |
|---|---|
| Object discovery, manifest retrieval, spatial queries | REST/HTTP (control plane) |
| Live status and update notifications | WebSocket or SSE |
| Asynchronous work | Event-driven messaging |
| Large files | Object-storage protocols |
| Partial file retrieval | HTTP range requests |
| Repeated access | CDN or distributed caches |
| Peer-to-peer transport | WebRTC where justified |
| Signed, content-addressed retrieval | Content-addressed manifests |

Define which communication is synchronous, asynchronous, or eventually
consistent. Define message schemas, version compatibility, authentication,
retries, timeouts, idempotency, and error handling.

---

## 12.9 Synchronization and consistency

Handle independently stored and updated objects:

- Authoritative object versions, manifest versions, assembly versions.
- Update events, dependency invalidation, cache invalidation.
- Conflict detection, stale references, concurrent updates, failed updates,
  rollbacks, partial availability, offline operation.

**Incremental update rule:** An update to a roof must not force a complete
reconstruction of the entire building or city:

1. Create a new roof component candidate.
2. Validate it.
3. Identify affected assembly dependencies.
4. Create a new assembly candidate.
5. Recompute only affected relationships or derived outputs.
6. Publish a new manifest version.
7. Notify connected clients.
8. Allow clients to transition safely to the new version.

Existing clients may continue rendering an older consistent assembly while the
newer version is being prepared.

**Consistency model:** Immutable versions and explicit manifest transitions.
Do not assume every spatial update must be globally synchronized at the same
instant.

## 12.9.1 Diagram — component update lifecycle

```mermaid
flowchart TD
    R1[Roof component changed<br>new version candidate]
    R2[Validate component candidate<br>digest, CRS, conformance]
    R3[Identify affected<br>assembly dependencies]
    R4[Create new assembly candidate]
    R5[Recompute only affected<br>relationships / derived outputs]
    R6[Publish new manifest version]
    R7[Notify connected clients]
    R8[Client transitions safely<br>to new version]
    R9[Old clients keep rendering<br>previous consistent assembly]
    
    R1 --> R2 --> R3 --> R4 --> R5 --> R6 --> R7 --> R8
    R6 --> R9
    
    style R1 fill:#e1f5e1
    style R6 fill:#d4e6f1
    style R8 fill:#fff3cd
    style R9 fill:#fff3cd

---

For collaborative annotations and metadata, consider CRDTs only where their
conflict-resolution properties are appropriate. Do not use CRDTs as a
substitute for validating physical geometry.

## 12.10 Runtime failure behavior

The world representation must remain useful when providers or components are
unavailable:

- Missing components → render available data, identify missing components.
- Unavailable storage nodes → provider fallback (Section 12.11).
- Network interruption → serve from cache, degrade gracefully.
- Expired storage references → re-resolve via coordination layer.
- Invalid hashes → reject, request re-delivery.
- Outdated manifests → fetch newer manifest.
- Failed reconstruction jobs → reassign or mark component pending.
- Incomplete assemblies → show available components, mark gaps.
- Permission failures → 403 at access-control layer.
- Unsupported formats → reject at conformance boundary.
- Corrupt assets → digest mismatch → reject.
- Slow providers → timeout and fall back to another provider.
- Conflicting component versions → preserve both, request validation.

Define redundancy and recovery strategies for important spatial assets.

---

## 12.11 Provider discovery and fallback

The runtime must not assume data is stored in one region or one server. Define a
provider discovery mechanism that identifies:

- Available replicas and storage locations.
- Provider capabilities, geographic proximity, network latency, reliability.
- Access restrictions, supported transfer protocols, asset version, integrity
  verification.

Provider selection considers: exact version availability, authorization,
network latency, availability, transfer cost, reliability, data-locality
requirements, privacy and jurisdictional restrictions. **Do not select a
provider solely because it is geographically close.** Define fallback retrieval
when the first provider fails.

## 12.12 Diagram — synchronized rendering runtime

```mermaid
sequenceDiagram
    participant U as User / Camera
    participant R as Runtime (client)
    participant I as Spatial Index (control plane)
    participant M as World Manifest (control plane)
    participant P as Provider Discovery
    participant S1 as Storage Provider A
    participant S2 as Storage Provider B
    participant C as Cache (local/shared)

    U->>R: geographic view request (camera position, extent)
    R->>I: spatial query: visible frustum + partitions
    I-->>R: relevant object IDs + versions
    R->>M: resolve object manifests + assembly versions
    M-->>R: component versions + storage references
    R->>P: discover providers for each component
    P-->>R: providers + latency + authorization
    R->>C: check local + shared caches
    alt cache hit
        C-->>R: component bytes
    else cache miss
        R->>S1: fetch component (range request)
        S1-->>R: bytes + digest
        R->>R: verify content identity & version
    end
    R->>R: coordinate transformation (local→global)
    R->>R: resolve dependencies + assembly relationships
    R->>R: assemble scene graph
    R-->>U: render available components (progressive LOD)
    R->>M: subscribe to update notifications
    M-->>R: new manifest version (incremental update)
    R->>R: safe scene transition (keep old assembly until new valid)
```

## 12.13 Connection to existing code

| Existing capability | Reused by this chapter | Gap |
|---|---|---|
| `routes/derived.rs` (derived_jobs, processing_key dedup) | Derived-job reuse pattern for component reuse | No manifest resolver, no spatial index service |
| `routes/reconstruction.rs` (jobs, drafts, anchor) | Component submission baseline | No multi-provider retrieval, no assembly resolver |
| `routes/stories.rs` (`/replay`, `/replay.geojson`, `/schema/replay`) | Read-only replay baseline | No synchronized 3D scene assembly |
| `routes/spatial.rs` (`/parcels/near`, zones) | Spatial query baseline | No hierarchical spatial index, no provider discovery |
| `routes/geo.rs` (`nearest-roads`, `pois`, `buildings`, `stats`) | Geo read model | No 3D mesh retrieval, no LOD streaming |
| `terra-core/geo-engine` (`canonical.rs`, `reach.rs`, `graph.rs`, `osm.rs`) | Geometry digest, sealed-road reach | No component storage, no spatial index, no scene graph |
| Frontend `lod.ts` (`LOD_LEVELS`, `LOD_TARGET_HEIGHT`) | LOD level definitions | No actual tiling/streaming, no 3D Tiles, no frustum culling |
| Frontend `cesiumTerrain.ts` | Terrain loading with retry | Single terrain provider, no fallback, no component streaming |

No code changes are made in this task. The deliverable is the specification.