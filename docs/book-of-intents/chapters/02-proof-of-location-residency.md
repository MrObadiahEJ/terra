# Chapter 2 — Proof of Location & Residency Tokens

**Status:** `SPECIFIED ONLY` — designed and acceptance-criteria'd; no code.
The building blocks below are real and `IMPLEMENTED AND VERIFIED`; the residency
semantics are not built.
**Founder review flag:** ⚠ pending — Q-004 (which jurisdictions recognize the
novel dual-signed attestation; tenant protections) is unresolved; no code is
written until it is.
See [OPEN_QUESTIONS.md](../OPEN_QUESTIONS.md) Q-004 and
[REQUIREMENTS_TRACEABILITY.md](../REQUIREMENTS_TRACEABILITY.md) REQ-021.

---

## 2.1 The problem

A person often needs to prove a *relationship to a place* — "I reside in this
building/zone/city, for this period" — to a bank, insurer, or service
provider. Today this requires exposing a full lease or utility document,
leaking personal data and remaining trivially forgeable.

The concrete failure this removes: **credit reconstruction for the
under-documented.** A tenant with thin formal credit history can present a
cryptographically verifiable residency credential without disclosing the lease
itself.

## 2.2 Desired outcome

An authorized party (property owner, or an institution) attests that a person
has a relationship with a property. The resident may counter-sign. The result
is a **selective-disclosure credential**: a verifier learns only the required
predicate ("resides in zone X, valid as of date D") and nothing else.

## 2.3 Why it is `PROPOSED`

The credential machinery and the disclosure machinery both exist; the
*composition* — a residency-credential instruction that binds a rights holder's
attestation to a resident's key with expiry, revocation, and tenant
protection — does not. Building it before Q-004 (which jurisdictions recognize
such attestations; what tenant protections apply) would ship an unsafe
capability, so it is research-gated.

**Research recorded 2026-11-02** ([RESEARCH_REFERENCES](../RESEARCH_REFERENCES.md),
Q-004): six jurisdictions surveyed — Portugal (Junta de Freguesia
`Atestado de Residência`, Lei 7/2001), Queensland (AU), DC (US), New York
(CUNY), Monaco (Sovereign Ordinance 8.566/8.372), Luxembourg (Act of
19 Jun 2013). **Every** jurisdiction requires *two* independent proofs of
address from distinct sources, plus a recency window (3–12 months) and a named
attestor with a relationship to the subject. **None** issues a
selective-disclosure credential — the resident always hands over the underlying
document. A dual-signed residency attestation is therefore a *novel* instrument
whose recognition cannot be assumed; it needs jurisdiction-specific legal review
before any code is written.

## 2.4 Building blocks (all `IMPLEMENTED AND VERIFIED`)

| Block | Evidence |
| --- | --- |
| Time-bound credentials (request → sign → finalize → verify) | `terra_registry/src/lib.rs` `request_credential`…`verify_credential` (L1766–1791); RFC-009 |
| Rights model proving a holder's relationship to a parcel | `grant_right`/`revoke_right` (L860–951); migration 0025 (ownership as Rights PDA) |
| Time-bound validity, expiry, grace | `time_bound.rs` (`update_right_status`) |
| Selective disclosure / ZK ownership proofs | `zk/groth16.rs`; `verify_ownership_proof` (L1733); RFC-011 (status 2 — verification incomplete) |
| Key rotation & recovery | identity `bind_identity.rs`; guardianship/succession |

## 2.5 Instruction flow (specification, not implemented)

1. **Establish attestor authority** — the attesting party must hold a verified
   right over the property (ownership/tenancy-management right), not merely
   claim one. Guard: `require!(attestor_holds_relevant_right, …)`.
2. **Request** — resident key + property reference + claim window.
3. **Dual sign** — attestor signature + resident acceptance.
4. **Finalize** → credential PDA with `expires_at`, `revoked` flag, and a
   disclosure predicate.
5. **Verify** — off-chain verifiers check the credential against the program;
   ZK path reveals only the predicate.

## 2.6 Trust assumptions & failure modes

- **Fraudulent landlord:** an attestor can only bind rights they verifiably
  hold; a competing claim routes to the dispute system, not to a credential.
- **Unauthorized attestation:** guard rejects attestors without a live,
  non-expired right.
- **Revocation & expiry:** a credential must not outlive the underlying right;
  sweeping (`sweep_expired_rights`) invalidates dependent credentials.
- **Tenant protection:** residency disclosure must be opt-in and predicate-only
  (Q-004 research required).

## 2.7 Acceptance criteria (before status leaves `PROPOSED`)

- [ ] Research note in RESEARCH_REFERENCES covering ≥1 jurisdiction's
      recognition of residency attestations (Q-004).
- [ ] Instruction implemented with attestor-authority + resident-acceptance guards.
- [ ] Revocation/expiry propagation test (credential dies with the right).
- [ ] ZK selective-disclosure path proven (no full lease on the wire).
- [ ] BPF lifecycle test; REQ-021 status updated.

**No code is written for this chapter until Q-004 is resolved.**
