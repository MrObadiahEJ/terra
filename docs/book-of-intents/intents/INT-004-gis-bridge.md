# INT-004 — Bridged GIS interoperability

**Status:** `IMPLEMENTED AND VERIFIED` — all code gaps closed. See
[IMPLEMENTATION_STATUS.md](../IMPLEMENTATION_STATUS.md) B1, B7, C1–C3.
**Chapter:** [7 — PostGIS Mirror & Geo-Engine](../chapters/07-postgis-geo-engine.md) *(planned)*

## Desired outcome

Every on-chain account has an off-chain PostGIS mirror so that conventional GIS
tooling can query Terra state, while the chain remains the source of truth.
Spatial queries run in pure Rust without external geo services.

## Boundary

The mirror is a *read model*. On-chain state wins conflicts; mirror rows should
be verifiable against on-chain commitments (full per-row hash anchoring not yet
implemented).

## Rationale (pain it removes)

Development economics & soft costs (Failure 4): fragmented, unverifiable GIS
data makes planning and permitting slow. A mirrored, anchored dataset removes
that friction.

## Evidence (what exists / what is missing)

**Exists (`IMPLEMENTED AND VERIFIED` pieces):**
- Sealed-road reachability via Dijkstra — `terra-core/geo-engine/src/reach.rs`
  (O(1) sealed checks; Dijkstra to nearest sealed vertex); geo suite ≥ 6.
- Canonical geometry digest `terra-geom-v2` — v1 baseline commit `775e20a`, v1 removed in Sprint 053;
  `docs/canonical-geometry-digest.md`.
- PostGIS mirror surface — ~25 API routers (`routes/mod.rs`); migrations
  0004/0017/0030; API suite.
- Client-side OSM road-graph routing with honest method labels —
  `terra-web/src/story/roads.ts` (vitest roads suite) — journeys route on the
  road network, but the graph is built **in the browser**, not served by the API.

**Missing:** none — all INT-004 code gaps are implemented:
- Server-side two-point routing (`GET /geo/route`) — implemented
  (`geo-engine/src/reach.rs` `route_between` Dijkstra; API
  `routes/geo.rs`); Q-007 resolved.
- `graph.rs` `RoadSegment` node-id topology (`from_node_id`/`to_node_id`) +
  A→B Dijkstra with tests — implemented; Q-007 resolved.
- Per-row mirror hash anchoring — implemented (migration 0048 + triggers +
  `GET /mirror/audit`); closes RISK R-004.
- IPFS mirror / storage independence — implemented (migration 0018 mirror
  tables served by `GET /providers/authority-registries` and
  `GET /providers/ipfs-docs`; `storage_providers`/`provider_replicas` from
  migration 0037; proof-of-possession from migration 0046; `content_address`
  binding from migration 0049; reliability derived from challenge results in
  `prove_replica`; `GET /providers/replication-audit` enforces N-of-M ≥2
  provider classes); Q-008 resolved.
