# Sprint 009 — Book volume 10: trust boundaries of the new machinery

**Loop:** Understand → Distil → Write → Cross-link → Record
**Phase derived from evidence:** Loop (documentation of already-verified work)

## Why

E4 recorded "volumes per ROADMAP still unwritten". Chapters 09 (replay) and
the Stage 1 systems (derived compute, request signing, AI provenance) had
registers and sprints but no narrative volume, so a new agent had to
reconstruct *what the guarantees are* from code and changelog entries.

## What was written

`chapters/10-derived-compute-and-provenance.md`:

- **10.2** processing-key dedup as a cache: atomic upsert, status guards,
  recursive-CTE invalidation; explicitly "a cache hit is not evidence".
- **10.3** signed requests: payload, headers, fail-closed authority key,
  separate wallet binding; "a signature is authority, not identity".
- **10.4** AI provenance: `provenance_kind` + model fields, fail-closed
  validation, DB constraints; "provenance is description, not verification".
- **10.5** failure-mode table mapping each guard to the failure it prevents.
- **10.6** acceptance evidence (API 128/128, live smokes 16/16 and 13/13, CI
  run `41474466105`).
- **10.7** open edges: `three_d_models` provenance, uncalibrated confidence,
  single-operator authority key until Q-003.

## Cross-links

- `BOOK_OF_INTENTS.md` and `SUMMARY.md` list Chapter 10.
- E4 evidence now cites chapters 01–10.
- Pointer to AGENT_GUIDE §6 for the signing contract.

## Evidence

Documentation-only change; no test counts move. All claims in the chapter
cite the sprint that produced them (003, 004, 008) or the CI run.

## Loop

- E4 remains `PARTIALLY IMPLEMENTED` by design — later volumes depend on
  research-gated intents (Q-003 institutional identity, Q-005 fractional
  participation).
- Next non-chain candidates: frontend/UX polish and R-012 (bundle discipline);
  no funding required.
