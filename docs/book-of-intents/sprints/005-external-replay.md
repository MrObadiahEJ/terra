# Sprint 005 — RFC-014 Phase 05: external read-only replay (REQ-018)

**Loop:** Research → Understand → Compare → Identify Gaps → Implement → Validate → Record
**Phase derived from evidence:** Test (validated) → Loop (this record)

## Research

- Story engine (Phases 01–03) serves the interactive player and client-side
  export; robotics/GIS/API consumers had no machine-readable surface
  (REQ-018, ROADMAP Stage 1 item 3).
- The story definition is the contract already stored in JSONB with
  `schema_version`; reusing it avoids a second source of truth. Replay must
  stay presentation data (ADR: no chain writes from story reads).

## Plan

- `GET /stories/{id}/replay` — versioned JSON document
  (`terra.story-replay/v1`): `read_only: true`, `definition_hash` pin,
  time-ordered timeline with explicit `position`, camera track, routes,
  captions.
- `GET /stories/{id}/replay.geojson` — GeoJSON FeatureCollection (stops →
  Point with altitude, routes → LineString 2D), `application/geo+json`.
- `GET /stories/schema/replay` — JSON Schema 2020-12, `$id` pinned; v1
  additive-only, v2 for breaking changes.
- Pure builders (`build_replay`, `build_geojson`, `replay_schema_doc`) so the
  shapes are unit-testable without a database.

## Build

- `routes/stories.rs`: three routes, replay/GeoJSON types, pure builders,
  `story_for_replay` (loads row + hashes canonical definition JSON), schema
  document, 4 unit tests.

## Test (evidence)

- `cargo test -p terra-api` → **121/121** (2026-11-02), 4 new tests:
  read-only/versioning, timeline ordering with definition index, GeoJSON
  stop points + route lines, schema pinning/links.
- Live smoke: created `phase05-smoke-1` (signed) → replay returned
  `schema`, `read_only`, `definition_hash` (64 hex), ordered timeline;
  GeoJSON served with `content-type: application/geo+json` and Point + route
  LineString features; schema `$id`
  `…/schemas/story-replay-v1.json` with `replay`/`geojson` links; editing the
  story changed `definition_hash`
  (`63d3ca…` → `5c957f…`); unknown ids → 404 on both data endpoints.

## Loop

- REQ-018 exit met; documented as Chapter 09. Book registers updated
  (traceability, status B2, roadmap, current, changelog).
- Next: verification refresh (registry lib/BPF, geo, identity, API) and CI
  Rust-job depth (R-008) → Stage 1 exit. Devnet deploy stays founder-blocked
  (Q-001).
