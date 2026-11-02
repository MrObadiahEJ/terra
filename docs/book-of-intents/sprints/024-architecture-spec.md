# Sprint 024 — Architecture spec: distributed reconstruction, world runtime, progressive delivery

**Loop:** Research → Understand → Compare → Identify Gaps → Implement → Validate → Record
**Phase derived from evidence:** Specify (no implementation code — Chapters 11–13 are `SPECIFIED ONLY`)

## Research

Inspected `main` (e9b2e7d), `dev` (d293b0e), and `terra-3d-spatial-pipeline` (origin,
bf346f4/bd2cf24 based on pre-constitution `775e20a`). Confirmed:

- `main`/`dev` contain the on-chain spatial stack (`SpatialAsset`,
  `GeometryVersion`, `ThreeDModel`, `SpatialSnapshot`, `task_economics`, `routing`),
  the reconstruction API, derived compute, AI provenance, and story replay.
- `dev` is ahead of `main` with all post-constitution work (constitution,
  RFC-014 Phases 01–05, auth fix R-016/R-017, verification refresh, sprints
  012–023).
- `terra-3d-spatial-pipeline` is **behind** `dev` (based on `775e20a`) and adds
  only `canonical3d.rs` (`terra-geom-v2`, 3D digest) and a `POST /geo/geometry/digest`
  route — treated as **experimental** (ADR-016), not merged.
- No World Representation Manifest, no spatial component package, no assembly
  system, no provider discovery, no progressive 3D streaming, no distributed
  runtime exist in any branch.

## Compare

Mapped every claimed capability to implementation evidence (Chapters 3, 10,
11, 12, 13). Key honest classifications:

- `SpatialAsset`/`GeometryVersion`/`ThreeDModel` → `IMPLEMENTED AND VERIFIED`
  (on-chain anchors exist).
- Reconstruction worker (`routes/reconstruction.rs`) → `PARTIALLY IMPLEMENTED`
  (anchor_prep builds a transaction but never executed on devnet).
- `terra-geom-v1` → `IMPLEMENTED AND VERIFIED` (2D, frozen).
- `terra-geom-v2`/`canonical3d.rs` → `SPECIFIED ONLY` / experimental (branch-only).
- Distributed reconstruction, World Representation Manifest, synchronized
  runtime, progressive delivery → `SPECIFIED ONLY` (Chapters 11–13, this sprint).

## Identify gaps

No component package schema, no conformance test, no assembly system, no spatial
index service, no manifest resolver, no provider discovery/fallback, no 3D
tiling/streaming, no frustum/occlusion culling, no multi-engine support. No
reconstruction marketplace (task_economics applies to verification tasks only).

## Implement → Validate → Record

No code written (doc-only, per master instruction). Chapters 11/12/13 specify:
the component package, control plane/data plane, World Representation Manifest,
spatial index, scene graph, synchronized rendering runtime, lazy loading,
progressive rendering, LOD/streaming formats, and device-adaptive rendering.

## Registers updated

- `REQUIREMENTS_TRACEABILITY.md` — REQ-026/027/028 added (`SPECIFIED ONLY`).
- `BOOK_OF_INTENTS.md` — chapters 11/12/13 added to index; ADR-014/015/016 note.
- `SUMMARY.md` — chapters 11/12/13 + sprint 024.
- `ROADMAP.md` — Stage 3 exit noted; Stage 4 lists REQ-026/027/028.
- `ARCHITECTURAL_DECISIONS.md` — ADR-014 (control/data plane + Manifest),
  ADR-015 (component package + conformance), ADR-016 (terra-geom-v2 experimental).
- `RISK_REGISTER.md` — R-018/019/020/021.
- `work/CURRENT.md` — sprint 024 recorded; next-up updated.
- `CHANGELOG.md` — sprint 024 entry.

## Loop

- All Stage 1 non-chain deliverables complete; CI green.
- Next: founder review of Stage 3 flags (Q-003/Q-004) and Stage 4 boundary
  (REQ-026/027/028). No funding required for doc-only work.
