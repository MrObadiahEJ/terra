# Sprint 006 — Verification refresh (ROADMAP Stage 1 item 4)

**Loop:** Research → Understand → Compare → Identify Gaps → Re-run → Record
**Phase derived from evidence:** Test (validated) → Loop (this record)

## Research

- Baselines in IMPLEMENTATION_STATUS were dated 2026-10-13 except API/vitest
  (2026-10-31). Stage 1 exit requires re-executed numbers with dates.
- Known trap: the BPF integration suites load prebuilt `.so` artifacts and
  fail loudly when those are older than `programs/*/src` (stale-build guard).

## Plan

- Rebuild program artifacts (`anchor build --skip-lint`), then run each suite
  sequentially with `CARGO_INCREMENTAL=0` (resource-constrained box).
- Record executed counts and note suite-size changes; confirm CI Rust-job
  depth (R-008) from `.github/workflows/ci.yml`.

## Build / run

1. `anchor build --skip-lint` → fresh `target/deploy/*.so` (2026-11-02).
2. `cargo test -p terra-geo` → **12 passed, 1 ignored** (baseline was 6; the
   suite has grown — recorded, not a regression).
3. `cargo test -p terra-registry --lib` → **153 passed**.
4. `cargo test -p terra-identity --test integration` → **23 passed** (after
   the rebuild; before it the freshness guard failed 18 tests with a clear
   "stale build" panic — environmental, not logic).
5. `cargo test -p terra-registry --test integration -- --test-threads=1` →
   **307 passed** (295.9 s).
6. `cargo test -p terra-api` → **121 passed**.
7. `pnpm exec vitest run` → **44/44**.

## Test (evidence)

All suites green on current sources; table re-dated in IMPLEMENTATION_STATUS.
CI audit: the "On-chain Program Tests" job runs registry/identity **lib** +
rfc012_structure + geo, and the API job runs `cargo test -p terra-api`; the
307-test BPF **integration** suite is not wired — it needs built `.so`
artifacts. R-008 confirmed; E1 downgraded to `PARTIALLY IMPLEMENTED`.

## Loop

- Remaining Stage 1 exit item: wire the BPF integration suite into CI
  (R-008). Everything else in the refresh is done and dated.
- Devnet deploy remains founder-blocked (Q-001); no on-chain status changes.
