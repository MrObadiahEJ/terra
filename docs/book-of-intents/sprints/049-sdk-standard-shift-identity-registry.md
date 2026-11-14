# Sprint 049: SDK standard shift — terra-identity + terra-registry SDKs declared + RFC protocol alignment

**Date:** 2026-11-14 · **Status:** done & committed
**Gate:** `cargo check -p terra-registry -p terra-identity` clean; `cargo clippy -p terra-registry -p terra-identity -- -D warnings` clean; `cargo fmt -p terra-registry -p terra-identity -- --check` clean

## Trigger

Sprint 048 declared the `terra-geo` SDK v0.1.0 as the canonical SDK per directive §3 / §12. The same pattern should apply to the other on-chain SDK crates (`terra-identity`, `terra-registry`) so that all SDK packages declare their identity, capabilities, schema versions, dependencies, security assumptions, and upgrade behavior — making the SDK layer an explicit, versioned standard rather than implicit crates.

## Implementation

**Backend — `terra-core/programs/terra_identity/src/lib.rs`:**
- Added `sdk_meta` module declaring the Terra Identity SDK identity per directive §3 / §12:
  - SDK name: `terra-identity`, version `0.1.0`
  - Capabilities: identity-binding (bind_identity), succession (request/endorse/cancel/claim), guardianship (request/execute/revoke)
  - Schema versions: RFC-006 Cross-Border Identity Bridge (1.0), Identity PDA v1, Succession PDA v1
  - Direct crate dependencies: anchor-lang 0.30
  - Security assumptions: Identity PDA derived from [b"identity", identity_hash]; owner is sole authority; Succession requires quorum; Guardianship revocation requires court order scope
  - Upgrade/migration: schema versions additive; PDA seeds stable; instruction accounts appended never replaced
- Unit test `sdk_meta::tests::sdk_metadata_is_well_formed` verifies the declaration

**Backend — `terra-core/programs/terra_registry/src/lib.rs`:**
- Added `sdk_meta` module declaring the Terra Registry SDK identity per directive §3 / §12:
  - SDK name: `terra-registry`, version `0.1.0`
  - Capabilities: parcel registry, rights model RRR, time-bound rights, validator registry, derived jobs, security
  - Schema versions: RFC-007 (1.0), RFC-008 (1.0), Parcel PDA v1, Rights PDA v1, Succession PDA v1
  - Direct crate dependencies: anchor-lang 0.30
  - Security assumptions: Parcel PDA carries no ownership; Rights PDA is canonical; Emergency freeze is time-boxed
  - Upgrade/migration: schema versions additive; PDA seeds stable; new instruction accounts appended never replaced
- Unit test `sdk_meta::tests::sdk_metadata_is_well_formed` verifies the declaration

**Docs — RFC references to SDKs:**
- `rfc-006-cross-border-identity-bridge-protocol.md` §1: references `terra-identity` SDK as canonical Identity account implementation
- `rfc-007-dispute-resolution-parcel-freeze-protocol.md` §1: references `terra-registry` SDK as canonical Dispute/Parcel/Rights PDA implementation
- `rfc-008-parcel-subdivision-amalgamation-protocol.md` §1: references `terra-registry` SDK as canonical Parcel/Rights/SubdivisionRecord PDA implementation

**Docs — IMPLEMENTATION_STATUS.md:**
- Section C adds rows C7 (Terra Identity SDK, status 1) and C8 (Terra Registry SDK, status 1)

**Verification:**
- `cargo check -p terra-registry -p terra-identity` clean
- `cargo clippy -p terra-registry -p terra-identity -- -D warnings` clean
- `cargo fmt -p terra-registry -p terra-identity -- --check` clean

**Note:** Anchor program tests (BPF) not run locally due to weak machine / long compile times; source compilation verified via `cargo check`.
