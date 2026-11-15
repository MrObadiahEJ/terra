# Sprint 046: Protocol architecture directives — economics + secure evidence intake

**Date:** 2026-11-13 · **Status:** done & committed (`6dd8018`); docs only

## Trigger

The treasury's economic role and the evidence-intake threat model were
underspecified for Stage 3/4, and several RFCs had gaps around retention,
compromise response and sensitive layers.

## Implementation

- New `docs/rfc-014-protocol-economics-contributor-rewards.md`: treasury
  outflow-only economics, Q-005 securities hard gate, staging bonds, governed
  epochs and Merkle claims; no token and no FeePolicy mutation.
- New `docs/rfc-017-secure-evidence-and-accountable-validation.md`: quarantine
  + sandbox intake, rendition and forensic marking, sealed attribution,
  Assurance Profiles AP-0…AP-3, retention/erasure.
- Amendments: RFC-016 §11 (retention/erasure), RFC-010 §15 (compromise
  fast-path), RFC-007 §9.5 (compromise freeze trigger), RFC-012 §6 rule 16
  (Assurance Profile routing), Terra File directive §7 (sensitive layers never
  default-on; identity documents excluded from exports unless authorized).
- Registers extended: REQ-041…052, ADR-022/023, R-027…R-032, Q-019/020/021.

## Verification (against code)

`MAX_BODY_BYTES` default 64 MiB; `REQUEST_TIMEOUT_SECS` 60 s;
`StorageBackend::Local` only — no quarantine/isolation, no retention/TTL/purge
path (R-027 documents the gap). Doc-only; no code gate.

## What is still missing

- Quarantine/sandbox intake pipeline, rendition pipeline, retention/purge.
- Multi-provider storage beyond the local backend.
- Founder review of RFC-014 economics and RFC-017 Assurance Profiles.

## Book updates

`CHANGELOG.md`, `SUMMARY.md`, `work/CURRENT.md`, `IMPLEMENTATION_STATUS.md`,
`REQUIREMENTS_TRACEABILITY.md`, `ARCHITECTURAL_DECISIONS.md`,
`RISK_REGISTER.md`, `OPEN_QUESTIONS.md`, and the RFC files.
