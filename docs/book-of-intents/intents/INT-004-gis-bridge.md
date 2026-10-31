# INT-004 — Bridged GIS interoperability

**Status:** `PARTIALLY IMPLEMENTED` (see
[IMPLEMENTATION_STATUS.md](../IMPLEMENTATION_STATUS.md) B1, B7, C1–C3)
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
- Canonical geometry digest `terra-geom-v1` — commit `775e20a`;
  `docs/canonical-geometry-digest.md`.
- PostGIS mirror surface — ~25 API routers (`routes/mod.rs`); migrations
  0004/0017/0030; API suite.
- Client-side OSM road-graph routing with honest method labels —
  `terra-web/src/story/roads.ts` (vitest roads suite) — journeys route on the
  road network, but the graph is built **in the browser**, not served by the API.

**Missing (`PARTIALLY IMPLEMENTED` gap):**
- Server-side two-point routing (`GET /geo/route`) — not implemented; open
  question Q-007.
- `graph.rs` `RoadSegment` carries LineString geometry without node-id topology
  for server-side pathfinding.
- Per-row mirror hash anchoring — not implemented (RISK R-004).
- IPFS mirror (migration 0018) — partial (B7); storage independence pending
  (Q-008).
