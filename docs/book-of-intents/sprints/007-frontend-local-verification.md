# Sprint 007 — Local frontend verification (no funding required)

**Loop:** Understand → Run → Observe → Record
**Phase derived from evidence:** Test (validated) → Loop (this record)

## Context

Founder directive 2026-11-02: devnet funding is the **last** step, not the
next. Stage 1 exit made Stage 2 the "next" roadmap item only by dependency
order; none of the frontend work needs SOL. This sprint verifies the product
surface locally so work can continue without chain access.

## Run

- API already running with the dev authority key on `:8080`.
- `pnpm dev --host 127.0.0.1` in `terra-web` → Vite 8.2.1 on `:5173`.
- Headless Chrome (`--headless=new`, SwiftShader WebGL) captured four routes
  at 1600×1000 after client hydration.

## Observed (2026-11-02)

| Route | Result |
| --- | --- |
| `/welcome` | Hero, wallet-overview panel, spatial-asset preview, live network pulse ("4 Recent API records", "API connected"). |
| `/stories` | Three offline demo stories (road journey, courtyard journey, geometry history) + one saved story from the API; create-journey form. |
| `/demo` | Verification scenario replayed to completion: `VERIFIED`, 12 events / 9938 ms, validator decision replay for 3 reviewers, live activity feed, protocol visualization. Banner: "Demo state · not chain state". |
| `/story/demo-road-journey` | Cesium globe with the mapped RN1 Soa Link route, timeline (0:00.0/0:31.0), playback controls; offline demo, no API/wallet. |

Notes: `/demo` needs a longer headless budget (~25 s virtual time) before the
first screenshot; everything else captures at ~15 s. Console warnings are the
known Cesium terrain-outline and Vite `buffer` externalization warnings — no
errors, no failed requests.

## Loop

- Frontend is demoable locally with no funding. Screenshots kept in
  `/tmp/opencode/*.png` (ephemeral, not committed).
- Book updated: CURRENT.md next-up reordered (product work first, funding
  last); ROADMAP Stage 2 sequencing note; CHANGELOG entry.
- Stage 2 remains available whenever the founder funds the deployer wallet;
  Q-001 itself is unchanged.
