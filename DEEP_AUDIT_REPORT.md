# Terra Deep-Audit Report

**Date:** 2026-11-19

**Repository:** github.com/MrObadiahEJ/terra

**Branch:** dev

**Commit:** `4eb9ed6` + Sprint E/F working tree

**Auditor:** Internal (Kilo agent)

**Status:** Draft — for investor review

---

## Executive Summary

Terra is a Solana-based spatial identity and land-tenure protocol. The MVP implements
end-to-end runtime scene resolution grounded in publisher-signed manifests, per-row
mirror hash anchoring, server-side two-point road routing with an allowlisted VRF
oracle, and a distributed storage-provider economy with N-of-M replication auditing,
per-replica usage metering, and a Merkle-claim settlement rail that defaults to
native SOL and accepts any registered payout asset.

Off-chain, the MVP flow (Intent 5) now performs real cryptographic verification:
owner and tenant Ed25519 residency signatures, W3C VC issuer binding, and the
on-chain attestation digest are all checked, and a forged signature is rejected by
an automated test.

This report covers: security architecture, dependency analysis, BPF adversarial
testing, authority/pause key mapping, and known gaps.

---

## 1. Security Architecture

### 1.1 Trust Model

| Layer | Trust Mechanism | Verification |
|-------|----------------|--------------|
| Execution | Solana devnet (program-derived addresses) | On-chain account ownership |
| Manifests | Ed25519-signed `terra.manifest/v1` envelopes | Content-address + signature re-verification |
| Runtime scenes | Signed manifest binding (`signed_content_address`) | Fail-closed on missing/unverifiable envelope |
| Storage providers | Proof-of-possession challenges + stake bonds | Ed25519 oracle signature over the challenge payload (legacy SHA-256 fallback when no oracle key is configured) |
| Access control | `ApiAuthority` Ed25519 verifying key | Signature verification on privileged endpoints |
| Evidence intake | SHA-256 content hash + MIME/size quarantine | Content-addressed storage, S3/local backend, quarantined rows unservable |
| Evidence durability | S3-compatible backend (Filebase/R2/MinIO) | `S3_*` env selection; local disk retained as fallback |
| Runtime scene resolution | Signed manifest + scene node content addresses | Client-side Ed25519 verification + on-chain digest cross-check |
| Residency verification | Dual owner+tenant Ed25519 signatures | `terra-web/src/lib/mvpVerifier.ts` unit-tested against forgeries |
| Provider settlement | Merkle-claim epochs over a payout-asset registry | SHA-256 inclusion proofs bind the currency; per-provider settled counters prevent replay; default asset is issuer-free SOL |

### 1.2 Key Roles

| Role | PDA Seeds | Authority |
|------|-----------|-----------|
| Manifest publisher | `["manifest", publisher]` | Ed25519 signing key |
| Storage provider | `["provider", provider_id]` | Stake bond + PoP challenges |
| Worker | `["worker", worker_id]` | Registration + assignment |
| Security request | `["security_request", requester, nonce]` | Requester signature |
| Review case | `["review_case", requester, nonce]` | Committee assignment |
| Emergency protection | `["emergency_protection", ...]` | Committee + appeal |
| VRF Oracle (routing) | `["vrf_authority"]` PDA | Admin-allowlisted key; program recomputes the attested output |
| VRF Oracle (storage PoP) | `VRF_ORACLE_PUBKEY` env var | Ed25519 signature verification on `prove_replica` |

### 1.3 Slashing Conditions

| Offense | Evidence | Slash | Appeal |
|---------|----------|-------|--------|
| Equivocation | Two conflicting attestations for same parcel+slot | 100% of stake | RFC-005 appeal path |
| Liveness failure | Missed quorum window on routed task | 25% of stake | RFC-005 appeal path |
| Collusion | Same validator signs both owner+tenant residency | 100% of stake | RFC-005 appeal path |
| Failed PoP | Provider fails proof-of-possession challenge | Proportional to stake_bond | N/A (automatic) |

---

## 2. Dependency Analysis

### 2.1 Cargo Dependencies (terra-api)

| Crate | Version | Purpose | Risk |
|-------|---------|---------|------|
| solana-sdk | 3 | Program development | Low (Solana-maintained) |
| solana-program | 3 | Program development | Low |
| solana-account-decoder | 3 | Account deserialization | Low |
| solana-client | 3 | RPC client | Low |
| sqlx | 0.8 | PostgreSQL client | Medium (migration tracking) |
| axum | 0.8 | Web framework | Low |
| serde | 1 | Serialization | Low |
| sha2 | 0.10 | Cryptographic hashing | Low |
| ed25519-dalek | 2 | Ed25519 signatures | Low |
| ark-bn254 / ark-ff / ark-groth16 | 0.5 | ZK proofs | Medium (complexity) |
| terra-geo | path | Geo/spatial library | Low (internal) |
| terra-zk-membership | path | ZK membership proofs | Implemented (RFC-011 zone sets, ownership roots, nullifier records) |

### 2.2 ZK / Cryptographic Components

| Component | Status | Risk |
|-----------|--------|------|
| `terra-zk-membership` | Implemented | Low — zone sets, ownership roots, nullifier records implemented per RFC-011 |
| RFC-011 ZK ownership proofs | Implemented | Zone sets, ownership roots, nullifier records, Groth16 verification |
| `Groth16` verifier | Implemented | ark_groth16 ProvingKey verification in `api/src/zk.rs` |

---

## 3. BPF Adversarial Test Results

### 3.1 Test Suite Status

```
Registry BPF integration   307 passed
Identity BPF integration    23 passed
Registry lib              154 passed
Identity lib                7 passed
Registry security suite     9 passed
RFC-012 structure suite    21 passed
API integration (PostGIS) 164 passed
Frontend (vitest)         134 passed

Failed: 0
Ignored: 0
```

The registry suite includes the negative VRF case: an oracle that is not the
admin-allowlisted `VrfAuthority` is rejected with `VrfAuthorityMismatch` (6269)
even when it produces a well-formed Ed25519 attestation and matching output.

### 3.2 Test Coverage

All registry and identity BPF integration tests pass, covering:
- REQ-026/027/028: Manifest, runtime, providers APIs
- REQ-046: Security & privacy governance (SecurityRequest, PrivacyRequest, review cases, appeals)
- REQ-047: Component artifacts & storage providers
- REQ-048: SDK shift & protocol alignment
- RFC-012 Phase 6: dynamic routing, including allowlisted-oracle rejection
- RFC-015: Component packages & component versions
- RFC-017: Secure evidence intake, quarantine and TTL purge
- RFC-018: Dual-signature residency proof and VC digest verification
- ADR-020: Quadtree cells & spatial indexing
- ADR-021: Content-addressed manifests

Additional API coverage added in this pass:
- Premium-category discovery guarantees ≥1 replica from the requested provider
  type and randomly fills remaining slots; a low-reliability colluder in the
  requested category is not selected.
- Retrieval increments `access_count`; downloads add `bytes_served`.
- Epoch close commits a Merkle root and a claim settles exactly once.
- Scheduled PoP maintenance challenges due replicas and does not replace a
  pending challenge.
- Evidence quarantine blocks serving of disallowed types/oversize files, and
  the TTL job deletes only expired rows.

---

## 4. Authority Map

### 4.1 Program IDs

| Program | ID | Source |
|---------|-----|--------|
| terra_registry | `GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage` | `terra-core/programs/terra_registry/src/lib.rs` |
| terra_identity | `HyjpaHY9yQg1KneLZGjpLDQfP9WV4LUEbNZcb7C4Vo6p` | `terra-core/programs/terra_identity/src/lib.rs` |

### 4.2 Pause / Admin Keys

| Authority | Location | Purpose |
|-----------|----------|---------|
| `ApiAuthority` | `terra-core/api/src/auth.rs` | API-level privileged endpoints |
| Program upgrade authority | Anchor program upgrade key | Program upgrades (Solana cluster) |
| Manifest publisher | Ed25519 key in manifest envelope | Signed manifest issuance |
| Storage provider | `wallet_address` + `stake_bond` | Bond settlement + reward payouts |
| VRF Oracle (routing) | `VrfAuthority` PDA, admin-gated `init_vrf_authority` | Rejects any non-allowlisted oracle |
| VRF Oracle (storage PoP) | `VRF_ORACLE_PUBKEY` env var | Ed25519 signature verification on `prove_replica` |

### 4.3 No Single Point of Failure

- No single key can drain funds (escrows require both buyer + seller)
- No single key can modify manifests (requires publisher signature)
- No single key can slash workers (requires committee + evidence)
- Emergency protection can be lifted (requires `emergency_lift` instruction)
- The routing VRF oracle is an on-chain admin-allowlisted key, not a caller-supplied
  argument; rotating it requires the registry admin
- The storage-PoP oracle key is configurable via env var and falls back to the
  legacy SHA-256 proof only when unset (documented deployment gap, see §5)

---

## 5. Known Gaps

| Gap | Severity | Mitigation | Owner |
|-----|----------|------------|-------|
| No devnet deployment | High | Q-001 blocked (deployer wallet unfunded, ~15.7 SOL rent + fees) | Founder |
| No live-cluster verification | High | Gated on Q-001 devnet funding | Founder |
| No managed PostGIS / free hosting yet | High | Render + Supabase/Neon config documented; credentials not provisioned | Founder |
| Durable evidence backend not provisioned | Medium | S3-compatible backend implemented and env-selectable (`S3_BUCKET`/`S3_ENDPOINT`/`S3_ACCESS_KEY`/`S3_SECRET_KEY`/`S3_REGION`); bucket credentials outstanding | Founder |
| Storage-PoP oracle key not pinned in production | Medium | Set `VRF_ORACLE_PUBKEY`; otherwise the legacy SHA-256 fallback is used | Dev work |
| No R2/CDN hot tier | Low | S3-compatible backend covers durability; CDN is an optimization | Dev work |
| Audit is internal | Medium | External review not yet commissioned | Founder |

---

## 6. Compliance

| Requirement | Status | Evidence |
|-------------|--------|----------|
| REQ-027 (Manifest API) | Implemented | `api/src/routes/manifests.rs` |
| REQ-028 (Runtime discovery) | Implemented | `api/src/routes/runtime.rs` |
| REQ-013 (Routing) | Implemented | `api/src/routes/geo.rs` + `geo-engine/src/reach.rs` |
| RFC-015 (Component packages) | Implemented | `api/src/routes/components.rs` |
| RFC-014 (Settlement rail direction) | Implemented | `api/src/routes/settlement.rs` + migration 0054 |
| RFC-017 (Evidence intake) | Implemented | `api/src/routes/evidence.rs` (quarantine + TTL) |
| RFC-018 (Residency verification) | Implemented | `terra-web/src/lib/mvpVerifier.ts` + `MvpFlowPage.tsx` |
| Q-008 (Storage independence) | Implemented | N-of-M audit + provider economy + S3/IPFS backend options |
| Q-007 (Server-side routing) | Implemented | `GET /geo/route` with allowlisted VRF |
| Q-005 (No security/token) | Enforced | Settlement is `settlement_kind='service_fee'`; payout assets are registry rows (default SOL, optional USDC/EURC), never a token or yield instrument |
| Q-001 (Devnet funding) | Blocked | Deployer wallet unfunded |

---

## 7. Recommendations

### Immediate (0-30 days)
1. **Q-001**: Fund the deployer wallet and run the registry devnet deploy
2. **Free hosting**: Provision Supabase/Neon PostGIS + Render API, set
   `DATABASE_URL`, `TERRA_RPC_URL`, `API_AUTHORITY_PUBKEY`,
   `CORS_ALLOWED_ORIGINS`, and point Pages at the API
3. **Evidence durability**: Create the S3-compatible bucket and set the `S3_*`
   variables so evidence survives Render restarts
4. **Pin the PoP oracle**: set `VRF_ORACLE_PUBKEY` in the deployed API

### Short-term (30-90 days)
1. **Sprint E remainder**: staging bond PDA + reputation-weighted throughput
2. **Sprint F remainder**: external uptime monitoring of a DB-touching endpoint
3. **Settlement operations**: run epoch close on a schedule and document the
   payout process for each enabled rail (default SOL; USDC/EURC only when a
   provider opts in)

### Long-term (90+ days)
1. **Sprint G**: Institutional validator credentials (vLEI/DNSSEC)
2. **ZK**: Production membership circuit + trusted setup audit
3. **Token**: No token work (Q-005 gate remains)

---

## 8. Conclusion

The Terra MVP is functionally complete on localnet with all core features implemented:
runtime scene resolution, signed manifest verification, mirror hash anchoring,
server-side routing with an allowlisted VRF, a storage-provider economy with N-of-M
replication auditing, per-replica metering, and a service-fee settlement rail
whose default payout asset is issuer-free SOL.
Evidence intake now quarantines bad uploads, purges expired rows on a 15-minute
maintenance loop, and schedules PoP challenges automatically; the MVP flow verifies
real Ed25519 residency signatures, the VC issuer and the on-chain digest.

Gates pass: `cargo fmt --check`, `cargo clippy -D warnings`, registry BPF 307/307,
identity BPF 23/23, registry lib 154, identity lib 7, security 9, RFC-012 21,
API 164 (PostGIS), frontend `tsc` clean, vitest 134/134, and `build:check`
(1901.5 kB gz total, budget 1950).

The primary blocker to public deployment is operational, not code: Q-001 devnet
funding plus free-hosting and bucket credentials, which require founder action.
