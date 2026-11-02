# Implementation Preparation Plan

**Status:** `SPECIFIED ONLY` — code-area mapping, no implementation code.
**Purpose:** For each missing requirement, identify the likely code areas that will
need work (smart contracts, DB migrations, backend services, API routes, worker
processes, spatial engine modules, frontend components, runtime modules,
storage interfaces, tests, documentation), and which changes are foundational
vs dependent on earlier work.

**See:** REQUIREMENTS_TRACEABILITY.md (REQ-020…REQ-028), Chapter 11, Chapter 12,
Chapter 13, RFC-015.

---

## 1. REQ-026 — Distributed spatial component package & assembly

**Status:** `SPECIFIED ONLY`. No code exists for the component package schema,
conformance test, component registry, or assembly system.

### Foundational changes (must come first)

| Code area | Change | Depends on |
|---|---|---|
| `docs/rfc-015-spatial-component-package.md` | Schema already specified; needs conformance validator implementation | — |
| `terra-core/geo-engine/src/` | Add `terra-geom-v2` digest canonicalization (3D + elevation ref + vertical units) — integrate-or-deprecate decision first (ADR-016) | REQ-026 |
| `terra-core/programs/terra_registry/src/spatial_asset.rs` | Extend `GeometryVersion` / add `ComponentVersion` PDA with `component_id`, `physical_entity_id`, `relationships`, `storage_reference`, `digest_algorithm_version` | RFC-015 schema |
| `terra-core/api/migrations/` | New migration: `component_packages` table + `component_dependencies` + `component_versions` | SpatialAsset change |
| `terra-core/api/src/routes/` | New `components` router: `POST /components` (submit package), `GET /components/{id}` (retrieve), conformance check middleware | Migration + RFC-015 |

### Dependent changes (come after foundational)

| Code area | Change | Depends on |
|---|---|---|
| `terra-core/api/src/routes/reconstruction.rs` | Route reconstruction jobs through component packages; `complete_job` writes a component package instead of a bare draft | Components router |
| `terra-core/api/src/routes/derived.rs` | Reuse `processing_key` dedup for identical component packages | Components router |
| `terra-web/src/lib/` | Frontend component-package client (submit, retrieve, validate digest) | Components router |
| `docs/book-of-intents/chapters/11-*.md` | Assembly spec already written; needs implementation ticket | Components router + assembly |

### Acceptance evidence (when implemented)
- Two different workers submit valid component packages with different internal engines
- Packages pass the conformance validator
- Component versions anchor to `SpatialAsset`/`GeometryVersion` on-chain

---

## 2. REQ-027 — World Representation Manifest & central coordination

**Status:** `SPECIFIED ONLY`. No manifest resolver, spatial index service, or manifest storage exists.

### Foundational changes

| Code area | Change | Depends on |
|---|---|---|
| `docs/book-of-intents/chapters/12-*.md` | Manifest architecture already specified; founder decision on one-root vs distributed registry needed first | — |
| `terra-core/api/migrations/` | New migration: `world_manifests` table + `manifest_versions` + `manifest_objects` | Founder decision |
| `terra-core/api/src/routes/world-registry/` | Extend existing `world_registry` router or add `manifests` router: `GET /manifests/{region}`, `PUT /manifests/{region}` | Migration |
| `terra-core/api/src/routes/spatial.rs` | Spatial index service: hierarchical spatial index (quadtree/S2/H3 decision needed) for partition queries | Migration |

### Dependent changes

| Code area | Change | Depends on |
|---|---|---|
| `terra-core/api/src/routes/reconstruction.rs` | Assembly resolver: resolve component versions + dependencies for an assembly | Manifest router |
| `terra-web/src/lib/` | Runtime: manifest resolver, component resolver, scene graph manager, LOD manager | Manifest router + spatial index |
| `terra-core/api/src/routes/derived.rs` | Cache invalidation propagation on manifest version change | Manifest router |

### Acceptance evidence
- Client discovers a building's components via manifest without downloading all geometry
- Manifest version transition keeps old clients on consistent assembly

---

## 3. REQ-028 — Progressive delivery & runtime discovery

**Status:** `SPECIFIED ONLY`. No provider discovery, fallback, tiling/streaming, or progressive LOD exists beyond frontend `LOD_LEVELS` constants.

### Foundational changes

| Code area | Change | Depends on |
|---|---|---|
| `terra-web/src/lib/lod.ts` | `LOD_LEVELS` already defined; needs integration with actual tile streaming | — |
| `terra-web/src/lib/cesiumTerrain.ts` | Single terrain provider; needs provider discovery + fallback | — |
| `terra-core/api/migrations/` | New migration: `storage_providers` table + `provider_replicas` + `provider_capabilities` | — |
| `terra-core/api/src/routes/` | New `storage-providers` router: `GET /providers`, `GET /providers/{id}/replicas` | Migration |

### Dependent changes

| Code area | Change | Depends on |
|---|---|---|
| `terra-web/src/lib/` | Asset retrieval service: range requests, cache prioritization, prefetching, cancellation/retry | Providers router |
| `terra-web/src/lib/` | Provider selection: exact-version availability, authorization, latency, cost, reliability | Asset retrieval |
| `terra-core/api/src/routes/` | Content-addressed retrieval + signed manifests | Providers router |
| `terra-web/src/` | Frustum culling, occlusion culling, screen-space error LOD | Asset retrieval |

### Acceptance evidence
- Runtime retrieves components from separate storage providers
- Runtime does not download unrelated geographic data
- Viewer displays low-detail data before high-detail finishes loading
- Storage-provider failure does not blank out an entire region

---

## 4. REQ-020 / REQ-021 / REQ-022 — Institutional identity, residency credential, lien encumbrance

**Status:** `SPECIFIED ONLY` with founder review flags (Q-003, Q-004). Q-005 hard-gates REQ-023.

### Foundational changes (blocked on founder decisions)

| Code area | Change | Blocker |
|---|---|---|
| `terra-core/programs/terra_registry/src/` | `register_institutional_identity` instruction + `InstitutionalValidator` PDA | Q-003 (bootstrap council — founder) |
| `terra-core/programs/terra_registry/src/` | Residency credential instruction + dual-signed attestation | Q-004 (jurisdiction recognition — founder) |
| `terra-core/programs/terra_registry/src/` | Lien encumbrance lifecycle (builds on RRR Rights PDA A3) | Q-003 (depends on Ch8 institutional identity) |
| `docs/book-of-intents/chapters/08-*.md` | Institutional identity spec already written; needs founder authorization | Q-003 |

### Acceptance evidence
- None until Q-003/Q-004 resolve; no code before founder authorization

---

## 5. Implementation order (dependency-ordered)

Per ROADMAP Stage 3–4 and Chapter 47:

1. **Founder decisions first:** Q-003 (bootstrap council), Q-004 (residency recognition) — unblock REQ-020/021/022 specs.
2. **Component package:** RFC-015 conformance validator + `component_packages` migration + `components` router (REQ-026 foundational).
3. **3D digest:** Integrate-or-deprecate `terra-geom-v2` (ADR-016 decision) — needed for 3D component digests.
4. **Manifest:** `world_manifests` migration + manifest router + spatial index (REQ-027 foundational).
5. **Providers:** `storage_providers` migration + providers router + provider discovery (REQ-028 foundational).
6. **Assembly:** assembly resolver + conflict detection + incremental assembly (REQ-026 dependent).
7. **Runtime:** scene graph + LOD manager + asset retrieval + progressive streaming (REQ-028 dependent).
8. **Frontend:** integrate `LOD_LEVELS`/`cesiumTerrain.ts` with actual tile streaming (REQ-028 dependent).

### No-funding work available now
All of the above is documentation/specification except items that require code. The workable non-funding items are:
- RFC-015 conformance validator (could be written as a standalone Rust/TS validator)
- Spatial index design decision (quadtree/S2/H3) — documented, not coded
- Manifest architecture founder decision — documented, pending

---

## 6. Code-area inventory (what exists to build on)

| Area | Existing | Status | Can build on? |
|---|---|---|---|
| SpatialAsset / GeometryVersion | `spatial_asset.rs` (on-chain, append-only, quorum verify) | IMPLEMENTED AND VERIFIED | Yes — component version anchors |
| ThreeDModel | `three_d_model.rs` (register + verify, off-chain URI) | IMPLEMENTED AND VERIFIED | Partial — model reference pattern only |
| Reconstruction API | `routes/reconstruction.rs` (jobs, lease, complete, draft, anchor) | PARTIALLY IMPLEMENTED | Yes — component submission baseline |
| Derived compute | `routes/derived.rs` (processing_key dedup, recursive-CTE invalidation) | IMPLEMENTED AND VERIFIED | Yes — dedup pattern for component packages |
| Task economics | `task_economics.rs` (FeePolicy, CoverageIncentive, TaskEscrow, RewardAllocation) | IMPLEMENTED AND VERIFIED | Partial — verification tasks only; needs reconstruction task types |
| Routing | `routing.rs` (nine-factor, seeded randomness) | IMPLEMENTED AND VERIFIED | Partial — needs reconstruction-task routing |
| Geo-engine | `canonical.rs` (terra-geom-v1, 2D), `reach.rs`, `graph.rs`, `osm.rs` | IMPLEMENTED / PARTIAL | Yes — 2D digest baseline; needs 3D (terra-geom-v2) |
| Frontend | `lod.ts` (LOD_LEVELS), `cesiumTerrain.ts` (terrain + retry), story engine | IMPLEMENTED AND VERIFIED | Partial — LOD defined but no streaming/tiling |
| API routers | ~26 routers in `routes/mod.rs` | IMPLEMENTED AND VERIFIED | Yes — add components/manifests/providers routers |
| Migrations | 0001–0034 chain | IMPLEMENTED AND VERIFIED | Yes — add component/manifest/provider migrations |

---

## 7. What must NOT be built yet

| Item | Reason |
|---|---|
| Reconstruction marketplace (offers/bids/settlement for reconstruction tasks) | `task_economics` applies to verification tasks only; new task types needed first |
| Speculative Terra token | Explicitly not a prerequisite (directive §41) |
| Mainnet deployment | Founder decision only (Q-006), after devnet soak + adversarial slashing simulation + external security review |
| `terra-geom-v2` in production | Integrate-or-deprecate decision pending (ADR-016); experimental branch-only |
| Fractional investment (REQ-023) | Hard-gated on Q-005 securities research |
| Institutional identity / residency / liens implementation | Founder review flags pending (Q-003/Q-004) |
