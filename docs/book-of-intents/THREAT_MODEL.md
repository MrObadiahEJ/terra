# Threat Model — Terra MVP

**Scope:** One flow end-to-end: licensed surveyor attests ONE parcel → issues
W3C Verifiable Credential (SD-JWT) → owner+tenant dual-sign residency proof →
tenant presents to mock verifier. Zero budget, free tiers only, SOL fees only.

## Adversaries

| Adversary | Capability | Motivation | Mitigation |
|---|---|---|---|
| **Compromised surveyor** | Signs false parcel attestation | Fraudulent residency claim | VC signed by surveyor's institutional key; revocable; surveyor reputation at stake |
| **Colluding owner+tenant** | Dual-signs false residency proof | Unqualified tenant claims residency | Two independent signatures required; VC binds to parcel+surveyor; mock verifier cross-checks VC signature |
| **Malicious mock verifier** | Accepts forged VC / residency proof | Bypass residency gate | Verifier runs local signature + credential-status check; no trusted third party in the flow |
| **Replay attacker** | Re-submits old VC or residency proof | Gain residency under stale conditions | VC includes `issuanceDate` + `expirationDate`; residency proof includes nonce + timestamp |
| **Storage provider (Irys)** | Loses or censors the 3D blob | Blob unavailable for verification | Content hash pinned on-chain; re-upload from any replica; N-of-M not required for MVP (single blob, hash-verified) |
| **RPC provider** | Returns stale/wrong chain state | Client acts on old data | Client confirms transaction finality before proceeding |

## Trust Root

The trust root is **Solana devnet** (SOL fee payments only, no token). On-chain state is the source of truth for:

1. **Parcel registry** — parcel exists, owner identified.
2. **Surveyor authority** — surveyor's institutional key is registered and not revoked.
3. **VC issuance record** — VC credential-subject hash and surveyor signature are anchored on-chain (content hash + slot).
4. **Residency proof** — dual-signed owner+tenant residency credential anchored on-chain with nonce + timestamp.

No off-chain service is trusted for correctness; every verifiable claim is anchored on-chain or verified locally via signature.

## Failure Modes

| Failure | Symptom | Detection | Recovery |
|---|---|---|---|
| **VC signature invalid** | Verifier rejects credential | Local Ed25519 verification fails | Surveyor re-issues VC with valid key |
| **Residency proof missing one signature** | Owner or tenant did not sign | Dual-signature check fails | Re-run dual-sign ceremony |
| **Content hash mismatch** | On-chain hash ≠ uploaded blob hash | Hash re-computed on download | Re-upload blob to Irys; update on-chain anchor |
| **RPC outage** | Transaction not confirmed | Timeout / no confirmation | Retry with alternate free RPC (Helius/QuickNode/Alchemy devnet) |
| **Surveyor key revoked** | VC issued by revoked key | On-chain revocation check fails | New surveyor attests; old VC invalidated |

## Slashing Conditions

Slashing applies to **validator/stake accounts** (not surveyors or tenants) for protocol-layer misconduct:

| Offense | Evidence | Slash | Appeal |
|---|---|---|---|
| **Equivocation** | Two conflicting attestations for same parcel+slot | 100% of stake | RFC-005 appeal path |
| **Liveness failure** | Missed quorum window on routed task | Proportional (25% stake) | RFC-005 appeal path |
| **Collusion** | Same validator signs both owner+tenant residency for same parcel | 100% of stake | RFC-005 appeal path |

Slashing is **not** applied to surveyors issuing false VCs (that is off-chain institutional discipline: key revocation, reputation). Slashing is **not** applied to tenants/owners for false residency (that is civil/legal, not protocol).

No speculative token, no fee-yield, no profit-sharing — SOL fee payments only, per constraint.
