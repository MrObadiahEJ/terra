# Sprint B: Real client write path

**Date:** planned (starts after Sprint A CI green) · **Status:** planned
**Exit gate:** Playwright E2E completes the full loop on localnet with zero
simulated steps.

## Trigger

Writes still depend on `terra-core/api/src/routes/tx_prep.rs`, a placeholder
server-side transaction builder. The frontend also builds identity PDAs against
the wrong program ID, and the generated IDL/types were not checked in, so the
client could not follow the deployed program.

## Scope

1. **Delete the placeholder builders.** Remove `tx_prep.rs` and its route
   wiring; no server-side instruction construction remains.
2. **Client-side writes only.** Every write uses the generated IDL through
   `program.methods.*` with real discriminators, PDA seeds, account metas and
   signers. Add a localnet-landing test for each write path (the transaction
   must land, not merely serialize).
3. **Fix identity PDAs.** `terra-web/src/lib/program.ts` `identityPda` and
   `successionPda` must derive from the `terra_identity` program ID
   (`HyjpaHY9yQg1KneLZGjpLDQfP9WV4LUEbNZcb7C4Vo6p`), not the registry ID.
4. **IDL/types sync.** Generate `target/types` for both programs, refresh
   `terra-web/src/idl/terra_registry.json` + `terraRegistry.ts`, and make the
   sync part of `make idl`/`build.sh`.
5. **Wire the UI loop.** `/console` and the parcel panels must drive:
   register parcel → rights → evidence upload → pay for verification → task
   routing → status → geometry version → proof export → dispute. Remove any
   remaining simulated data paths from these screens.

## Acceptance

- Playwright E2E starts from a clean Surfpool localnet and completes the loop
  using only the UI; no mocked writes, no `tx_prep`, no “SIMULATION” chips.
- Every transaction in the run lands on chain and is discoverable by signature.
- `identityPda`/`successionPda` match the IDL-derived addresses.

## Blocked by

Nothing. Sprint A provides the deployed local cluster and the lifecycle
reference.
