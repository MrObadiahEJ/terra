# Sprint 034 — Restore the CI Formatting and Clippy Gates

**Date:** 2026-11-13 · **Status:** done, committed (`9ab419d`, `f9f0a0d`)
**Gate:** `cargo fmt --all -- --check`; `cargo clippy -- -D warnings`;
`cargo test -p terra-api` — **139 passed**; `cargo test -p terra-geo --lib` —
**48 passed, 2 ignored**

## Trigger

CI run `43454803577` (Sprint 032 docs) showed API tests, API build, and frontend
green, but the `On-chain Program Tests` job failed at `cargo fmt --check`.
After formatting was fixed in `9ab419d`, the next run (`43458223319`) reached
Clippy and failed there on pre-existing workspace lints. The clippy gate had
never been green for the API crate; the formatting debt was also older than the
spatial work.

## Changes

### `9ab419d` — formatting only

`cargo fmt --all` across five API files (`main.rs`, `routes/components.rs`,
`routes/mod.rs`, `routes/providers.rs`, `routes/workers.rs`). No behavior change.

### `f9f0a0d` — lint fixes without behavior changes

- `api/src/main.rs`: replace deprecated `TimeoutLayer::new` with
  `TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, …)`. The previous
  default was already 408; this makes it explicit and matches the timeout test.
- `api/src/routes/workers.rs`: remove an unused `put` import.
- `api/src/routes/runtime.rs`: remove unused `MANIFEST_OBJECTS_FOR_SCENE` and
  `AssemblyObjectRow`; remove an `i32 -> i32` cast.
- `api/src/routes/assemblies.rs`, `components.rs`, `manifests.rs`: drop needless
  borrows of owned `serde_json::Value` binds and of a copyable `Option<Uuid>`.
- `geo-engine/src/component_package.rs`: collapse a nested `if`, use
  `is_none_or`, `.first()`, a `CoordExtremes` type alias, and `matches!`.

The timeout test (`.layer(TimeoutLayer::new(...))` in `main.rs` tests) was also
updated to `with_status_code`, removing the last test-build warning.

## Verification

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | clean |
| `cargo clippy -- -D warnings` (workspace) | clean, 16.8 s after build |
| `cargo test -p terra-api` | **139 passed** (133 baseline + 6 Sprint 033 tests) |
| `cargo test -p terra-geo --lib` | **48 passed, 2 ignored** |
| CI `43460062612` | **all jobs green** — formatting, clippy, registry/identity/RFC-012/geo tests, SBF build, BPF integration, API tests, API build, frontend |

The local full API suite was run for the first time since Sprint 033; the
machine needed only the incremental test build. No BPF/SBF build or
full-workspace test suite was run locally.

## What this does not change

No endpoint, query, schema, or response behavior changed. The on-chain job still
runs its own unit/SBF steps after clippy; those results are tracked separately
from this cleanup.

## Book updates

`work/CURRENT.md`, `CHANGELOG.md`, `SUMMARY.md`, and `BOOK_OF_INTENTS.md` record
the maintenance sprint and the restored gates.
