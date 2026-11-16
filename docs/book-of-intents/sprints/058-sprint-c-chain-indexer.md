# Sprint C: Chain indexer and mirror audit

**Date:** planned · **Status:** planned
**Exit gate:** a parcel created with the CLI appears in the API/globe with no
client POST; a hand-edited DB row is flagged by `/mirror/audit`.

## Trigger

Mirror tables are currently fed by client POSTs. The API cannot distinguish
what is on chain from what a client claims, and `/mirror/audit` cannot prove
chain agreement.

## Scope

1. **`api/src/indexer.rs`** + an RPC client held in `AppState`.
2. Poll `getProgramAccounts` / `getMultipleAccounts` for registry-owned
   accounts; decode Anchor accounts from the committed IDL (`idl/`).
3. Upsert mirror rows for parcels, rights, claims, evidence, attestations,
   tasks, assignments, geometry versions, snapshots, escrows and disputes.
   Every row records `slot`, `signature` and `chain_verified`.
4. Client-POSTed rows stay **unverified** until the indexer observes the same
   state on chain.
5. Extend `/mirror/audit` to compare DB rows against chain for claims,
   evidence, attestations and tasks; report mismatches (missing on chain,
   divergent fields, stale slot).
6. Cover the indexer with a DB-gated test that feeds a decoded account and
   asserts the upsert + audit result.

## Acceptance

- CLI-created parcel (no API POST) appears in `/parcels` and on the globe
  after one indexer pass.
- A row edited directly in PostgreSQL is reported as `chain_mismatch` by
  `/mirror/audit`; an indexer pass restores it.
- No client write is marked `chain_verified` before the chain confirms it.

## Blocked by

Nothing; uses the Sprint A local cluster. Devnet slot/signature evidence waits
for Sprint D.
