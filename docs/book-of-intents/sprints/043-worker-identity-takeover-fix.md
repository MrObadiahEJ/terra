# Sprint 043: Worker identity takeover fix

**Date:** 2026-11-13 · **Status:** done & committed (`5df3770`)

## Trigger

`register_worker` used `ON CONFLICT (worker_id) DO UPDATE SET wallet_address =
EXCLUDED.wallet_address`. Anyone who knew a worker_id could re-register it under
their own wallet and receive that worker's assignments and rewards (REQ-038).

## Implementation

- `terra-core/api/src/routes/workers.rs`: wallet binding is set once at worker
  creation; re-registration no longer overwrites `wallet_address`.
- A conflicting registration with a different wallet is rejected instead of
  silently taking over the identity.

## Verification

`cargo check -p terra-api` clean; `cargo clippy -- -D warnings` clean;
`cargo fmt --check` clean. CI later exposed unrelated migration/registry
failures (repaired in Sprint 054).

## What is still missing

- No dedicated regression test existed in this commit; the behavior is enforced
  by the upsert clause. Add a DB-backed re-registration test when touching the
  worker routes.

## Book updates

`work/CURRENT.md`, `IMPLEMENTATION_STATUS.md`, `REQUIREMENTS_TRACEABILITY.md`
(REQ-038), `CHANGELOG.md`.
