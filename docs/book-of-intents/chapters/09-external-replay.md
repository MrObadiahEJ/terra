# Chapter 9 — External Read-Only Story Replay

**Status:** `IMPLEMENTED AND VERIFIED` — RFC-014 Phase 05 (REQ-018).
**See:** [REQUIREMENTS_TRACEABILITY.md](../REQUIREMENTS_TRACEABILITY.md)
REQ-018, [IMPLEMENTATION_STATUS.md](../IMPLEMENTATION_STATUS.md) B9,
[sprints/005](../sprints/005-external-replay.md).

---

## 9.1 The problem

The story engine produces deterministic spatially-anchored narratives for the
interactive player. Robotics fleets, GIS pipelines, and third-party API
consumers cannot drive a browser: they need the same timeline as stable,
machine-readable data — with no ambiguity about what is provenance and what
is presentation.

## 9.2 The contract

Three read-only, unauthenticated endpoints under `/api/v1`:

| Endpoint | Returns |
| --- | --- |
| `GET /stories/{id}/replay` | Replay document (`application/json`) |
| `GET /stories/{id}/replay.geojson` | GeoJSON `FeatureCollection` (`application/geo+json`) |
| `GET /stories/schema/replay` | JSON Schema (draft 2020-12) for the replay document |

Replay document (`schema: "terra.story-replay/v1"`):

- `read_only: true` — replay never writes state.
- `story_id`, `story_type`, `title`, `description`, `schema_version`,
  `duration_s`.
- `coordinate_system: "WGS84 (EPSG:4326); heights in metres"`.
- `definition_hash` — SHA-256 of the stored story definition JSON. A consumer
  that pins a hash detects any content change under a stable story id.
- `timeline[]` — stops ordered by `time_s`, each with `index` (definition
  position), `kind` (`start`/`waypoint`/`arrival`/`version`), `position`
  `{lon, lat, height_m}`, optional `heading_deg`/`pitch_deg`, and optional
  `parcel_id`/`asset_pubkey`/`geometry_version`.
- `camera_track`, `routes[]`, `captions[]` — the player's exact inputs, so a
  robot can reproduce the camera or the route geometry.

## 9.3 GeoJSON mapping

- Each stop → `Feature` with `Point` geometry `[lon, lat, height_m]` and
  properties `kind: "stop"`, `stop_id`, `stop_kind`, `label`, `time_s`,
  heading/pitch, and entity references when present.
- Each route → `Feature` with `LineString` geometry `[lon, lat]` pairs and
  properties `kind: "route"`, `route_id`, `method`
  (`road-graph`/`great-circle`/`drawn`), `length_m`, `label`.

Heights ride in the third coordinate slot; GeoJSON permits mixed dimension
per feature, so routes stay 2D while stops carry altitude.

## 9.4 Stability policy

`terra.story-replay/v1` is **additive-only**: fields may be added, never
removed or retyped, within v1. A breaking change mints `v2` at a new path
(`/stories/{id}/replay-v2`, `/schemas/story-replay-v2.json`) while v1 keeps
serving. The schema endpoint's `$id` is the pinning anchor for consumers.

## 9.5 What this is not

- Not chain state and not a proof: `definition_hash` proves *which document*
  was served, not that any parcel, right, or geometry claim in it is true.
- Not an access control boundary: stories are already public presentation
  data. Attributed stories still only accept author-signed mutations.
- Not a server-side renderer: video export remains client-side (Chapter 3 /
  REQ-017); replay is data, not pixels.

## 9.6 Acceptance evidence

- Unit tests (4): replay versioning/read-only flag, timeline ordering with
  definition index, GeoJSON stop points + route lines, schema pinning/links
  — `cargo test -p terra-api` **121/121** (2026-11-02).
- Live smoke: story created → replay document fields verified → GeoJSON
  served with `content-type: application/geo+json` → schema `$id` pinned →
  updating the story changed `definition_hash` → missing ids return 404.
