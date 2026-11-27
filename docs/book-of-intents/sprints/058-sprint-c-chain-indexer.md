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

## Addendum — 2026-11-27 P0 source-audit hardening (committed)

**Status:** code + migration + tests complete on `dev`; founder authorized
commit/merge 2026-11-27 (AGENT_GUIDE §7).

A source audit found three P0 defects not recorded in any ledger (R-039–R-041,
ADR-026): `upsert_claim()` queried a `claims_onchain` table no migration
created, so the first Claim PDA would abort the 15 s poll and starve later
accounts; spatial/geometry upserts never wrote chain provenance columns; and
all upserts bound `chain_signature = ""` plus the poll-time slot, making
`chain_verified` mean "seen during a scan". Fixed by:

1. Forward migration `0057_claims_onchain_mirror.sql` (applied to Supabase;
   20 columns verified, includes `chain_verified`/`chain_slot`/
   `chain_signature`/`mirror_hash` + trigger). No applied migration rewritten
   (ADR-011).
2. Per-account error isolation in `poll_once()`: only poll-level RPC failures
   abort; one bad account logs and continues.
3. `AccountProvenance` from `getSignaturesForAddress` (newest non-failing
   transaction), bound on INSERT and UPDATE for parcels, attestations, claims,
   spatial assets and geometry versions. RPC/empty-signature failures yield
   `FALSE`/`NULL`/`NULL` — never an empty signature or scan slot.
4. Two latent bugs exposed by the new DB-backed test: geometry INSERT had 24
   target columns / 23 values (fixed `$23`), and `asset_pubkey` was hex despite
   an FK to base58 `spatial_assets_onchain.pubkey` (now
   `Pubkey::new_from_array(...).to_string()`).

**Evidence / gates:** `cargo check -p terra-api` clean;
`cargo test -p terra-api` **173/173** with `DATABASE_URL` against Supabase;
7 new indexer tests (claim discriminator, provenance filtering,
unknown/truncated buffers, claims columns, claim provenance, spatial/geometry
provenance). Original exit checks (CLI parcel + hand-edited row via
`/mirror/audit`) still require a live devnet run; hosted RPC rate-limit impact
of per-account signature lookups remains under monitoring (R-041).

## Addendum 2 — 2026-11-27 findings 1d/1e (committed)

**Status:** code + two forward migrations + tests complete on `dev`; founder
authorized commit/merge 2026-11-27 (AGENT_GUIDE §7).

A second source audit found five defects that survived Addendum 1 (ADR-027,
R-042–R-046):

1. Hand-written Borsh parsers read string lengths as `u8`/`u16` instead of
   `u32` LE. Fixed by generating decoders from `idl/terra_registry.json`
   (`scripts/generate-indexer-decoders.py` → `api/src/indexer_decoders.rs`),
   pinned by a new CI step (`--check`). All 8 mirrored accounts round-trip in
   `idl_decoders_round_trip_all_mirrored_accounts`.
2. Only 5 of 8 accounts with mirror tables were decoded; `Rights`,
   `ThreeDModel` and `SpatialSnapshot` were never mirrored. Added, with a
   two-phase poll so FK-dependent rows follow their parents in RPC order.
3. `Rights.parcel` is the Parcel PDA, but the old lookup used `parcel.id` and
   fell back to writing the PDA as a fake wallet holder. The holder now comes
   from `Rights.holder` and is written in base58; proven by
   `rights_mirror_uses_parcel_pda_and_real_wallet_holder`.
4. On-chain `geometry_hash` was never compared to the polygon. The client
   anchors `sha256(JSON.stringify(ring))`; PostGIS cannot reproduce those
   bytes, so forward migration `0059_parcel_source_geometry.sql` keeps the
   submitted GeoJSON verbatim and `submitted_ring_hash()` recomputes the
   anchor. `chain_verified=TRUE` now requires both that match and a real
   transaction. Chain-only parcels are never fabricated (`geometry` is
   `NOT NULL` and guarded). Test:
   `parcel_verification_requires_matching_geometry_digest`.
5. The 15 s full-account poll had no backoff and hosted attestation defaults
   contradicted ADR-026 (`TRUE`/`0`/`''`). Forward migration
   `0058_indexer_mirror_honesty.sql` fixes the defaults, downgrades existing
   rows, adds `rights.onchain_id` + unique index; `/mirror/audit` now covers
   `claims_onchain` and its hash SQL matches migrations 0057/0058/0059.
   Poll cadence/timeout/backoff/batch size are env-driven (default 60 s) and
   the scan is header-only + batched.

**Gates:** IDL generator `--check` PASS; `cargo check -p terra-api` clean;
`cargo test -p terra-api` **182/182** with `DATABASE_URL` against Supabase;
`git diff --check` clean. Frontend unchanged (no frontend gates re-run).
**Residual / founder choices:** Q-033(b) and Q-035(a) were authorized and
implemented in `sprints/063-sprint-q033-q035.md` (generic all-IDL mirror +
content-addressed chain-only geometry); typed parity for the remaining account
types is stage 2. Solana RPC 3.1.12 has no cursor pagination (Q-034); hosted
provisioning and RPC rate-limit measurement remain founder-gated
(Q-032/R-045). Live CLI-parcel + hand-edited-row exit checks still pending.
