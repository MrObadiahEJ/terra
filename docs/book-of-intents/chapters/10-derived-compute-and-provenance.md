# Chapter 10 — Derived Compute, Request Authenticity, AI Provenance

**Status:** `IMPLEMENTED AND VERIFIED` — RFC-014 Phase 04, R-016/R-017,
REQ-024.
**See:** [ROADMAP](../ROADMAP.md) Stage 1,
[REQUIREMENTS_TRACEABILITY.md](../REQUIREMENTS_TRACEABILITY.md) REQ-019/024,
[IMPLEMENTATION_STATUS.md](../IMPLEMENTATION_STATUS.md) B8–B10,
sprints [003](../sprints/003-derived-compute.md),
[004](../sprints/004-request-auth.md),
[008](../sprints/008-ai-provenance.md).

---

## 10.1 The problem

Three different questions look similar in an API response but mean different
things:

1. *Did we already compute this?* — a cache/dedup question.
2. *Who asked for this mutation?* — an authenticity question.
3. *Was this output produced by a model?* — a provenance question.

Mixing them (treating a cache hit as a verified result, a signature as an
identity, or a model label as evidence) would violate the protocol's central
rule:

```
OBSERVATION ≠ EVIDENCE ≠ VERIFICATION ≠ CONSENSUS ≠ LEGAL RECOGNITION
```

This chapter records how Terra keeps the three separate.

## 10.2 Derived compute is a cache, not a fact

Incremental work is de-duplicated by a **processing key**:

```
processing_key = SHA-256(kind ‖ input_hash)
```

- `derived_jobs.processing_key` is `UNIQUE`; creation is a single atomic
  `INSERT … ON CONFLICT … DO UPDATE … RETURNING`, so two concurrent requests
  for the same input cannot both claim to have created the job.
- A finished (`done`) row is reused verbatim. Pending/running/stale/failed
  rows are reset to pending for a retry.
- Invalidation is a recursive CTE over `derived_job_dependencies`: consumers
  are marked stale transitively.
- Completion is idempotent and guarded: only a pending/running row may move
  to `done`; a stale or failed row returns `409`.

**What this proves:** the same key yields the same stored output.
**What it does not prove:** that the output is correct, current, or verified.
A cache hit is not evidence and never becomes a consensus input.

## 10.3 Request authenticity is not identity

Privileged mutations (`derived` complete/fail/invalidate, reconstruction
create/lease/progress/complete/anchor) require a signed request:

```
{METHOD}\n{PATH}\n{BODY_SHA256_HEX}\n{TIMESTAMP_EPOCH_SECS}
```

Headers: `Authorization: Signature <base64 ed25519>`, `x-timestamp`,
`x-body-hash`, and optional `x-wallet`. Signatures are verified against
`API_AUTHORITY_PUBKEY`; when that variable is unset the endpoints **fail
closed** (401). The timestamp window is ±60 s.

Wallet binding is a separate check (`verify_wallet`) used where the caller's
identity is recorded — for example the reconstruction lease.

**What this proves:** the request was signed by the holder of the authority
key, over this exact method/path/body/time.
**What it does not prove:** that the signer is a verified human, a licensed
institution, or a legitimate owner of any parcel. Keys are authority, not
identity (Chapter 8 is where institutional scoping is proposed).

The shipped client `terra-core/api/examples/sign_request.rs` emits the four
headers for smoke scripts and integrations; the contract is in
[AGENT_GUIDE §6](../AGENT_GUIDE.md).

## 10.4 AI provenance is description, not verification

A geometry draft records how it was produced:

| Field | Meaning |
| --- | --- |
| `provenance_kind` | `deterministic` (default), `ai-inference`, `ai-assisted` |
| `model_provider` | vendor, optional |
| `model_name` | model identifier — required for AI kinds |
| `model_version` | checkpoint/version — required for AI kinds |
| `model_confidence` | model-reported probability in `[0,1]` |

`job_complete` rejects ambiguous payloads **before** changing job state:

- model metadata on a `deterministic` draft → 400;
- AI kind without model name **and** version → 400;
- confidence outside `[0,1]`, NaN, or infinity → 400;
- provenance strings outside 1–120 characters → 400.

Database constraints mirror the invariants, so a writer bypassing the API
still cannot create an AI draft without a model name or a confidence outside
the probability range.

`model_confidence` is returned verbatim. It is the model's own claim, not a
protocol score, not a quorum result, and not a substitute for review.

**What this proves:** which model, version, and provider (if declared)
produced the draft, if the caller declared it.
**What it does not prove:** that the draft is accurate, that the model is
trusted, or that the geometry is a parcel. A draft becomes a fact only after
the anchor is confirmed on-chain and the normal review path accepts it.

## 10.5 Failure modes this design prevents

| Failure | Guard |
| --- | --- |
| Two workers compute the same thing twice | UNIQUE processing key + atomic upsert |
| A retry overwrites a finished result | status guard; idempotent `done` |
| Invalidation misses dependent jobs | recursive-CTE closure over dependencies |
| An unsigned caller mutates jobs | Ed25519 verification, fail-closed |
| A gateway strips auth | 401; no implicit trust from network position |
| A model output is silently treated as deterministic | provenance kind + fail-closed validation |
| A caller inflates “truth” with a confidence number | documented as model-reported only; no protocol consumer treats it as a score |

## 10.6 Acceptance evidence

- **Derived compute:** API tests (7 derived), live smoke **16/16**; two
  identical reconstruction computations → 2 drafts, **1** ledger row, equal
  hashes (2026-10-31, sprint 003).
- **Request authenticity:** 8 handlers fail-closed; unsigned → **401**,
  wrong key → **401**, authority key → 200 (2026-11-02, sprint 004).
- **AI provenance:** API **128/128** (7 provenance tests), live smoke
  **13/13**, psql constraints reject bypasses (2026-11-02, sprint 008).
- CI runs the API and frontend jobs plus the 307-test BPF suite
  (run `41474466105`, green).

## 10.7 Connection to distributed reconstruction

Derived compute (Section 10.2) is a **dedup cache for a single deterministic
function**. Distributed reconstruction (Chapter 11) is a different beast:
multiple independent engines may produce overlapping components that must be
registered, aligned, and assembled. The two systems connect as follows:

- A reconstruction **task** is a higher-level unit than a derived job. A task
  may spawn several derived jobs (one per component: roof, facade, terrain)
  plus one assembly job.
- Component packages (Chapter 11 §11.3) carry their own `geometry_digest`
  plus provenance, accuracy, and dependency fields — a superset of the
  derived-job `processing_key` concept.
- Resource metering (Part VIII §37) applies to reconstruction tasks: CPU/GPU
  time, memory, storage, bandwidth, and human labor are recorded per worker,
  not just per cache hit.
- Evidence lineage: every component package references its source evidence,
  and every derived job that consumes a component records that dependency so
  invalidation can propagate transitively (as derived jobs already do).

Derived compute remains a cache — never a consensus input. Reconstruction
outputs become facts only after validation and assembly acceptance.

## 10.8 Open edges

- Provenance is on `geometry_drafts`; if AI-generated **model artifacts**
  become common, `three_d_models` should carry the same fields.
- Confidence calibration is not validated; it is recorded as reported.
- Institutional scoping of the authority key is `PROPOSED` (Chapter 8,
  Q-003) — today the key is a single operator authority, not a federation.
