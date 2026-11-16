# Terra — Whitepaper-Lite

**MVP — Residency Attestation & Spatial Identity**
**Zero budget · Free tiers only · SOL fees only · No token**

---

## 1. Problem

Land tenure and residency claims in emerging markets rest on paper records, weak
spatial indexing, and centralized registries that are expensive to maintain and
easy to tamper with. A resident who cannot prove where they live cannot access
banking, credit, property rights, or government services. The verification
pipeline — surveyor attests parcel → authority verifies → resident holds proof —
is fragmented across siloed databases with no cryptographic link between the
on-chain parcel registry and the off-chain attestation.

Terra closes this gap with a minimal viable product that anchors one flow
end-to-end:

1. A **licensed surveyor** attests a parcel and issues a W3C Verifiable
   Credential (VC) with SD-JWT selective disclosure.
2. **Owner + tenant** dual-sign a residency proof.
3. The **tenant** presents the VC + residency proof to a verifier.

Every step is anchored on Solana devnet (SOL fees only), verified client-side,
and never touches a token.

---

## 2. Architecture

Terra is a Solana Anchor program (`terra_registry`) plus a TypeScript/React
frontend (`terra-web`) and an Axum API (`terra-core/api`). The MVP touches four
on-chain accounts:

| Account | Purpose | On-Chain Data |
|---|---|---|
| `Parcel` | Land parcel registry | ID, name, status, geometry hash, on-chain ID |
| `VerificationAttestation` | W3C VC (SD-JWT) anchor | VC type, subject, dates, SD-JWT digest |
| `ThreeDModel` | 3D reconstruction claim | Model hash (content), H3 index, format |
| `TaskAssignment` | Validator routing assignment | Task, validator, winner, draw_bps |

**Trust root:** Solana devnet. On-chain state is the source of truth for parcel
registry, surveyor authority, VC issuance records, and residency proofs. No
off-chain service is trusted for correctness.

**Randomness:** Validator routing uses Switchboard VRF (primary) or Pyth
Entropy (fallback) — never blockhash-only.

**Storage:** 3D reconstruction blobs live on Irys/Arweave; on-chain accounts
store only content hash (SHA-256) + H3 cell index.

---

## 3. Threat Model

**Adversaries:** compromised surveyor (false attestation), colluding owner +
tenant (false residency), malicious mock verifier (accept forged VC), replay
attacker (re-submit stale VC), storage provider (loses/censors blob), RPC
provider (stale state).

**Trust root:** Solana devnet — parcel registry, surveyor authority, VC
issuance, residency proof.

**Failure modes:** invalid VC signature (re-issue), missing dual-signature
(re-run ceremony), content hash mismatch (re-upload), RPC outage (retry with
alternate free RPC), revoked surveyor key (new surveyor attests).

**Slashing:** applies to validator/stake accounts for equivocation (100%),
liveness failure (25%), collusion (100%). Not applied to surveyors (institutional
discipline) or tenants/owners (civil/legal). No token, no fee-yield, no
profit-sharing.

Full threat model: `docs/book-of-intents/THREAT_MODEL.md`.

---

## 4. Attestation Format (W3C VC + SD-JWT)

Migration from a custom attestation format to W3C Verifiable Credentials 2.0
with SD-JWT selective disclosure. The `VerificationAttestation` account now
anchors:

- `vc_type`: "VerificationAttestation" or "ResidencyAttestation"
- `issuer`: validator public key
- `credential_subject`: parcel / owner / tenant public key
- `issuance_date` / `expiration_date`: unix seconds
- `sd_jwt_digest`: SHA-256 of the serialized SD-JWT
- `selective_disclosure`: boolean flag

The full SD-JWT is held off-chain and can be selectively disclosed — the holder
reveals only chosen claims. The on-chain digest anchors it: anyone can
recompute `sha256(sd_jwt)` and compare.

---

## 5. Validator Routing (VRF)

Multi-factor selection per Design Rule 3: eligibility → capability →
jurisdiction → geographic relevance → availability → reputation → independence
→ conflict → randomized pick.

The route seed was changed from `task_id ⊕ blockhash ⊕ slot` (blockhash-only,
prohibited) to `task_id ⊕ VRF randomness ⊕ slot`. The VRF randomness is
obtained off-chain from Switchboard VRF (primary) or Pyth Entropy (fallback)
and passed as an instruction argument. On-chain, only the seed mix occurs —
blockhash is never a randomness source.

---

## 6. Spatial Storage (Irys/Arweave)

3D reconstruction blobs moved off-chain to Irys/Arweave. The `ThreeDModel`
account now stores:

- `model_hash`: SHA-256 content digest of the off-chain bundle
- `h3_index`: H3 cell index for spatial lookup

No `storage_reference` URL or IPFS CID is stored on-chain. The globe client
resolves models by H3 cell + content hash, streaming the referenced Irys/
Arweave bundle.

---

## 7. Roadmap

| Phase | Scope | Status |
|---|---|---|
| Sprint A | Localnet lifecycle smoke (parcel → dispute) | Delivered |
| Sprint B | Real write path: IDL/client `program.methods`, fix PDAs, wire UI | In progress |
| Sprint C | Chain indexer: RPC polling, mirror upserts, DB-vs-chain audit | Planned |
| Sprint D | Devnet hosting: hosted API + PostGIS + private RPC, Pages deploy | Q-001 gated |
| Sprint E | Routing/economics hardening: bonds, reputation, VRF ADR | Planned |
| Sprint F | Evidence durability: S3 + pinning, quarantine, retention, PoP | Planned |
| Sprint G | Vision features: institutional credentials, residency attestation | Q-003/Q-004 gated |

Devnet funding (Q-001) blocks Sprint D; founder ratification (Q-003/Q-004)
blocks Sprint G. All other work is zero-SOL and unblocked.

---

## 8. Team

Terra is an open-source spatial identity and land-tenure protocol. The MVP
represents a single flow end-to-end, demonstrating that residency attestation
can be anchored on-chain with W3C Verifiable Credentials, dual-signed by
owner+tenant, and presented to any verifier — without tokens, without
speculative economics, and with zero budget.

**Repository:** `github.com/terra-org/terra`
**Live MVP:** `https://mrobadiahej.github.io/terra/mvp`
**License:** Apache 2.0

---

*Constraints honored: zero budget (free tiers only), no M3D/matrical currency
(separate project), no tokenomics beyond SOL fee payments for MVP.*
