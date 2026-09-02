# RFC-011: Zero-Knowledge Ownership Proof Protocol

## 1. Status

- **Status:** Draft
- **Created:** 2026-09-03
- **Supersedes:** None
- **Target Phase:** 8 (Global platform) — earliest
- **Depends on:** RFC-006 (ZK credential circuit) shipped and audited successfully
- **WARNING:** This is a second, independent ZK circuit (membership + range proofs + nullifiers), not a reuse of RFC-006's circuit. Treat as its own multi-month specialization requiring dedicated ZK expertise or an external audit partner. Do not schedule until RFC-006 has been audited and deployed — this is a harder version of the same unproven capability. ZK-expertise warning doubled.

## 2. Summary

**Program ID:** `GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage`

This RFC specifies the **Zero-Knowledge Ownership Proof Protocol** — a privacy-preserving layer that allows Terra participants to prove ownership attributes about their parcels without revealing which parcels they own, how many they own, or who they are. Proofs are constructed as ZK-SNARKs (or STARKs) over a Merkle-committed set of parcel owners, with nullifier-based double-use prevention and selective disclosure granularity.

**Use cases:**

- A landowner qualifies for a government subsidy by proving they own at least one parcel in a zone, without revealing their identity or parcel count.
- A landowner votes in a landowners' association by proving membership in the zone's owner set, without revealing specific holdings.
- A borrower proves solvency by proving they own parcels worth > $X in aggregate, without revealing exact values or parcel identities.
- A developer proves they control a threshold number of parcels in a jurisdiction for zoning applications.

The on-chain program never sees plaintext ownership data, parcel identifiers, or wallet addresses in proofs — it only verifies cryptographic proofs, manages Merkle roots, and tracks nullifiers.

## 3. Threat Model

### 3.1 Adversary Classes

| Class | Description | Mitigation |
|-------|-------------|------------|
| **Proof replayer** | Reuses a valid proof to claim ownership multiple times | Nullifier hash recorded on-chain; double-use rejected |
| **Proof sharing** | Prover shares proof with a non-owner to transfer false credentials | Nullifier is bound to a specific commitment; range proofs are selective-disclosure |
| **Colluding provers** | Multiple owners coordinate to fabricate false aggregate claims | Nullifiers prevent double-counting; Merkle root is zone-scoped and versioned |
| **On-chain observer** | Reads all on-chain data to deanonymize provers | Only nullifier hashes and Merkle roots are on-chain; no parcel IDs, no wallets |
| **Range oracle** | Attempts to infer exact values from range proofs | Pedersen commitments with Pedersen hash; statistically hiding |
| **Merkle root manipulator** | Tries to present an old or forged root | Roots are signed by zone authority and stored on-chain with version counters |
| **Future adversary** | Harvests proofs today, waits for cryptographic break | Post-quantum migration path via algorithm_id + root rotation |

### 3.2 In Scope

- ZK-SNARK/STARK membership proofs (prove ownership of a parcel in a zone)
- Range proofs (prove aggregate value exceeds a threshold)
- Nullifier system (prevent double-proving)
- Selective disclosure (prove "at least 1" vs. "exactly N" parcels)
- Revocation on ownership transfer (old proofs invalidated)
- On-chain verification of proofs
- Merkle root management for zone owner sets
- Post-quantum migration path

### 3.3 Out of Scope

- Identity verification (covered by AuthorityRegistry and RFC-006)
- Parcel valuation (application-layer concern; the circuit accepts pre-committed values)
- Cross-zone proofs (single zone per proof; cross-zone requires separate circuits)
- The specific ZK proving system implementation (SNARK vs. STARK is a deployment choice)
- Prover infrastructure (client-side concern; the on-chain program only verifies)

## 4. Cryptographic Choices

### 4.1 Zero-Knowledge Proofs: Groth16 (SNARK) or PLONK

- **Groth16:** Smallest proof size (~128 bytes), fastest on-chain verification, requires per-circuit trusted setup
- **PLONK:** Universal trusted setup, slightly larger proofs, better for iterative development
- Deployment choice depends on circuit stability and audit results
- Both produce constant-size proofs regardless of the owner set size

### 4.2 Merkle Tree: Poseidon Hash

- Poseidon is ZK-friendly (optimized for arithmetic circuits)
- Hash rate: ~1 constraint per element (vs. ~10k for SHA-256 in a circuit)
- Tree depth: 20 levels (supports ~1M parcels per zone)
- Branch size: 20 hashes × 32 bytes = 640 bytes per proof

### 4.3 Commitments: Pedersen Commitments

- For range proofs: commit to parcel values without revealing them
- Additively homomorphic: allows aggregate range proofs (sum of commitments = commitment of sum)
- Blinding factor generated per proof (statistically hiding)

### 4.4 Nullifier System: Poseidon Hash of (owner_commitment, zone_root)

- Nullifier = Poseidon(owner_commitment, zone_root_version)
- Owner commitment is a Pedersen commitment to the owner's public key and parcel IDs
- Binding: each owner can only produce one valid proof per zone root version
- Hiding: nullifier reveals nothing about the owner's identity

### 4.5 Hashing: SHA-256

- On-chain data integrity (Merkle root storage, nullifier records)
- Not used inside ZK circuits (too expensive); Poseidon is used there

### 4.6 Signatures: Ed25519

- Solana native — zone authority signs Merkle roots
- Used for transaction signing (on-chain instruction authorization)
- Used for off-chain proof generation authorization

### 4.7 Post-Quantum Migration: Algorithm ID

- `u8` enum in ZoneOwnershipRoot: `0 = Poseidon + Groth16`, `1 = PQ-secure hash + STARK`
- Only variant 0 is implemented for the pilot
- The `generate_ownership_root` flow can re-commit under a new algorithm without program upgrade

## 5. Data Model

### 5.1 ZoneSet (on-chain PDA)

**PDA seed:** `["zone_set", zone_id]`

| Field | Type | Description |
|-------|------|-------------|
| `zone_id` | `Pubkey` | Authority-paused zone identifier |
| `authority` | `Pubkey` | Zone administrator (AuthorityRegistry validator) |
| `parcel_count` | `u32` | Number of parcels registered in this zone set |
| `current_root_version` | `u32` | Monotonic counter for root updates |
| `created_at` | `i64` | Zone set creation timestamp |
| `updated_at` | `i64` | Last modification timestamp |
| `bump` | `u8` | PDA bump seed |

### 5.2 OwnershipRoot (on-chain PDA)

**PDA seed:** `["ownership_root", zone_set_key]`

| Field | Type | Description |
|-------|------|-------------|
| `zone_set` | `Pubkey` | ZoneSet PDA key |
| `merkle_root` | `[u8; 32]` | Poseidon Merkle root of all owner commitments in the zone |
| `version` | `u32` | Monotonic counter, bumped on each root update |
| `commitment_count` | `u32` | Number of leaf commitments in the tree |
| `algorithm_id` | `u8` | Hash/proof system (0=Poseidon+Groth16, 1=PQ-STARK) |
| `snapshot_cid` | `String` | IPFS CID of the full commitment tree snapshot |
| `snapshot_hash` | `[u8; 32]` | SHA-256 of the snapshot bytes |
| `authority_signature` | `[u8; 64]` | Ed25519 signature of the root by the zone authority |
| `created_at` | `i64` | Root creation timestamp |
| `bump` | `u8` | PDA bump seed |

### 5.3 NullifierRecord (on-chain PDA)

**PDA seed:** `["nullifier", nullifier_hash]`

| Field | Type | Description |
|-------|------|-------------|
| `nullifier_hash` | `[u8; 32]` | Poseidon(owner_commitment, zone_root_version) |
| `zone_set` | `Pubkey` | ZoneSet PDA key |
| `root_version` | `u32` | Version of the Merkle root this nullifier is bound to |
| `prover` | `Pubkey` | Wallet that submitted the proof (transaction signer) |
| `proof_purpose` | `String` | Human-readable purpose (e.g., "subsidy_qualification") |
| `block_time` | `i64` | Solana block time when proof was verified |
| `bump` | `u8` | PDA bump seed |

### 5.4 ParcelCommitment (off-chain, committed to Merkle tree)

| Field | Type | Description |
|-------|------|-------------|
| `parcel_id` | `Pubkey` | Parcel PDA key (from AuthorityRegistry) |
| `owner` | `Pubkey` | Owner wallet or identity PDA |
| `value_commitment` | `[u8; 32]` | Pedersen commitment to parcel value |
| `leaf_hash` | `[u8; 32]` | Poseidon(owner, value_commitment) — inserted into Merkle tree |

### 5.5 OwnershipProofEvent (emitted as event)

| Field | Type | Description |
|-------|------|-------------|
| `nullifier_hash` | `[u8; 32]` | Nullifier (proves uniqueness) |
| `zone_set` | `Pubkey` | ZoneSet PDA key |
| `root_version` | `u32` | Merkle root version used in proof |
| `proof_purpose` | `String` | Human-readable purpose |
| `disclosure_type` | `u8` | 0=membership (at least 1), 1=range (value > X), 2=count (exactly N) |
| `prover` | `Pubkey` | Transaction signer (the prover's wallet) |
| `block_time` | `i64` | Solana block time |

## 6. Instructions

### 6.1 `register_zone_set`

**Purpose:** Register a new zone for ZK ownership proofs. Creates the zone set and initializes the first empty ownership root.

**Accounts:**
- `zone_set` (init, PDA `["zone_set", zone_id]`)
- `ownership_root` (init, PDA `["ownership_root", zone_set_key]`)
- `zone_id` (the zone's unique identifier account)
- `authority` (signer, mut — must be admin or zone authority in AuthorityRegistry)
- `system_program`

**Args:** `snapshot_cid: String`, `snapshot_hash: [u8; 32]`

**Guards:**
- `authority` must be the registry admin or the zone's authority in AuthorityRegistry
- `snapshot_hash != [0; 32]`
- `snapshot_cid` is non-empty
- No pending zone set already exists for this `zone_id`

**Effects:**
- `zone_set.authority = authority.key()`
- `zone_set.parcel_count = 0`
- `zone_set.current_root_version = 0`
- `zone_set.created_at = now`
- `ownership_root.merkle_root = POSEIDON_EMPTY_TREE_ROOT` (empty tree hash)
- `ownership_root.version = 0`
- `ownership_root.commitment_count = 0`
- `ownership_root.algorithm_id = 0`
- `ownership_root.snapshot_cid = snapshot_cid`
- `ownership_root.snapshot_hash = snapshot_hash`
- `ownership_root.created_at = now`

**Emits:** `ZoneSetRegistered { zone_id, authority, snapshot_cid, created_at }`

### 6.2 `generate_ownership_root`

**Purpose:** Update the Merkle root for a zone after parcels have been added, removed, or ownership has transferred. The authority commits a new root derived from the current set of owner commitments.

**Accounts:**
- `zone_set` (mut)
- `ownership_root` (mut — re-initialized with new root)
- `authority` (signer — must match zone_set.authority)
- `system_program`

**Args:** `new_merkle_root: [u8; 32]`, `new_snapshot_cid: String`, `new_snapshot_hash: [u8; 32]`, `commitment_count: u32`

**Guards:**
- `authority.key() == zone_set.authority`
- `new_merkle_root != [0; 32]`
- `new_snapshot_hash != [0; 32]`
- `new_snapshot_cid` is non-empty
- `commitment_count > 0` (cannot generate a root for an empty zone)
- `commitment_count` matches the number of leaves in the committed snapshot

**Effects:**
- `zone_set.current_root_version += 1`
- `zone_set.parcel_count = commitment_count`
- `zone_set.updated_at = now`
- `ownership_root.merkle_root = new_merkle_root`
- `ownership_root.version = zone_set.current_root_version`
- `ownership_root.commitment_count = commitment_count`
- `ownership_root.snapshot_cid = new_snapshot_cid`
- `ownership_root.snapshot_hash = new_snapshot_hash`
- `ownership_root.authority_signature = sign(new_merkle_root || zone_set.current_root_version)` by authority

**Emits:** `OwnershipRootUpdated { zone_set, new_merkle_root, version, commitment_count, block_time }`

### 6.3 `verify_ownership_proof`

**Purpose:** Verify a ZK proof of ownership and record the nullifier to prevent double-proving. This is the core instruction — it verifies the proof off-chain logic is enforced by the circuit constraints.

**Accounts:**
- `nullifier_record` (init, PDA `["nullifier", nullifier_hash]`)
- `ownership_root` (readonly)
- `zone_set` (readonly)
- `prover` (signer, mut — pays rent for nullifier record)
- `system_program`

**Args:** `proof_data: Vec<u8>`, `nullifier_hash: [u8; 32]`, `root_version: u32`, `proof_purpose: String`, `disclosure_type: u8`

**Guards:**
- `nullifier_hash != [0; 32]`
- `nullifier_record` does NOT already exist (first-use check — prevents double-proving)
- `root_version == ownership_root.version` (proof must reference the current root)
- `proof_data.len() <= MAX_PROOF_SIZE (1024 bytes)`
- `proof_purpose` is non-empty and <= 128 characters
- `disclosure_type` is 0, 1, or 2
- The ZK proof is valid against `ownership_root.merkle_root` (verified by the circuit)
- The nullifier in the proof matches `nullifier_hash`

**Effects:**
- `nullifier_record.nullifier_hash = nullifier_hash`
- `nullifier_record.zone_set = zone_set.key()`
- `nullifier_record.root_version = root_version`
- `nullifier_record.prover = prover.key()`
- `nullifier_record.proof_purpose = proof_purpose`
- `nullifier_record.block_time = now`

**Emits:** `OwnershipProofVerified { nullifier_hash, zone_set, root_version, proof_purpose, disclosure_type, prover, block_time }`

### 6.4 `invalidate_proof`

**Purpose:** Invalidate all proofs tied to a specific Merkle root version when ownership transfers or the root is updated. This is an administrative action — it marks the old root version as stale so proofs generated against it are rejected.

**Accounts:**
- `zone_set` (mut)
- `authority` (signer — must match zone_set.authority)
- `system_program`

**Args:** `stale_version: u32`

**Guards:**
- `authority.key() == zone_set.authority`
- `stale_version < zone_set.current_root_version` (cannot invalidate the current version)
- `stale_version > 0` (cannot invalidate version 0 / the initial empty state)

**Effects:**
- The `stale_version` is recorded in an emitted event; validators and applications must stop accepting proofs with this version
- On-chain: `zone_set.updated_at = now`
- The nullifier records for the stale version remain on-chain (they still prevent replay of old proofs)
- Proofs against the stale version are now rejected by `verify_ownership_proof` (root_version check)

**Emits:** `ProofVersionInvalidated { zone_set, stale_version, current_version, block_time }`

## 7. Off-Chain Protocol

### 7.1 Zone Registration Ceremony

1. Zone authority (e.g., a government land registry) identifies the set of parcels in their zone.
2. For each parcel, compute: `leaf = Poseidon(owner_pubkey, Pedersen(parcel_value))`.
3. Build a Poseidon Merkle tree from all leaves. Record the root.
4. Upload the full commitment tree snapshot to IPFS → receive CID.
5. Authority calls `register_zone_set` on-chain with the snapshot CID and hash.
6. On-chain record now points to the zone; the root is the empty state.
7. Authority calls `generate_ownership_root` to commit the first populated root.

### 7.2 Proof Generation (Client-Side)

1. The prover obtains the current Merkle root and snapshot CID from the on-chain record.
2. The prover downloads the snapshot from IPFS and locates their leaf commitment.
3. The prover constructs a Merkle proof (20 hashes) showing their leaf is in the tree.
4. The prover generates a ZK proof:
   - **Membership proof (disclosure_type=0):** "I know a leaf in this Merkle tree such that Poseidon(owner, value_commitment) = leaf, and I know the opening of the value_commitment."
   - **Range proof (disclosure_type=1):** "I know a leaf such that the committed value > threshold, without revealing the value."
   - **Count proof (disclosure_type=2):** "I know exactly N leaves in the tree that I can open." (Requires a different circuit or accumulator.)
5. The prover computes `nullifier = Poseidon(owner_commitment, root_version)`.
6. The prover calls `verify_ownership_proof` on-chain with the proof data, nullifier, root version, purpose, and disclosure type.

### 7.3 Ownership Transfer & Revocation

1. When a parcel is sold or transferred (via AuthorityRegistry), the owner changes.
2. The zone authority detects the transfer (event listener or polling).
3. The authority recomputes the Merkle tree with the new owner's leaf.
4. The authority calls `generate_ownership_root` with the updated root.
5. The old root version is invalidated via `invalidate_proof`.
6. All proofs generated against the old root version are now rejected.
7. The new owner must generate fresh proofs against the new root.

### 7.4 Proof Verification by Third Parties

1. A third party (government agency, bank, association) requests a ZK proof from a prover.
2. The prover generates and submits the proof on-chain.
3. The third party reads the `OwnershipProofVerified` event from on-chain data.
4. The third party verifies:
   - The nullifier hash matches the event.
   - The root version is current (not invalidated).
   - The proof purpose matches the requested use case.
5. If valid, the third party accepts the proof. No further interaction with Terra is needed.

### 7.5 Snapshot Management

- Snapshots are stored on IPFS (content-addressed, immutable).
- Each `generate_ownership_root` call uploads a new snapshot with the updated tree.
- Old snapshots remain on IPFS (content-addressed — old CIDs still resolve).
- Third-party verifiers can download any snapshot and independently reconstruct the Merkle root to verify historical proofs.

### 7.6 Selective Disclosure Semantics

| disclosure_type | What is proved | What is hidden |
|-----------------|----------------|----------------|
| 0 (membership) | "I own at least 1 parcel in this zone" | Which parcel, how many, who I am |
| 1 (range) | "I own parcels worth > $X in aggregate" | Exact value, which parcels |
| 2 (count) | "I own exactly N parcels" | Which parcels, total value |

## 8. Collusion Resistance

### 8.1 Nullifier Prevents Double-Counting

Each proof generates a unique nullifier = Poseidon(owner_commitment, root_version). Once a nullifier is recorded on-chain, the same proof cannot be submitted again. This prevents:
- A single owner from voting twice in a landowners' association.
- A borrower from inflating their apparent holdings by submitting multiple proofs.
- A prover from sharing their proof with non-owners.

### 8.2 Root Versioning Prevents Stale Proofs

When ownership transfers, the Merkle root changes. Old proofs are rejected because `root_version != ownership_root.version`. This prevents:
- A former owner from using a proof generated before the transfer.
- A prover from using a snapshot that no longer reflects reality.

### 8.3 Authority Signature on Root

The Merkle root is signed by the zone authority (Ed25519). This prevents:
- A malicious prover from forging a Merkle root that includes fabricated ownership.
- A third party from accepting a root that was never authorized.

### 8.4 Pedersen Commitment Hiding

Pedersen commitments are statistically hiding — the committed value cannot be recovered even with unlimited computational power (given the discrete log assumption). This prevents:
- An observer from learning parcel values from the on-chain commitment data.
- A prover from revealing their exact holdings through the commitment scheme.

### 8.5 Cross-Zone Isolation

Each zone has its own Merkle root and nullifier namespace. A proof valid in Zone A cannot be used in Zone B, even if the same owner holds parcels in both zones. This prevents:
- A prover from aggregating holdings across zones to meet a threshold they don't meet in any single zone.

## 9. Liveness Guarantees

### 9.1 Root Update Authority

The zone authority is responsible for calling `generate_ownership_root` when ownership changes. If the authority is unresponsive:
- Proofs continue to work against the current root (ownership has not changed on-chain).
- If ownership actually transferred but the root wasn't updated, the new owner cannot generate a valid proof until the root is updated.
- Mitigation: AuthorityRegistry has governance mechanisms for replacing unresponsive authorities.

### 9.2 Proof Availability Window

Proofs are valid as long as:
- The root version is current (not invalidated).
- The nullifier has not been used.

If the authority invalidates a root version (e.g., due to a suspected compromise), all proofs against that version are rejected. This is a feature, not a bug — it prevents stale proofs from being accepted.

### 9.3 Snapshot Accessibility

Snapshots are stored on IPFS. If a pinning service goes down, the snapshot may become temporarily unavailable. Mitigation:
- Multiple pinning services (IPFS pin redundancy).
- Arweave permanent backup (optional but recommended for production).
- Third parties can cache snapshots locally after initial download.

### 9.4 Prover Liveness

Proofs can be generated at any time as long as:
- The prover has access to the current Merkle root and their leaf commitment.
- The prover has a ZK proving key (distributed during zone registration).
- The on-chain program is operational.

## 10. Storage Architecture

### 10.1 IPFS

- Content-addressed by design. CID is the integrity check.
- Commitment tree snapshots are stored here.
- Anyone can pin the snapshot without trusting them (the data is just Merkle leaves).
- Mutable pins via pinning service (web3.storage, Pinata, etc.)
- Gateway fallback: ipfs.io, cloudflare-ipfs.com, etc.

### 10.2 Arweave

- Permanent, pay-once storage.
- Appropriate for zone snapshots that must outlive any single pinning service.
- Cost is negligible for commitment trees (KB to low MB per zone).

### 10.3 On-Chain (Solana)

- Only identifiers, hashes, thresholds, and state transitions go on-chain:
  - Merkle root (32 bytes)
  - Nullifier record (~128 bytes per proof)
  - Zone set metadata (~64 bytes)
- Proof data itself is never stored on-chain — only the nullifier hash.

### 10.4 PostGIS (Off-Chain Cache)

- Local cache + indexing for the API layer.
- Reconstructable from IPFS snapshots + on-chain hashes.
- Not the source of truth — only a performance optimization.

### 10.5 CID Verification Protocol

1. Fetch the snapshot from any IPFS gateway using the CID.
2. Compute SHA-256 of the fetched bytes.
3. Compare against `ownership_root.snapshot_hash` (on-chain).
4. If they match, the snapshot is authentic and untampered.
5. Anyone can perform this verification — no trust in the API required.

## 11. Replay / Nonce Hygiene

### 11.1 Nullifier Uniqueness

The nullifier hash is the primary replay-prevention mechanism. It is derived from:
- The owner's commitment (unique per owner per zone).
- The Merkle root version (changes on ownership transfer).

Once a nullifier is recorded on-chain, the same proof cannot be submitted again.

### 11.2 Root Version Binding

Every proof must specify the `root_version` it was generated against. The program checks:
- `root_version == ownership_root.version` (must be current).
- If the root version has been invalidated, the proof is rejected.

This prevents replay of proofs generated against older, stale roots.

### 11.3 Purpose Binding

Each proof includes a `proof_purpose` string. While not a cryptographic nonce, it provides:
- Audit trail: what was the proof used for?
- Application-layer enforcement: a "subsidy_qualification" proof cannot be reused for a "vote" if the application checks the purpose field.
- The purpose is signed by the prover (part of the ZK circuit inputs).

### 11.4 Transaction Uniqueness

Solana transactions have unique signatures. Even if the same proof data were somehow resubmitted, the transaction would be a different Solana transaction with a different signature. However, the nullifier check is the authoritative replay prevention — the transaction uniqueness is a secondary defense.

## 12. Post-Quantum Migration Path

### 12.1 Algorithm ID in OwnershipRoot

The `algorithm_id` field is a `u8` enum:
- `0` = Poseidon hash + Groth16 SNARK (current)
- `1` = PQ-secure hash (e.g., SPHINCS+ or XMSS) + STARK proof (future)

### 12.2 Hybrid Transition During Root Update

When a post-quantum algorithm is standardized and ready for use:
1. During a normal `generate_ownership_root` call, the authority re-commits the tree under the new hash function.
2. The new root is uploaded to IPFS with a new CID.
3. `generate_ownership_root` updates the on-chain record with `algorithm_id = 1`.
4. Old proofs (generated under Poseidon+Groth16) are rejected because the root version changed.
5. New proofs must use the PQ-secure hash and STARK proof system.

### 12.3 Timeline Considerations

- NIST post-quantum standards are finalized (2024).
- STARK proof systems are mature and deployed in production (e.g., StarkNet).
- For the pilot (1-2 years), Poseidon+Groth16 is sufficient.
- Migration to PQ-secure hash should happen before the system scales beyond the pilot region.
- The nullifier system is hash-based and can be migrated by simply changing the hash function — no structural changes to the protocol.

## 13. Operational Security

### 13.1 Prover Key Management

- The ZK proving key must be kept secret. If compromised, an attacker could generate valid proofs for any owner in the zone.
- Mitigation: Proving keys are generated per-zone and distributed to authorized provers only.
- Key rotation: When the authority calls `generate_ownership_root`, the proving key can be rotated (new key pair for the new root version).

### 13.2 Authority Key Security

- The zone authority's Ed25519 key signs Merkle roots. If compromised, an attacker could forge roots.
- Mitigation: Authority key is managed via AuthorityRegistry governance. Key rotation is supported.
- Multi-sig: The authority can be a multi-sig wallet for high-security zones.

### 13.3 Snapshot Integrity

- Snapshots on IPFS are content-addressed — tampering changes the CID.
- The on-chain `snapshot_hash` is the authoritative integrity check.
- Third parties should always verify the CID against the on-chain hash before trusting a snapshot.

### 13.4 Proof Submission Security

- Provers should generate proofs locally (on their own device) and submit them via their own wallet.
- A relay or custodial service that submits proofs on behalf of users introduces a trust assumption.
- Mitigation: The proof is valid regardless of who submits it — the nullifier prevents misuse even if the submitter is untrusted.

### 13.5 Validator Onboarding / Offboarding

- **Onboarding:** New zone authority receives the proving key and snapshot via a secure channel. Their pubkey is registered in AuthorityRegistry.
- **Offboarding:** The departing authority's key is revoked via AuthorityRegistry governance. A new authority is appointed. The root is updated under the new authority's signature.

## 14. Test Vectors

### 14.1 Zone Registration + First Root

- Zone ID: random Pubkey
- Authority: wallet A
- 5 parcels: owners [B, C, D, E, F], values [100, 250, 500, 1000, 750]
- Leaf hashes: Poseidon(owner_i, Pedersen(value_i)) for each
- Merkle root: Poseidon-based tree of 5 leaves
- Snapshot: uploaded to IPFS → CID "QmZoneTest123..."
- Expected: ZoneSet created with parcel_count=5, OwnershipRoot created with version=1, correct Merkle root

### 14.2 Membership Proof (Disclosure Type 0)

- Zone: 5 parcels as in 14.1
- Prover: wallet C (owns parcel worth 250)
- Proof: ZK proof that Poseidon(C, Pedersen(250)) is a leaf in the Merkle tree
- Nullifier: Poseidon(Pedersen(C), root_version=1)
- Expected: NullifierRecord created, OwnershipProofVerified event emitted with disclosure_type=0

### 14.3 Range Proof (Disclosure Type 1)

- Zone: 5 parcels as in 14.1
- Prover: wallet E (owns parcel worth 1000)
- Threshold: $500
- Proof: ZK proof that committed value > 500 for at least one leaf owned by prover
- Nullifier: Poseidon(Pedersen(E), root_version=1)
- Expected: NullifierRecord created, OwnershipProofVerified event emitted with disclosure_type=1

### 14.4 Double-Proof Prevention

- Zone: 5 parcels as in 14.1
- Prover: wallet C submits proof (nullifier N)
- NullifierRecord for N is created
- Prover: wallet C tries to submit another proof with the same nullifier N
- Expected: `NullifierAlreadyUsed` error

### 14.5 Ownership Transfer + Root Update

- Zone: 5 parcels as in 14.1
- Parcel originally owned by C is transferred to G
- Authority recomputes Merkle tree with new owner G
- Authority calls `generate_ownership_root` with new root (version=2)
- Authority calls `invalidate_proof` with stale_version=1
- Prover: wallet C tries to submit a proof with root_version=1
- Expected: `RootVersionStale` error (proof rejected because version 1 is invalidated)

### 14.6 Stale Root Rejection

- Zone: root_version=2 is current
- Prover: submits proof with root_version=1
- Expected: `RootVersionMismatch` error (proof references a non-current root)

### 14.7 Authority Mismatch

- Zone: authority is wallet A
- Signer: wallet B tries to call `generate_ownership_root`
- Expected: `UnauthorizedAuthority` error

### 14.8 Empty Zone Registration

- Zone: no parcels registered yet
- Authority tries to call `generate_ownership_root` with commitment_count=0
- Expected: `EmptyZoneSet` error (cannot generate a root for an empty zone)

### 14.9 Proof Purpose Validation

- Zone: valid proof data
- Prover: submits proof with empty proof_purpose string
- Expected: `InvalidProofPurpose` error

### 14.10 Collusion Scenario — Cross-Zone

- Zone A: prover owns 1 parcel worth $200
- Zone B: prover owns 1 parcel worth $200
- Prover tries to combine proofs from both zones to claim $400 aggregate
- Expected: Invalid — each zone has its own root and nullifier namespace; cross-zone aggregation is not possible with this protocol

### 14.11 Proof Version Invalidation Edge Cases

- Zone: root_version=3 is current, versions 1 and 2 are already invalidated
- Authority tries to invalidate version=2 again
- Expected: `VersionAlreadyInvalidated` error (or no-op, depending on implementation choice)

- Authority tries to invalidate version=3 (the current version)
- Expected: `CannotInvalidateCurrentVersion` error

- Authority tries to invalidate version=0
- Expected: `CannotInvalidateGenesisVersion` error
