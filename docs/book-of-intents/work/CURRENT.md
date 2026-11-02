# Current Work Ledger

Authoritative record of active execution. Phase derives from evidence here.
Last reconciled: 2026-11-02 (working tree on top of `1b50524`).

## Active intents

- **INT-001/002/003** — maintained (`IMPLEMENTED AND VERIFIED`; regressions
  caught by BPF/vitest gates).
- **INT-004** — `PARTIALLY IMPLEMENTED` (see intent for the exact gap list).
- **INT-005** — `PROPOSED` (research-gated; Q-003).

## Execution state

| Work item | State | Evidence |
| --- | --- | --- |
| RFC-014 Phases 01–03 (story engine, road-graph navigation + version provenance, export refinements) | **done & committed** | commits `c9faf54`, `54ad2bd`, `0f0cc2e` + fixes `b464cdb`, `a2b2afe`, `11be94c`, `ae20af8`; vitest 44/44 (2026-10-31) |
| Canonical geometry digest `terra-geom-v1` | **done & committed** | commit `775e20a`; `docs/canonical-geometry-digest.md` |
| Book of Intents constitution (this directory) | **v0.1.0 delivered & committed** | commit `6be7c91`; sprint 002; CHANGELOG |
| RFC-014 Phase 04 — incremental derived compute (REQ-019) | **done (2026-10-31), commit pending** | migration 0033 + `routes/derived.rs`; API **117/117** (7 derived tests); live smoke **16/16**; reconstruction: 2 drafts → 1 ledger row, equal hashes; sprints/003 |
| R-016 — verify signatures on privileged endpoints | **done (2026-11-02)** | 8 handlers now `auth.verify(…)` fail-closed (derived + reconstruction); live: unsigned 401, wrong key 401, authority key 200; sprints/004 |
| R-017 — shipped request-signing client | **done (2026-11-02)** | `api/examples/sign_request.rs` (deps already present); contract in AGENT_GUIDE §6; drove the live smoke; sprints/004 |
| RFC-014 Phase 05 — external replay/access (REQ-018) | **done (2026-11-02), commit pending** | `routes/stories.rs` `/replay` + `/replay.geojson` + `/schema/replay`; `terra.story-replay/v1`; API **121/121**; live smoke (hash tracks edits, 404s); chapters/09, sprints/005 |
| Verification refresh (all suites re-executed) | **done (2026-11-02)** | API 121/121; registry lib 153/153; BPF 307/307 (295.9 s); identity 23/23; geo 12 (+1 ignored); vitest 44/44; baselines re-dated; sprints/006 |
| CI Rust-job depth (R-008) | **done (2026-11-02)** | `.github/workflows/ci.yml`: Solana `v3.1.14` + SBF build + 307-test BPF suite; run `41474466105` green (On-chain Program Tests 5m18s) |
| Frontend local verification (no funding) | **done (2026-11-02)** | dev server + local API; `/welcome`, `/stories`, `/demo` (VERIFIED, 12 events / 9938 ms), `/story/demo-road-journey` all render; sprints/007 |

## Known blockers

- **Devnet deploy (B6 / Q-001).** Deployer wallet `CCng7wFcSKCh1RadwcFEN36E4tRrPy1D64fwsQDrsy4m`
  holds 0 SOL; faucet rate-limited (~46 attempts). `./deploy.sh devnet` ready.
  Founder action. All on-chain statuses remain local-verified until this runs
  (RISK R-001). **Founder directive 2026-11-02: funding is the last step, not
  the next one — it blocks only Stage 2, never frontend/API/local work.**
- **Not blocked:** dev server (`pnpm dev` + terra-api), offline demo stories
  (no wallet/API), API suites, and frontend gates all run with zero SOL.

## Next up (order)

1. Local product work on `dev` — frontend/UX + API; no funding needed.
   Verified 2026-11-02: `/welcome`, `/stories`, `/demo`, `/story/demo-road-journey`
   all render against the local API (sprints/007).
2. Remaining non-chain requirements (e.g. R-014 AI provenance fields).
3. **Last:** fund the devnet deployer wallet (`CCng7wFcSKCh1RadwcFEN36E4tRrPy1D64fwsQDrsy4m`)
   — founder action, Q-001 — then `./deploy.sh devnet` + Stage 2 on-chain smoke
   (A1/B6 cluster-verified).
