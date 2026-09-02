# RFC-006: Cross-Border Identity Bridge Protocol

## 1. Status

- **Status:** Draft
- **Created:** 2026-09-03
- **Target Phase:** 7 (Regional Expansion) at earliest
- **Supersedes:** None

## 2. Summary

This RFC specifies the **Cross-Border Identity Bridge Protocol** — a jurisdiction-scoped extension to Terra's existing Identity account that enables a person in Country A to transact on a parcel in Country B without exposing Country A's raw credential to Country B's validators. The design adds a jurisdiction tag and a zero-knowledge membership proof to the existing `identity_hash` model. Country A issues a W3C Verifiable Credential (or equivalent), the holder generates a ZK proof of membership in that jurisdiction's credential set, and Country B's validators verify the proof on-chain without ever seeing the underlying ID number, name, or photo. Revocation is handled by a jurisdiction-scoped authority that can invalidate a binding if the original credential is revoked.

**Primitive-spirited framing:** This is not a new identity system. It is a natural extension of the existing Identity account — the addition is a jurisdiction tag plus a zero-knowledge membership proof, not a new credential.

**Warnings:**
- ZK circuit design is a specialized skill, not a library import. Building and auditing a correct ZK-SNARK/STARK circuit is a multi-month specialization. The circuits specified here are design-level; they require formal specification, independent audit, and test-vector validation before implementation.
- Surveillance and sanctions surface: cross-jurisdiction identity binding touches KYC/AML law and export-control concerns. Legal review is mandatory before any pilot deployment.

## 3. Threat Model

### 3.1 Adversary Classes

| Class | Description | Mitigation |
|-------|-------------|------------|
| **Rogue jurisdiction authority** | Country A issues a credential to an ineligible person | ZK proof is bound to a specific credential commitment; revocation is immediate on-chain |
| **Colluding validators** | k validators in Country B verify a forged proof | ZK-SNARK verification is trustless on-chain; validators cannot bypass the proof |
| **Credential harvester** | Harvests identity_hash + jurisdiction_tag to correlate across countries | identity_hash is a salted hash, not the raw credential; ZK proof reveals nothing about the credential |
| **Replay attacker** | Reuses a valid ZK proof across jurisdictions or after revocation | Proof includes jurisdiction_tag and nullifier; on-chain nullifier prevents reuse; revocation check is mandatory |
| **Cross-jurisdiction sybil** | Creates multiple bindings for the same credential | Nullifier derived from credential commitment prevents double-binding |
| **Platform operator** | API server or infrastructure provider | ZK proof is verified on-chain; operator never sees the credential or proof inputs |
| **Future adversary** | Harvests proofs today, waits for cryptographic break | Post-quantum migration path via algorithm_id (STARK fallback) |
| **Jurisdiction exit attacker** | Country A revokes Alice's ID but Alice already transacted in Country B | Revocation check is mandatory at verification time; grace period before binding becomes invalid |

### 3.2 In Scope

- Cross-border identity binding with jurisdiction-scoped ZK proofs
- Credential format translation (W3C Verifiable Credentials ↔ country-specific formats)
- Revocation propagation (Country A revokes → Country B's on-chain binding reflects this)
- Nullifier-based proof uniqueness (prevent double-binding, prevent proof reuse)
- On-chain verification of jurisdiction membership without personal data exposure

### 3.3 Out of Scope

- The actual credential issuance process in Country A (country-specific)
- KYC/AML compliance logic (application-layer concern, varies by jurisdiction)
- W3C Verifiable Credential schema design (standards-body concern)
- Cross-border parcel transfer logic (covered by future RFC)
- Validator identity verification (covered by AuthorityRegistry)

## 4. Cryptographic Choices

### 4.1 ZK Proof System: Groth16 (SNARK) or FRI-based (STARK)

- **Pilot:** Groth16 over BN254. Small proof size (~128 bytes), fast on-chain verification (~200k compute units).
- **Production:** FRI-based STARK if post-quantum resistance is required (no trusted setup, larger proofs).
- The `algorithm_id` field on `JurisdictionBinding` controls which verifier is called.

### 4.2 Hash Function: SHA-256

- Used for `identity_hash` (SHA-256 of credential, already established in Identity account)
- Used for nullifier derivation: `nullifier = SHA-256(credential_commitment || jurisdiction_tag || nonce)`
- Used for on-chain integrity checks

### 4.3 Nullifier Scheme

- Derive a unique nullifier from the credential commitment and jurisdiction tag.
- The nullifier prevents:
  - Double-binding: same credential cannot bind to two jurisdictions under the same identity
  - Proof reuse: a proof cannot be submitted twice (nullifier is recorded on-chain)
- Nullifier derivation: `nullifier = SHA-256(credential_commitment || jurisdiction_tag || domain_sep)`

### 4.4 Credential Commitment

- Country A generates a Pedersen commitment to the credential: `commitment = g^r * h^credential_hash`
- The commitment is published on-chain (or referenced via jurisdiction binding).
- The ZK proof proves knowledge of the opening of this commitment and that the committed credential is valid in Country A's registry.

### 4.5 Signature: Ed25519

- Solana native — all validator keys are Ed25519.
- Used for transaction signing (on-chain authorization).
- Used for off-chain protocol messages (jurisdiction authority coordination).

### 4.6 Post-Quantum Migration: Algorithm ID

- `u8` enum in `JurisdictionBinding`: `0 = Groth16 (BN254)`, `1 = FRI-STARK (post-quantum)`
- Only variant 0 is implemented for the pilot.
- Migration to variant 1 happens when a binding's proof is re-generated under the STARK circuit.

## 5. Data Model

### 5.1 Identity Account (existing, unchanged)

**PDA seed:** `["identity", identity_hash]`

| Field | Type | Description |
|-------|------|-------------|
| `identity_hash` | `[u8; 32]` | SHA-256 of credential (never raw) |
| `owner` | `Pubkey` | Active wallet acting on behalf of this identity |
| `recovery` | `Pubkey` | Backup / recovery wallet |
| `parcel_count` | `u16` | Number of parcels currently owned |
| `created_at` | `i64` | Creation timestamp |
| `updated_at` | `i64` | Last update timestamp |

> **Note:** The Identity account is NOT modified by this RFC. The cross-border extension adds new PDAs that reference the existing identity_hash.

### 5.2 Jurisdiction PDA (new)

**PDA seed:** `["jurisdiction", country_code]`

| Field | Type | Description |
|-------|------|-------------|
| `country_code` | `[u8; 16]` | ISO 3166-1 alpha-2 padded to 16 bytes (e.g., `b"KE\x00\x00..."` for Kenya) |
| `authority` | `Pubkey` | The jurisdiction authority wallet (issues/revokes credentials) |
| `jurisdiction_name` | `String` | Human-readable name (max 64 chars) |
| `credential_schema_cid` | `String` | IPFS CID of the W3C Verifiable Credential schema |
| `revocation_registry` | `Pubkey` | On-chain or oracle reference for revocation checks |
| `verification_key_hash` | `[u8; 32]` | SHA-256 of the ZK verification key for this jurisdiction's circuit |
| `algorithm_id` | `u8` | 0 = Groth16, 1 = FRI-STARK |
| `status` | `u8` | 0 = Active, 1 = Suspended, 2 = Withdrawn |
| `created_at` | `i64` | Registration timestamp |
| `updated_at` | `i64` | Last update timestamp |

### 5.3 JurisdictionBinding PDA (new)

**PDA seed:** `["cross_border_identity", jurisdiction_key, identity_hash]`

This is the core on-chain record that binds a person (via their identity_hash) to a jurisdiction with a ZK proof.

| Field | Type | Description |
|-------|------|-------------|
| `identity_hash` | `[u8; 32]` | The identity_hash from the existing Identity account |
| `jurisdiction_key` | `Pubkey` | The Jurisdiction PDA this binding belongs to |
| `credential_commitment` | `[u8; 32]` | Pedersen commitment to the credential (published by Country A's authority) |
| `nullifier` | `[u8; 32]` | Derived nullifier to prevent double-binding and proof reuse |
| `proof_data` | `Vec<u8>` | The serialized ZK proof (max 512 bytes) |
| `proof_version` | `u8` | Version of the proof circuit (for future upgrades) |
| `algorithm_id` | `u8` | 0 = Groth16, 1 = FRI-STARK |
| `revoked` | `bool` | Whether this binding has been revoked |
| `revoked_at` | `i64` | Timestamp of revocation (0 if not revoked) |
| `revoked_by` | `Pubkey` | Who revoked this binding (default = authority) |
| `bound_at` | `i64` | When the binding was created |
| `expires_at` | `i64` | Optional expiry (0 = no expiry) |
| `version` | `u32` | Monotonic counter, bumped on re-verification |

### 5.4 JurisdictionRevocationLog (emitted as event)

| Field | Type | Description |
|-------|------|-------------|
| `jurisdiction` | `Pubkey` | Jurisdiction PDA |
| `identity_hash` | `[u8; 32]` | The revoked identity |
| `binding` | `Pubkey` | The JurisdictionBinding PDA |
| `reason` | `String` | Human-readable reason (max 128 chars) |
| `revoked_by` | `Pubkey` | Who performed the revocation |
| `block_time` | `i64` | Solana block time |

## 6. Instructions

### 6.1 `register_jurisdiction`

**Purpose:** Register a new jurisdiction (country/zone) in the system. Only the registry admin can register jurisdictions.

**Accounts:**
- `jurisdiction` (init, PDA `["jurisdiction", country_code]`)
- `authority` (signer, mut — pays rent; must be registry admin)
- `registry` (readonly — AuthorityRegistry account)
- `system_program`

**Args:** `country_code: [u8; 16]`, `jurisdiction_name: String`, `credential_schema_cid: String`, `revocation_registry: Pubkey`, `verification_key_hash: [u8; 32]`, `algorithm_id: u8`

**Guards:**
- `authority.key() == registry.admin` (only registry admin can register jurisdictions)
- `country_code` is non-zero (at least one non-zero byte)
- `credential_schema_cid` is non-empty
- `verification_key_hash != [0; 32]`
- `algorithm_id` is supported (0 or 1)
- `jurisdiction_name.len() <= 64`
- No jurisdiction PDA with this `country_code` already exists (init handles this)

**Effects:**
- Creates `Jurisdiction` account with the provided parameters
- `status = Active`
- `created_at = now`, `updated_at = now`

**Emits:** `JurisdictionRegistered`

### 6.2 `update_jurisdiction`

**Purpose:** Update jurisdiction parameters (e.g., rotate verification key, change status).

**Accounts:**
- `jurisdiction` (mut)
- `authority` (signer — must be jurisdiction.authority)
- `system_program`

**Args:** `new_verification_key_hash: Option<[u8; 32]>`, `new_revocation_registry: Option<Pubkey>`, `new_status: Option<u8>`

**Guards:**
- `authority.key() == jurisdiction.authority`
- At least one argument is `Some`
- If `new_status` is `Some`, value must be 0, 1, or 2

**Effects:**
- Updates the provided fields
- `updated_at = now`

**Emits:** `JurisdictionUpdated`

### 6.3 `bind_cross_border_identity`

**Purpose:** Bind an identity to a jurisdiction with a ZK proof. The prover demonstrates knowledge of a valid credential in the jurisdiction without revealing the credential itself.

**Accounts:**
- `binding` (init, PDA `["cross_border_identity", jurisdiction_key, identity_hash]`)
- `identity` (readonly — existing Identity account)
- `jurisdiction` (mut — Jurisdiction PDA)
- `prover` (signer, mut — pays rent; must be identity.owner or identity.recovery)
- `system_program`

**Args:** `credential_commitment: [u8; 32]`, `proof_data: Vec<u8>`, `nullifier_nonce: [u8; 32]`, `expires_at: i64`

**Guards:**
- `prover.key() == identity.owner || prover.key() == identity.recovery`
- `jurisdiction.status == Active`
- `proof_data.len() <= 512`
- `proof_data.len() > 0`
- `credential_commitment != [0; 32]`
- `nullifier_nonce != [0; 32]`
- `expires_at == 0 || expires_at > now`
- ZK proof verification passes against `jurisdiction.verification_key_hash` (off-chain verification delegated to application layer; on-chain stores the proof and relies on validator attestation — see §7)
- No existing binding with the same `identity_hash` and `jurisdiction_key` (init handles this)
- No nullifier collision (the derived nullifier is unique per credential commitment + jurisdiction)

**Effects:**
- Creates `JurisdictionBinding` account
- `nullifier = SHA-256(credential_commitment || jurisdiction_key || nullifier_nonce)`
- `revoked = false`
- `bound_at = now`, `version = 0`
- `identity.parcel_count` is NOT modified (this is a jurisdiction binding, not a parcel ownership change)

**Emits:** `CrossBorderIdentityBound`

### 6.4 `verify_jurisdiction_membership`

**Purpose:** A validator verifies that a binding is valid and the underlying credential has not been revoked. This is the on-chain check that Country B's validators perform before allowing a cross-border transaction.

**Accounts:**
- `binding` (mut — JurisdictionBinding PDA)
- `jurisdiction` (readonly — Jurisdiction PDA)
- `identity` (readonly — Identity account)
- `validator` (signer — must be in the registry)
- `registry` (readonly — AuthorityRegistry)
- `system_program`

**Args:** `off_chain_nonce: [u8; 32]`

**Guards:**
- `validator.key()` is in `registry.validators`
- `binding.revoked == false`
- `binding.jurisdiction_key == jurisdiction.key()`
- `binding.identity_hash == identity.identity_hash`
- `binding.expires_at == 0 || binding.expires_at > now`
- `off_chain_nonce != [0; 32]`
- Revocation check: the application layer must query `jurisdiction.revocation_registry` and confirm the credential is still valid. The on-chain instruction records the verification event but does NOT perform the revocation check itself (gas cost). The validator's signature attests that the check was performed off-chain.

**Effects:**
- `binding.version = binding.version.saturating_add(1)`

**Emits:** `JurisdictionMembershipVerified`

### 6.5 `revoke_jurisdictional_identity`

**Purpose:** Revoke a binding if the original credential has been revoked by Country A's authority. Can be called by the jurisdiction authority or by the identity owner themselves.

**Accounts:**
- `binding` (mut — JurisdictionBinding PDA)
- `jurisdiction` (readonly — Jurisdiction PDA)
- `identity` (readonly — Identity account)
- `authority` (signer — must be jurisdiction.authority or identity.owner or identity.recovery)
- `system_program`

**Args:** `reason: String`

**Guards:**
- `authority.key() == jurisdiction.authority || authority.key() == identity.owner || authority.key() == identity.recovery`
- `binding.revoked == false`
- `reason.len() <= 128`

**Effects:**
- `binding.revoked = true`
- `binding.revoked_at = now`
- `binding.revoked_by = authority.key()`
- `binding.version = binding.version.saturating_add(1)`

**Emits:** `JurisdictionIdentityRevoked`

### 6.6 `rebind_cross_border_identity`

**Purpose:** Re-bind after a verification key rotation or credential refresh. The old binding is superseded by a new one with an updated proof.

**Accounts:**
- `old_binding` (mut — existing JurisdictionBinding PDA)
- `new_binding` (init, PDA `["cross_border_identity", jurisdiction_key, identity_hash]` — same seeds, closes old)
- `identity` (readonly — Identity account)
- `jurisdiction` (mut — Jurisdiction PDA)
- `prover` (signer, mut — pays rent; must be identity.owner or identity.recovery)
- `system_program`

**Args:** `credential_commitment: [u8; 32]`, `proof_data: Vec<u8>`, `nullifier_nonce: [u8; 32]`, `expires_at: i64`

**Guards:**
- `old_binding.identity_hash == identity.identity_hash`
- `old_binding.jurisdiction_key == jurisdiction.key()`
- `prover.key() == identity.owner || prover.key() == identity.recovery`
- `jurisdiction.status == Active`
- `proof_data.len() <= 512`
- `proof_data.len() > 0`
- `credential_commitment != [0; 32]`
- `nullifier_nonce != [0; 32]`
- `expires_at == 0 || expires_at > now`
- ZK proof verification passes against `jurisdiction.verification_key_hash`
- Old binding must not be revoked (revoked bindings cannot be re-bound)

**Effects:**
- Close `old_binding` (lamports returned to prover)
- Create `new_binding` with updated proof, commitment, nullifier, and `version = 0`
- `bound_at = now`

**Emits:** `CrossBorderIdentityRebound`

## 7. Off-Chain Protocol

### 7.1 Jurisdiction Registration Ceremony

1. The registry admin coordinates with Country A's government or designated authority.
2. Country A provides:
   - A W3C Verifiable Credential schema (or equivalent) → uploaded to IPFS → CID.
   - A ZK circuit (prover + verifier) for proving membership in Country A's credential set.
   - The verification key (vk) → hashed → stored on-chain as `verification_key_hash`.
   - A revocation registry reference (on-chain or oracle).
3. The registry admin calls `register_jurisdiction` with the above parameters.
4. The jurisdiction is now active and can accept bindings.

### 7.2 Credential Issuance + Binding Ceremony

1. Alice lives in Country A and holds a valid national ID (credential).
2. Alice (or her agent) generates a Pedersen commitment to her credential: `commitment = g^r * h^credential_hash`.
3. Alice generates a ZK proof:
   - **Private inputs:** credential, randomness `r`, credential_hash
   - **Public inputs:** credential_commitment, jurisdiction_tag, nullifier
   - **Statement:** "I know a credential that is valid in Country A's registry and I have committed to it."
4. Alice calls `bind_cross_border_identity` on-chain with the commitment, proof, and a nullifier nonce.
5. The on-chain record stores the binding. Country B's validators can now verify Alice's jurisdiction membership without seeing Alice's credential.

### 7.3 Cross-Border Transaction Verification

1. Alice wants to transact on a parcel in Country B.
2. Alice presents her `identity_hash` and `jurisdiction_key` to Country B's validator.
3. Country B's validator:
   a. Fetches the `JurisdictionBinding` PDA from on-chain.
   b. Verifies the ZK proof against the jurisdiction's verification key (off-chain, using the vk).
   c. Queries the revocation registry to confirm Alice's credential is still valid.
   d. If both checks pass, calls `verify_jurisdiction_membership` on-chain to record the verification.
4. Country B's validator proceeds with the parcel transaction.

### 7.4 Revocation Ceremony

1. Country A revokes Alice's credential (e.g., identity theft, fraud, death).
2. Country A's jurisdiction authority calls `revoke_jurisdictional_identity` on-chain.
3. The binding is marked as revoked. Future verification attempts by Country B will fail the `binding.revoked == false` check.
4. Any in-flight transactions that relied on Alice's binding are invalid.

### 7.5 Credential Refresh Ceremony

1. Alice's credential in Country A expires or the verification key is rotated.
2. Alice generates a new ZK proof with updated parameters.
3. Alice calls `rebind_cross_border_identity` which closes the old binding and creates a new one.
4. Country B's validators see the updated binding and can continue verifying.

### 7.6 Side-Channel Requirements

- **Secure channel for proof generation:** Alice's ZK proof generation happens locally (client-side). The prover application must run on a trusted device.
- **Revocation oracle:** Country A must provide a reliable revocation oracle (on-chain program, oracle service, or signed CRL). The oracle must be queryable by Country B's validators.
- **Schema registry:** W3C Verifiable Credential schemas are stored on IPFS and referenced by CID. Schema updates require jurisdiction re-registration.

## 8. Collusion Resistance

### 8.1 Trust Minimization via ZK Proofs

- Country B's validators verify a ZK proof. The proof is a mathematical guarantee — validators cannot bypass it.
- Even if all validators in Country B collude, they cannot forge a valid proof without knowing Alice's credential.
- The only trust assumption is that Country A's jurisdiction authority correctly issues and revokes credentials.

### 8.2 Nullifier-Based Double-Binding Prevention

- The nullifier is derived from the credential commitment and jurisdiction tag.
- If Alice tries to create two bindings for the same credential in the same jurisdiction, the nullifier collides and the second `init` fails.
- If Alice tries to bind in two different jurisdictions, the nullifiers are different (jurisdiction_tag is part of the derivation), so both bindings are valid — this is the intended behavior.

### 8.3 Audit Trail

Every `verify_jurisdiction_membership` and `revoke_jurisdictional_identity` event is emitted on-chain with:
- The binding key
- The identity_hash
- The jurisdiction
- The validator who performed verification
- The block timestamp

This creates a public, immutable audit trail. Any observer can detect:
- Unusual frequency of cross-border verifications
- Revocation events and their timing
- Binding creation patterns

### 8.4 Authority Accountability

- The jurisdiction authority is a single point of trust. If Country A's authority is compromised, it can issue fraudulent credentials.
- Mitigation: the authority is a registered, auditable entity. Revocation events are public. Country B can require multiple independent revocation confirmations.
- Future improvement: threshold authority (multiple signers per jurisdiction) if trust requirements increase.

## 9. Liveness Guarantees

### 9.1 Binding Expiry

Bindings can optionally have an `expires_at` timestamp. If set, the binding becomes invalid after expiry. This prevents stale bindings from being used if the underlying credential may have changed.

### 9.2 Revocation Propagation

Revocation is immediate on-chain. Once `revoke_jurisdictional_identity` is called, the binding is marked revoked and all future verification attempts fail. There is no delay or grace period (unlike succession protocols).

### 9.3 Jurisdiction Status

A jurisdiction can be suspended or withdrawn:
- **Suspended:** No new bindings can be created. Existing bindings continue to work until revoked.
- **Withdrawn:** All bindings for this jurisdiction are effectively invalid (verification checks `jurisdiction.status == Active` implicitly through the binding's jurisdiction_key reference).

### 9.4 Offline Verification

Country B's validators can verify a ZK proof offline (the proof is self-contained). The on-chain `verify_jurisdiction_membership` call is an optional audit trail — the proof itself is the trust anchor.

## 10. Storage Architecture

### 10.1 On-Chain (Solana)

- `Jurisdiction` PDA: jurisdiction metadata, verification key hash, status
- `JurisdictionBinding` PDA: identity binding, ZK proof, nullifier, revocation state
- Events: all state transitions are emitted as events for indexing

### 10.2 IPFS

- Credential schemas: W3C Verifiable Credential schemas stored content-addressed
- Verification keys: the actual verification key bytes (the on-chain hash is for integrity)

### 10.3 Revocation Registry

- **Option A:** On-chain program maintained by the jurisdiction authority. Queries are on-chain (gas cost).
- **Option B:** Signed Certificate Revocation List (CRL) hosted by the jurisdiction authority. Validators fetch and verify the signature off-chain.
- **Option C:** Decentralized oracle (e.g., Pyth, Chainlink) for revocation data.

The choice is jurisdiction-specific and specified at registration time via `revocation_registry`.

### 10.4 PostGIS / Application Cache

- Local cache + indexing for the API layer.
- Reconstructable from on-chain data + IPFS.
- Not the source of truth — only a performance optimization.

## 11. Replay / Nonce Hygiene

### 11.1 Nullifier Uniqueness

The `nullifier` is derived deterministically from `credential_commitment || jurisdiction_key || nullifier_nonce`. The on-chain PDA seeds include the `identity_hash` and `jurisdiction_key`, ensuring:
- One binding per (identity, jurisdiction) pair
- The nullifier prevents proof reuse across different binding attempts

### 11.2 Nonce for Verification Events

The `off_chain_nonce` in `verify_jurisdiction_membership` must be unique per verification event. This prevents the same verification from being replayed to count multiple times.

### 11.3 Expiry Enforcement

If `expires_at` is set on a binding, the verification instruction checks `expires_at > now`. Stale bindings are rejected.

### 11.4 Version Bumping

Each `verify_jurisdiction_membership` call bumps `binding.version`. This allows validators to confirm they are verifying the latest state of the binding.

## 12. Post-Quantum Migration Path

### 12.1 Algorithm ID in JurisdictionBinding

The `algorithm_id` field is a `u8` enum:
- `0` = Groth16 over BN254 (current)
- `1` = FRI-based STARK (post-quantum)

### 12.2 Migration During Rebinding

When a post-quantum algorithm is ready for use:
1. The jurisdiction authority generates a new ZK circuit (STARK-based) with the same statement.
2. The jurisdiction updates its `verification_key_hash` and `algorithm_id` via `update_jurisdiction`.
3. Existing holders call `rebind_cross_border_identity` to generate a new proof under the STARK circuit.
4. The old Groth16-based binding is closed and the new STARK-based binding is created.

### 12.3 Timeline Considerations

- Groth16 is mature and widely deployed. Sufficient for the pilot (1-2 years).
- STARK-based proofs are larger (~50-200KB) but do not require a trusted setup.
- Migration should happen before the system scales beyond the pilot region.

## 13. Operational Security

### 13.1 ZK Circuit Audit

- All ZK circuits must undergo independent security audit before deployment.
- The circuit must prove: "I know a credential `c` such that `H(c)` is in the jurisdiction's registry and `commitment = g^r * h^{H(c)}`."
- The circuit must NOT leak: credential content, name, photo, or any personally identifiable information beyond the jurisdiction membership claim.

### 13.2 Verification Key Management

- The verification key is a critical trust anchor. If compromised, forged proofs become possible.
- The jurisdiction authority must store the proving key in a secure, air-gapped environment.
- Verification key rotation is supported via `update_jurisdiction` + `rebind_cross_border_identity`.

### 13.3 Revocation Oracle Security

- The revocation oracle must be highly available. If it goes offline, Country B cannot verify Alice's credential status.
- Mitigation: multiple oracle endpoints, fallback to signed CRLs, cached revocation data with TTL.

### 13.4 Jurisdiction Authority Key Management

- The jurisdiction authority's wallet (used to call `revoke_jurisdictional_identity`) must be secured with multi-sig or HSM.
- Compromise of this key allows arbitrary revocation of all bindings for that jurisdiction.

### 13.5 Client-Side Proof Generation

- Alice's ZK proof generation happens on her device. The proving key must never leave the device.
- The prover application should be open-source and auditable.
- Proof generation should use a hardware RNG for randomness.

## 14. Test Vectors

### 14.1 Jurisdiction Registration

- Registry admin: wallet A
- Country code: `b"KE\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"` (Kenya)
- Jurisdiction name: "Republic of Kenya"
- Credential schema CID: "QmKenyaSchema123..."
- Verification key hash: SHA-256 of the Kenya Groth16 verification key
- Algorithm: 0 (Groth16)
- Expected: Jurisdiction PDA created with correct fields, status = Active

### 14.2 Identity Binding with ZK Proof

- Identity: Alice (identity_hash = SHA-256("alice_credential_kenya"))
- Jurisdiction: Kenya (country_code = "KE")
- Credential commitment: random 32-byte Pedersen commitment
- Proof data: 128-byte mock Groth16 proof
- Nullifier nonce: random 32 bytes
- Expected: JurisdictionBinding PDA created with correct identity_hash, jurisdiction_key, credential_commitment, nullifier, proof_data

### 14.3 Cross-Border Verification

- Binding: Alice's Kenya binding
- Validator: wallet B (registered in AuthorityRegistry)
- Off-chain nonce: random 32 bytes
- Expected: JurisdictionMembershipVerified event emitted, binding.version incremented

### 14.4 Revocation

- Binding: Alice's Kenya binding (currently active)
- Authority: Kenya jurisdiction authority wallet
- Reason: "Credential revoked due to identity theft"
- Expected: JurisdictionIdentityRevoked event emitted, binding.revoked = true, binding.revoked_at = now

### 14.5 Verification After Revocation

- Binding: Alice's Kenya binding (revoked)
- Validator: wallet B
- Expected: `BindingRevoked` error

### 14.6 Double-Binding Prevention

- Alice tries to bind to Kenya twice with the same credential commitment
- Expected: `InitConstraintViolation` (PDA already exists) or nullifier collision error

### 14.7 Expiry Enforcement

- Binding: expires_at = now - 1 (expired)
- Validator: wallet B
- Expected: `BindingExpired` error

### 14.8 Unauthorised Revocation

- Revoker: wallet C (not jurisdiction authority, not identity owner)
- Expected: `NotAuthorizedToRevoke` error

### 14.9 Rebinding After Verification Key Rotation

- Existing binding: Alice's Kenya binding (algorithm_id = 0, Groth16)
- Jurisdiction updates verification_key_hash and algorithm_id = 1 (STARK)
- Alice calls `rebind_cross_border_identity` with new proof
- Expected: Old binding closed, new binding created with algorithm_id = 1

### 14.10 Collusion Scenario

- Validator B tries to verify a binding that belongs to a different jurisdiction
- Expected: `JurisdictionMismatch` error (binding.jurisdiction_key != jurisdiction.key())
