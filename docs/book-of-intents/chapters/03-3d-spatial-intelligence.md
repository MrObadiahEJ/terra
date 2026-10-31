# Chapter 3 — Multi-Layer 3D Spatial Intelligence

**Status:** `IMPLEMENTED AND VERIFIED` for on-chain 3D model registration,
versioned geometry, and snapshots; `PARTIALLY IMPLEMENTED` for the off-chain
reconstruction pipeline (worker runtime is real, but anchor execution has never
run on a public cluster).
**Evidence:** RFC-013 Phases C/D/E; `spatial_asset.rs`, `three_d_model.rs`,
`spatial_snapshot.rs`; migrations 0029–0031; stage3 BPF suites; commit
`775e20a` (`terra-geom-v1` digest).

---

## 3.1 The problem

2D parcel boundaries cannot represent stacked cities, underground structures,
vertical farming, or volumetric rights. Fragmented reconstruction data (LiDAR,
drones, photogrammetry) has no tamper-evident link to on-chain ownership —
Failure 2 in [Chapter 1](01-core-identity-and-philosophy.md).

## 3.2 Desired outcome

Physical entities have **stable identifiers, versioned 2D/3D geometry,
elevation semantics, temporal history, and cryptographic commitments** — so
applications can reason about *volumetric* space and *verifiable construction
progress*, not just footprints.

## 3.3 What is built (`IMPLEMENTED AND VERIFIED`)

| Piece | Evidence |
| --- | --- |
| Spatial assets with append-only geometry versions | `init_spatial_asset`, `append_geometry_version`, `append_geometry_version_from_evidence`, `verify_geometry_version` (lib.rs L1909–1976) |
| 3D model registration with format label | `register_3d_model`, `verify_3d_model` (L1979–2001); `three_d_model.rs` |
| Spatial snapshots | `create_spatial_snapshot` (L2004); `spatial_snapshot.rs` |
| Elevation envelope rules | elevation_range_ok / none-is-zero / matches-dimension / within-asset; version dimension ≤ asset dimension (errors 6231–6246) |
| Canonical geometry digest | `terra-geom-v1` (commit `775e20a`); `docs/canonical-geometry-digest.md` |
| BPF lifecycle tests | `stage3_spatial_asset_lifecycle`, `stage3_geometry_version_verification`, `stage3_evidence_linked_geometry`, `stage3_geometry_quorum_config`, `stage3_3d_model_lifecycle`, `stage3_spatial_snapshot_lifecycle` |
| Off-chain mirror | migration `0030_spatial_accounts_mirror.sql`; `/spatial-accounts/*` REST |

**Canonicalization honesty:** the geometry hash commits to a *representation*
under documented normalization rules. It does **not** prove the real-world
boundary is correct — that remains a verification-consensus question (golden
rule, Chapter 1 §1.3).

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

## 3.5 AI-assisted reconstruction (partial)

AI/draft outputs are labeled (`capture-hull` vs fallback) so a consumer can
always tell measured geometry from interpolated geometry. Model provenance
(model, version, confidence) is **not yet recorded** — RISK R-014 / REQ-024.
The protocol must never let an AI output silently become verified geometry.

## 3.6 Acceptance criteria (to reach full `IMPLEMENTED AND VERIFIED`)

- [ ] Devnet: worker lease → complete → anchor transaction executes on-chain.
- [ ] Model provenance fields (model/version/confidence) recorded per draft.
- [ ] Multi-layer nesting (building → floor → room) tested against elevation
      envelope rules.
