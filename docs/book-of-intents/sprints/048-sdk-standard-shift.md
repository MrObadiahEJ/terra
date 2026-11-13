# Sprint 048: SDK standard shift — terra-geo SDK v0.1.0 declared + RFC-015/protocol alignment

**Date:** 2026-11-13 · **Status:** done & committed (`609d9f0`)
**Gate:** `cargo check --workspace` clean; `cargo clippy --workspace -- -D warnings` clean; `cargo fmt --all --check` clean; `cargo test -p terra-geo --lib` 49 passed (incl. new `sdk_metadata_is_well_formed`).

## Trigger

The TERRA_PROTOCOL_DIRECTIVE.md §3 (MODULAR SDK — Independent Capabilities) and
§12 (Rust-First SDK Strategy) require the SDK to be an explicit, versioned,
modular standard with each module declaring capabilities, schema versions,
dependencies, security assumptions, and upgrade/migration behavior. The
`geo-engine` crate (package `terra-geo` v0.1.0) already contained the SDK core
(canonical digests, component-package conformance validator, signed manifests,
quadtree discovery) but did not declare itself as an SDK. RFC-015 referenced
`geo-engine/src/component_package.rs` without naming it as "the SDK" with a
version. Sprint 048 closes this gap.

## Implementation

**Backend — `terra-core/geo-engine/src/lib.rs`:**
- Added `sdk_meta` module declaring the SDK identity per directive §3 / §12:
  - SDK name: `terra-geo`, version `0.1.0`
  - Capabilities: geometry canonicalization + digest (terra-geom-v1 / terra-geom-v2); RFC-015 Spatial Component Package conformance validator (`validate`, `validate_with_geometry`); content-addressed signed manifests (`ContentAddress`, `ManifestPayload`, `SignedManifest`); quadtree global-discovery partitions (ADR-020); spatial access, road graph, OSM data, POIs, reachability
  - Schema versions: RFC-015 (1.0), terra-geom-v1 (1), terra-geom-v2 (2), terra.manifest/v1 (1)
  - Direct crate dependencies: geo, geo-types, serde, serde_json, anyhow, ed25519-dalek, osmpbf, sha2, thiserror
  - Security assumptions: content-addressed artifact storage (SHA-256); Ed25519 domain-separated signatures; digest integrity verified before trust
  - Upgrade/migration: schema versions additive; digest algorithm versions declared per-artifact and never silently changed
- Re-exports `sdk_capabilities`, `sdk_name`, `sdk_schema_versions`, `sdk_security_assumptions`, `sdk_upgrade_migration`, `sdk_version`
- Added unit test `sdk_meta::tests::sdk_metadata_is_well_formed` verifying name/version/capabilities/schema versions are non-empty and well-formed

**Docs — `docs/rfc-015-spatial-component-package.md`:**
- §5 table updated: `canonical3d.rs` (`terra-geom-v2`, SDK `terra-geo` v0.1.0) marked production (was "branch-only, experimental"); `geo-engine/src/component_package.rs` (`terra-geo` SDK v0.1.0) marked as the canonical RFC-015 conformance validator implementation (no gap)
- Added §5.1 "SDK standard declaration" naming `terra-geo` SDK v0.1.0 as the canonical implementation with capabilities, schema versions, security assumptions, upgrade/migration
- §6 open decisions updated: terra-geom-v2 promotion resolved (production); conformance-validator location resolved (`geo-engine/src/component_package.rs` in SDK)

**Docs — `docs/book-of-intents/chapters/11-distributed-spatial-reconstruction-and-assembly.md`:**
- §11.3 references the `terra-geo` SDK v0.1.0 as the canonical RFC-015 conformance validator implementation

**Docs — `docs/book-of-intents/IMPLEMENTATION_STATUS.md`:**
- Section C adds row C6 (SDK standard declaration, status 1): SDK metadata declaration, capabilities, schema versions, dependencies, security assumptions, upgrade behavior

**Verification:**
- `cargo check --workspace` clean
- `cargo clippy --workspace -- -D warnings` clean
- `cargo fmt --all --check` clean
- `cargo test -p terra-geo --lib` 49 passed (incl. new `sdk_metadata_is_well_formed`)

## Remaining RFC-016 Phase 2+ code (backend, unblocked)

- `SecurityRequest.subject` still `Pubkey::default()` — fill via instruction param
- `PrivacyState` writes on finalize
- Provider proof-of-possession audits
- Client-side digest verification vs signed manifest (frontend: `fetchComponentAsset` already wired)
- `POST /components` auth
- `complete_assignment` auth

These remaining items are backend (Rust) work requiring heavier builds; the frontend data-plane artifact path is already wired and verified.
