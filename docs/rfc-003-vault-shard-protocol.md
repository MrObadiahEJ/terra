# RFC-003: Vault Shard Protocol

## 1. Status

- **Status:** Draft
- **Created:** 2026-09-01
- **Supersedes:** None

## 2. Summary

This RFC specifies the **Vault Shard Protocol** — a threshold-encrypted personal-data protection layer for Terra. Sensitive personal data (biometric templates, national ID scans, photographs, court documents) is encrypted client-side, split into Shamir shards distributed across `n` validators, and stored on content-addressed decentralized storage (IPFS/Arweave). The on-chain program never sees plaintext, keys, or shards — it only logs authorization events and manages vault metadata. Reconstruction requires `k` validators to coordinate off-chain, reconstruct the AES key, and decrypt locally.

## 3. Threat Model

### 3.1 Adversary Classes

| Class | Description | Mitigation |
|-------|-------------|------------|
| **Rogue validator** | A single validator attempts to access vault data alone | Threshold encryption (k-of-n required) |
| **Colluding subset** | k validators collude to reconstruct without authorization | Authorization event logging, anomaly detection, guardian co-sign |
| **Platform operator** | The API server or infrastructure provider | Ciphertext is encrypted before upload; operator never sees keys |
| **External attacker** | Hacks, subpoenas, or social engineers | Content-addressed storage, encrypted at rest, no single point of failure |
| **Future adversary** | Harvests ciphertext today, waits for cryptographic break | Post-quantum migration path via algorithm_id + shard rotation |

### 3.2 In Scope

- Confidential personal data protection (biometrics, national IDs, photos)
- Shard distribution, reconstruction, and rotation ceremonies
- On-chain authorization logging (who authorized what, when)
- Decentralized ciphertext storage (IPFS/Arweave)
- Liveness guarantees (ping protocol, auto-rotation on missed pings)

### 3.3 Out of Scope

- Attestation, succession, and forfeiture flows (covered by RFC-001/002)
- Validator identity verification (covered by AuthorityRegistry)
- The specific encryption of biometric templates (application-layer concern)

## 4. Cryptographic Choices

### 4.1 Symmetric Encryption: AES-256-GCM

- 256-bit key, 96-bit nonce, 128-bit authentication tag
- Key is randomly generated per vault (never derived from validator keys)
- Nonce is random per encryption operation (never reused)

### 4.2 Key Derivation: HKDF-SHA256

- Used to derive the AES key from a high-entropy source (hardware RNG or OS CSPRNG)
- The derived key is split via Shamir's Secret Sharing

### 4.3 Secret Sharing: Shamir's Secret Sharing

- Over GF(2^8) with polynomial degree `k-1`
- Parameters: `n` total shards, `k` threshold for reconstruction
- Each shard is a 32-byte point on the polynomial
- Shards are distributed to validator wallets (pubkey-encrypted if transmission channel allows)

### 4.4 Hashing: SHA-256

- Content addressing (IPFS CIDs are SHA-256-based)
- On-chain ciphertext hash for integrity verification
- Vault record hash for version binding

### 4.5 Signatures: Ed25519

- Solana native — all validator keys are Ed25519
- Used for transaction signing (on-chain authorization)
- Used for off-chain protocol messages (validator coordination)

### 4.6 Post-Quantum Migration: Algorithm ID

- `u8` enum in VaultRecord: `0 = AES-256-GCM`, `1 = Kyber-768 + AES-256-GCM hybrid`
- Only variant 0 is implemented for the pilot
- The `rotate_vault_shards` flow can re-encrypt under a new algorithm without program upgrade

## 5. Data Model

### 5.1 VaultRecord (on-chain PDA)

**PDA seed:** `["vault_record", subject]`

| Field | Type | Description |
|-------|------|-------------|
| `subject` | `Pubkey` | Identity PDA key (the person whose data is protected) |
| `ciphertext_cid` | `String` | IPFS CID v1 of the encrypted dossier |
| `ciphertext_hash` | `[u8; 32]` | SHA-256 of the ciphertext bytes |
| `algorithm_id` | `u8` | Encryption algorithm (0=AES-256-GCM) |
| `storage_uris` | `Vec<String>` | Fallback gateways (max 4) |
| `shard_holders` | `Vec<Pubkey>` | n validators holding shards (max 8) |
| `threshold` | `u8` | k — minimum shards for reconstruction |
| `version` | `u32` | Monotonic counter, bumped on rotation |
| `last_ping_at` | `i64` | Last shard health check timestamp |
| `created_at` | `i64` | Vault creation timestamp |

### 5.2 VaultShardRotation (on-chain PDA)

**PDA seed:** `["vault_shard_rotation", vault, new_ciphertext_hash]`

| Field | Type | Description |
|-------|------|-------------|
| `vault` | `Pubkey` | VaultRecord key |
| `old_ciphertext_hash` | `[u8; 32]` | Current ciphertext hash (for verification) |
| `new_ciphertext_hash` | `[u8; 32]` | New ciphertext hash (commitment before execution) |
| `new_shard_holders` | `Vec<Pubkey>` | New validator set (may include new validators) |
| `new_threshold` | `u8` | New k value |
| `initiated_by` | `Pubkey` | Validator who initiated the rotation |
| `endorsements` | `Vec<Pubkey>` | Current validators who endorsed |
| `required_endorsements` | `u8` | ceil(2n/3) of current validators |
| `initiated_at` | `i64` | When rotation was initiated |
| `effective_at` | `i64` | initiated_at + ROTATION_TIMELOCK_SECS (7 days) |
| `status` | `u8` | 0=Pending, 1=Executed, 2=Cancelled |

### 5.3 VaultAccessLog (emitted as event)

| Field | Type | Description |
|-------|------|-------------|
| `subject` | `Pubkey` | Identity PDA key |
| `vault` | `Pubkey` | VaultRecord key |
| `purpose` | `String` | Human-readable purpose (e.g., "recovery_request_#42") |
| `validators` | `Vec<Pubkey>` | Validators who authorized access |
| `off_chain_nonce` | `[u8; 32]` | Nonce generated off-chain by the validator group |
| `expiry` | `i64` | Authorization expires at this timestamp |
| `block_time` | `i64` | Solana block time |

### 5.4 Ciphertext Storage

- **IPFS:** Content-addressed, mutable pins via pinning service. CID v1.
- **Arweave:** Permanent, pay-once mirror. Optional for pilot, required for production.
- **PostGIS:** Local cache + indexing. Reconstructable from IPFS/Arweave + on-chain hashes.

## 6. Instructions

### 6.1 `create_vault`

**Purpose:** Create a vault record for a subject (identity).

**Accounts:**
- `vault_record` (init, PDA `["vault_record", subject]`)
- `subject` (Identity account)
- `authority` (signer, mut — pays rent; must be admin or subject's recovery wallet)
- `system_program`

**Args:** `ciphertext_cid: String`, `ciphertext_hash: [u8; 32]`, `algorithm_id: u8`, `storage_uris: Vec<String>`, `shard_holders: Vec<Pubkey>`, `threshold: u8`

**Guards:**
- `authority` must be the registry admin or the subject's recovery wallet
- `shard_holders.len() <= MAX_SHARD_HOLDERS (8)`
- `threshold <= shard_holders.len()`
- `threshold >= 2` (minimum for any vault)
- `ciphertext_hash != [0; 32]`
- `ciphertext_cid` is non-empty
- `algorithm_id` is supported (currently only 0)

**Emits:** `VaultCreated`

### 6.2 `authorize_vault_access`

**Purpose:** Log that k validators have collectively authorized access to a vault. The actual reconstruction happens off-chain.

**Accounts:**
- `vault_record` (mut)
- `subject` (Identity account)
- `authority` (signer, mut — one of the k validators)
- `system_program`

**Args:** `purpose: String`, `expiry: i64`, `off_chain_nonce: [u8; 32]`

**Guards:**
- `authority` is in `vault_record.shard_holders`
- At least `threshold` signers in `remaining_accounts` are in `vault_record.shard_holders`
- `expiry` is within 24 hours of now
- `off_chain_nonce` is non-zero
- Same nonce not used for a previous authorization on this vault

**Emits:** `VaultAccessAuthorized`

### 6.3 `initiate_shard_rotation`

**Purpose:** Begin a shard rotation ceremony. Time-locked for 7 days.

**Accounts:**
- `rotation` (init, PDA `["vault_shard_rotation", vault, new_ciphertext_hash]`)
- `vault_record` (mut)
- `initiator` (signer, mut — active validator for this vault)
- `system_program`

**Args:** `new_ciphertext_hash: [u8; 32]`, `new_shard_holders: Vec<Pubkey>`, `new_threshold: u8`

**Guards:**
- `initiator` is in `vault_record.shard_holders`
- `new_shard_holders.len() <= MAX_SHARD_HOLDERS`
- `new_threshold <= new_shard_holders.len()`
- `new_ciphertext_hash != [0; 32]`
- No pending rotation already exists for this vault

**Emits:** `ShardRotationInitiated`

### 6.4 `endorse_shard_rotation`

**Purpose:** A current validator endorses the proposed rotation.

**Accounts:**
- `rotation` (mut)
- `vault_record` (readonly)
- `validator` (signer — active validator for this vault)
- `system_program`

**Args:** None

**Guards:**
- `rotation.status == Pending`
- `validator` is in `vault_record.shard_holders`
- `validator` is not already in `rotation.endorsements`
- `validator` is not the `rotation.initiated_by` (self-endorsement prevention)
- `now < rotation.effective_at` (still within time lock)

**Emits:** `RotationEndorsed`

### 6.5 `execute_shard_rotation`

**Purpose:** Finalize the rotation after time lock expires and quorum is met. Off-chain re-encryption must happen BEFORE this call.

**Accounts:**
- `rotation` (mut, close → initiator)
- `vault_record` (mut)
- `initiator` (signer, mut — receives closed account lamports)
- `system_program`

**Args:** None

**Guards:**
- `rotation.status == Pending`
- `now >= rotation.effective_at` (time lock expired)
- `rotation.endorsements.len() >= rotation.required_endorsements` (quorum met)
- `rotation.new_ciphertext_hash == vault_record.ciphertext_hash` is NOT required — the new hash is committed at initiation and verified by the endorsing validators

**Effects:**
- `vault_record.ciphertext_hash = rotation.new_ciphertext_hash`
- `vault_record.shard_holders = rotation.new_shard_holders`
- `vault_record.threshold = rotation.new_threshold`
- `vault_record.version += 1`
- `vault_record.last_ping_at = now`
- Close `rotation` account

**Emits:** `ShardRotationExecuted`

### 6.6 `cancel_shard_rotation`

**Purpose:** Cancel a pending rotation (e.g., if the ceremony is abandoned).

**Accounts:**
- `rotation` (mut, close → canceller)
- `vault_record` (readonly)
- `canceller` (signer, mut — admin or the initiator)
- `system_program`

**Args:** None

**Guards:**
- `rotation.status == Pending`
- `canceller` is either the registry admin or `rotation.initiated_by`

**Effects:** Close `rotation` account.

**Emits:** `ShardRotationCancelled`

### 6.7 `ping_shard`

**Purpose:** Validators periodically confirm they still hold their shard and can participate in reconstruction.

**Accounts:**
- `vault_record` (mut)
- `validator` (signer — one of the shard holders)
- `system_program`

**Args:** None

**Guards:**
- `validator` is in `vault_record.shard_holders`
- `now >= vault_record.last_ping_at + PING_INTERVAL_SECS`

**Effects:**
- `vault_record.last_ping_at = now`

**Emits:** `ShardPinged`

## 7. Off-Chain Protocol

### 7.1 Vault Creation Ceremony

1. Subject (or their guardian) generates a high-entropy AES-256 key using a hardware RNG or OS CSPRNG.
2. Subject encrypts their personal data dossier under AES-256-GCM with the generated key.
3. Subject uploads the ciphertext to IPFS → receives CID.
4. Subject splits the AES key into `n` Shamir shards with threshold `k`.
5. Subject distributes each shard to a designated validator via a secure channel (in-person handoff, encrypted USB, QR code on paper).
6. Subject calls `create_vault` on-chain with the CID, hash, shard holders, and threshold.
7. On-chain record now points to the ciphertext; reconstruction requires `k` validators.

### 7.2 Normal Reconstruction Ceremony

1. A requestor (subject, guardian, or authorized party) identifies the need to access the vault.
2. The requestor coordinates with `k` validators via a secure side-channel (Signal group, physical meeting).
3. Each validator confirms their identity and willingness to participate.
4. The validators generate a shared `off_chain_nonce` (random 32 bytes, agreed upon by all participants).
5. Each validator submits `authorize_vault_access` on-chain with the purpose, expiry, and nonce. The first signer pays; the others sign as co-signers via `remaining_accounts`.
6. On-chain program verifies quorum and emits `VaultAccessAuthorized`.
7. Off-chain — AFTER the on-chain authorization — the `k` validators meet (physically or via encrypted channel).
8. Each validator inputs their shard into a local, air-gapped reconstruction device (laptop in a closed room).
9. The device reconstructs the AES key using Lagrange interpolation.
10. The device decrypts the dossier using the reconstructed key.
11. The plaintext is displayed to the authorized parties. **It never leaves the device.**
12. The validators securely destroy any copies of the reconstructed key.

### 7.3 Shard Rotation Ceremony

1. A validator notices a shard holder is unavailable (phone lost, leaving the pilot, etc.).
2. The initiator calls `initiate_shard_rotation` on-chain, committing the new ciphertext hash and new shard holder set.
3. The time lock begins (7 days). During this window, current validators endorse the rotation.
4. Off-chain — BEFORE the time lock expires — the endorsing validators meet.
5. They use their current shards to reconstruct the AES key.
6. They generate a new AES key (or the same key if only shard holders are changing).
7. They re-encrypt the dossier under the new key (if changed).
8. They upload the new ciphertext to IPFS → receive new CID + hash.
9. They split the new key into new shards and distribute to the new shard holders.
10. After the time lock expires, anyone calls `execute_shard_rotation`.
11. On-chain record updates: new hash, new shard holders, new threshold, version bumped.
12. Old shards are orphaned (content-addressed — old CID still resolves but is no longer the canonical version).

### 7.4 Emergency Recovery Ceremony

Emergency recovery is used when:
- The subject is deceased or permanently unavailable
- More than `n - k` validators have lost their shards
- A court order requires access to the vault

**Threshold:** `ceil(3n/4)` validators + court order hash (or guardian co-sign).

1. The requestor obtains a court order (or guardian co-signs).
2. The requestor calls `authorize_vault_access` with purpose "emergency_recovery" and the court order hash as part of the purpose string.
3. `ceil(3n/4)` validators sign the authorization.
4. The reconstruction ceremony proceeds as in 7.2.

### 7.5 Side-Channel Requirements

- **Signal group:** Each vault has a dedicated Signal group with the `k` shard holders. Messages are end-to-end encrypted.
- **Physical meetings:** For high-security operations (rotation, emergency recovery), validators meet in person.
- **Air-gapped device:** The reconstruction device must never have been connected to the internet. A dedicated laptop running a minimal OS (Tails, Qubes) is recommended.
- **Shard transfer:** QR code on paper (for initial distribution), encrypted USB (for rotation), or in-person handoff.

### 7.6 Reconstruction Device Procedure

1. Boot the air-gapped device from a live USB (Tails or similar).
2. Open the reconstruction application (a simple Python/CLI tool).
3. Each validator inputs their shard (via QR scan or manual entry).
4. The device reconstructs the AES key and displays a hash of the key for verification.
5. The device fetches the ciphertext from IPFS (via a USB-transferred copy of the CID).
6. The device decrypts and displays the plaintext.
7. After viewing, the device is powered off without saving any state.
8. The reconstructed key exists only in RAM and is lost on power-off.

## 8. Collusion Resistance

### 8.1 Rotation Quorum > Recovery Quorum

- **Normal reconstruction:** k-of-n validators (e.g., 2-of-5)
- **Shard rotation:** ceil(2n/3) endorsements (e.g., 4-of-5)
- **Emergency recovery:** ceil(3n/4) validators + court order (e.g., 4-of-5)

This means a colluding subset of k validators can reconstruct the key but cannot unilaterally rotate the shards to lock out honest validators.

### 8.2 Observer Audit Log

Every `authorize_vault_access` and `execute_shard_rotation` event is emitted on-chain with:
- The list of participating validators
- The off_chain_nonce (for correlation)
- The block timestamp

This creates a public, immutable audit trail. Any observer can detect:
- Unusual frequency of access authorizations
- The same group of validators always authorizing access
- Rotation ceremonies that happen too quickly

### 8.3 Guardian Co-Sign

For emergency recovery, the subject's designated guardian must co-sign. The guardian is:
- Declared at vault creation time (optional)
- Stored in the Identity account or a separate Guardian record
- Required only for emergency recovery (not normal access)

### 8.4 Time-Locked Rotation

Shard rotation has a mandatory 7-day time lock. This gives:
- Honest validators time to detect and cancel a malicious rotation
- The subject (or guardian) time to challenge the rotation
- An audit window for anomaly detection systems

## 9. Liveness Guarantees

### 9.1 Ping Protocol

Validators call `ping_shard` weekly to confirm they still hold their shard and can participate in reconstruction.

### 9.2 Auto-Rotation on Missed Pings

If a validator misses 3 consecutive pings (3 weeks), the system auto-initiates a shard rotation:
- Any validator can trigger this by calling `initiate_shard_rotation` with the same parameters
- The rotation replaces the non-responsive validator with a new one
- The time lock ensures the non-responsive validator has a window to object

### 9.3 Emergency Recovery Path

If fewer than `k` validators are available (e.g., natural disaster, political instability), emergency recovery requires:
- `ceil(3n/4)` validators + court order hash
- Or: subject's guardian + `ceil(n/2)` validators

### 9.4 Offline Validator Handling

If a validator is temporarily offline (no internet), they can:
- Catch up on missed pings when they reconnect (within the 3-week window)
- Participate in reconstruction via a delayed ceremony (validators can wait for the offline validator to come back online)
- Be replaced via rotation if they remain offline for >3 weeks

## 10. Storage Architecture

### 10.1 IPFS

- Content-addressed by design. CID is the integrity check.
- Anyone can pin the encrypted blob without trusting them (ciphertext is useless without shards).
- Mutable pins via pinning service (web3.storage, Pinata, etc.)
- Gateway fallback: ipfs.io, cloudflare-ipfs.com, etc.

### 10.2 Arweave

- Permanent, pay-once storage.
- Appropriate for land registry data that must outlive any single server.
- Cost is negligible for small dossiers (KB to low MB).

### 10.3 PostGIS

- Local cache + indexing for the API.
- Reconstructable from IPFS + on-chain hashes.
- Not the source of truth — only a performance optimization.

### 10.4 CID Verification Protocol

1. Fetch ciphertext from any IPFS gateway using the CID.
2. Compute SHA-256 of the fetched bytes.
3. Compare against `vault_record.ciphertext_hash` (on-chain).
4. If they match, the ciphertext is authentic and untampered.
5. Anyone can perform this verification — no trust in the API required.

## 11. Replay / Nonce Hygiene

### 11.1 Version Matching

The `vault_record.version` is bumped on every rotation. Validators must verify that the version they see matches the version they expect before participating in a reconstruction or rotation.

### 11.2 Nonce Uniqueness

The `off_chain_nonce` must be unique per authorization event. The program checks that the same nonce has not been used for a previous `VaultAccessAuthorized` event on the same vault.

### 11.3 Expiry Enforcement

The `expiry` field in `authorize_vault_access` must be within 24 hours of the current block time. This prevents stale authorizations from being replayed.

## 12. Post-Quantum Migration Path

### 12.1 Algorithm ID in VaultRecord

The `algorithm_id` field is a `u8` enum:
- `0` = AES-256-GCM (current)
- `1` = Kyber-768 + AES-256-GCM hybrid (future)

### 12.2 Hybrid Re-Encryption During Rotation

When a post-quantum algorithm is standardized and ready for use:
1. During a normal shard rotation ceremony, validators re-encrypt the dossier under the hybrid scheme.
2. The new ciphertext is uploaded to IPFS with a new CID.
3. `execute_shard_rotation` updates the on-chain record with the new hash and `algorithm_id = 1`.
4. Old shards (encrypted under AES-256-GCM only) are orphaned.

### 12.3 Timeline Considerations

- NIST post-quantum standards are finalized (2024).
- Kyber-768 is mature and deployed in production systems (Signal, Cloudflare).
- For the pilot (1-2 years), AES-256-GCM is sufficient.
- Migration to hybrid encryption should happen before the system scales beyond the pilot region.

## 13. Operational Security

### 13.1 Reconstruction Device Procedure

1. Use a dedicated laptop that has never been connected to the internet.
2. Boot from a live USB (Tails OS).
3. The reconstruction application is pre-installed on the USB.
4. After the ceremony, power off the device without saving state.
5. The USB is stored in a secure location between ceremonies.

### 13.2 Shard Transfer Protocol

- **Initial distribution:** QR code on paper, hand-delivered to each validator.
- **Rotation:** Encrypted USB drive, hand-delivered. The encryption key is shared verbally.
- **Emergency:** If physical handoff is impossible, use a secure messaging channel (Signal) with disappearing messages.

### 13.3 Device Seizure Response

If the reconstruction device is seized:
- The device contains no persistent data (live USB, no saved state).
- The reconstructed key exists only in RAM and is lost on power-off.
- The ciphertext on IPFS is still encrypted and useless without the shards.
- The shards are held by the validators, not on the device.

### 13.4 Validator Onboarding / Offboarding

- **Onboarding:** New validator receives a shard via in-person handoff. Their pubkey is added to the vault record via `rotate_vault_shards`.
- **Offboarding:** The departing validator's shard is invalidated via rotation. The remaining validators reconstruct, re-encrypt, and redistribute.

## 14. Test Vectors

### 14.1 Vault Creation + Shard Distribution

- Subject: wallet A
- n=5 validators: wallets B, C, D, E, F
- k=2 threshold
- Ciphertext: 1KB random bytes → CID "QmTest123..."
- Hash: SHA-256 of ciphertext bytes
- Expected: VaultRecord created with correct fields

### 14.2 Normal Reconstruction

- Vault: 2-of-5 threshold
- Validators B and D authorize access
- Expected: VaultAccessAuthorized event emitted with correct validators, nonce, expiry

### 14.3 Shard Rotation

- Vault: 2-of-5, threshold 2
- Initiator: validator B
- New shard holders: B, C, D, G (E and F replaced by G)
- New threshold: 2
- Expected: Time-locked rotation, endorsements from B and C (2 of 5, meets ceil(2*5/3)=4? No — 2 endorsements from the current set of 5 is NOT enough for ceil(2*5/3)=4. This test vector needs 4 endorsements.)

**Corrected:** Initiate rotation, get endorsements from B, C, D, E (4 of 5, meets ceil(2*5/3)=4). After 7 days, execute. VaultRecord updated.

### 14.4 Emergency Recovery

- Vault: 2-of-5, subject unavailable
- 4 validators authorize with purpose "emergency_recovery_court_order_#123"
- Expected: VaultAccessAuthorized event emitted

### 14.5 Collusion Scenario

- Vault: 2-of-5, threshold 2
- Validators B and C try to rotate shards without quorum (only 2 of 5, need 4)
- Expected: `QuorumNotMetForRotation` error

### 14.6 Self-Endorsement Prevention

- Vault: 2-of-5
- Initiator: validator B
- Validator B tries to endorse their own rotation
- Expected: `AlreadyEndorsedRotation` or self-endorsement error

### 14.7 Ping Liveness

- Vault: last_ping_at = 3 weeks ago
- Validator D calls ping_shard
- Expected: Success, last_ping_at updated

- Vault: last_ping_at = 3 days ago
- Validator D calls ping_shard
- Expected: `PingIntervalNotElapsed` error
