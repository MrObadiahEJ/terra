# Chapter 7 — Off-Chain PostGIS Mirror & Geo-Engine

**Status:** `PARTIALLY IMPLEMENTED` — the mirror, reachability, and canonical
digest are `IMPLEMENTED AND VERIFIED`; server-side two-point routing and
per-row hash anchoring are not.
**See:** [INT-004](../intents/INT-004-gis-bridge.md),
[REQUIREMENTS_TRACEABILITY.md](../REQUIREMENTS_TRACEABILITY.md) REQ-013.

---

## 7.1 The problem

Conventional GIS tooling cannot query a Solana account, and putting large
spatial datasets on-chain is impractical — Failure 4 in
[Chapter 1](01-core-identity-and-philosophy.md). PropTech, planning, and
permitting tools need standard spatial queries over verified data.

## 7.2 Desired outcome

Every on-chain account has an off-chain PostGIS mirror so standard GIS can
query Terra, while the **chain remains the source of truth** and the mirror
stays verifiable against on-chain commitments (ADR-003).

## 7.3 What is built (`IMPLEMENTED AND VERIFIED`)

| Piece | Evidence |
| --- | --- |
| ~25 REST mirror routers | `routes/mod.rs` (parcels, identities, vaults, disputes, escrows, rights, cross-border, subdivision, staking, zk, spatial, spatial-accounts, geo, fusion, world-registry, recovery, credentials, evidence, reconstruction, tx, demo, activity) |
| PostGIS architecture | migrations `0017_postgis_architecture.sql`, `0004`, `0030` |
| Geo endpoints | `routes/geo.rs`: `nearest-roads`, `pois`, `buildings`, `stats` (L88–93) |
| Sealed-road reachability (Dijkstra) | `geo-engine/src/reach.rs` — O(1) per-vertex sealed checks; Dijkstra to nearest sealed vertex |
| Canonical geometry digest | `terra-geom-v1` (commit `775e20a`); `docs/canonical-geometry-digest.md` |
| Client-side road-graph routing | `terra-web/src/story/roads.ts` (`parseRoadSegments`, `snapToRoadNetwork`, `buildRoadGraph`, `routeOnRoads`, `buildRoadRoute`); vitest roads suite |

## 7.4 The honest gaps (`PARTIALLY IMPLEMENTED`)

1. **No server-side two-point routing (`GET /geo/route`).** Routing currently
   runs **in the browser** (`story/roads.ts`). External consumers (robotics,
   API clients — REQ-018) cannot reuse it. Open question Q-007.
2. **`graph.rs` lacks node-id topology.** `RoadSegment` carries a LineString
   without the node adjacency needed for server-side Dijkstra between two
   arbitrary points (reach.rs does nearest-sealed-vertex, not A→B).
3. **Per-row mirror hash anchoring is not implemented.** The mirror is a read
   model whose rows are not individually committed on-chain → mirror drift is a
   real risk (RISK R-004).
4. **IPFS mirror is partial** (migration `0018`); storage-provider independence
   is still a goal (Q-008).

## 7.5 What must stay off-chain vs on-chain (ADR-003)

- **Off-by-default:** imagery, documents, point clouds, 3D models, large
  geometries.
- **Committed on-chain:** canonical hashes, ownership, versions, validator
  attestations, access hashes.
- **Never:** dependence on one commercial storage/cloud/hardware provider.

## 7.6 Acceptance criteria (to reach full `IMPLEMENTED AND VERIFIED`)

- [ ] `GET /geo/route` with honest `method` label (`road-graph` success,
      `great-circle` fallback) — or a recorded decision that client-side
      routing suffices (close Q-007).
- [ ] `graph.rs` node-id topology + A→B Dijkstra with tests.
- [ ] Mirror audit tool comparing mirror rows against on-chain hashes (closes
      R-004).
- [ ] Storage independence decision recorded (Q-008).
