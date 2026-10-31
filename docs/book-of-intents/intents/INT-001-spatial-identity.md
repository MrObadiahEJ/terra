# INT-001 — Persistent spatial identity for real-world entities

**Status:** `IMPLEMENTED AND VERIFIED` (see [IMPLEMENTATION_STATUS.md](../IMPLEMENTATION_STATUS.md) A2–A21)
**Chapter:** [1 — Core Identity & Philosophy](../chapters/01-core-identity-and-philosophy.md)

## Desired outcome

Any real-world entity — a person, a parcel, a building, a device, an
institution — can hold a persistent, verifiable spatial identity on Solana with
a history that anyone can inspect and no one can forge.

## Boundary

Terra records *identity and evidence about* an entity. It does not assert legal
title and is not a court of record.

## Rationale (pain it removes)

The institutional identity void (Failure 3) and the ground-truth gap (Failure 2):
without persistent, verifiable identity, physical-world attestations cannot be
trusted or composed across systems.

## Evidence

- `terra-core/programs/terra_registry/src/spatial_asset.rs`, `spatial_snapshot.rs`, `world_registry.rs`
- `terra-core/programs/terra_registry/src/device_identity.rs`, `cross_border.rs`
- Migrations `0030_spatial_accounts_mirror.sql`
- Suites: registry lib ≥ 153, BPF integration ≥ 307 (see §1.6 of Chapter 1)
