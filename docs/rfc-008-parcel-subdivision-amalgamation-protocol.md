# RFC-008: Parcel Subdivision & Amalgamation Protocol

## 1. Status

- **Status:** Draft
- **Created:** 2026-09-03
- **Supersedes:** None

## 2. Summary

This RFC specifies the **Parcel Subdivision & Amalgamation Protocol** — a geometric state-transition layer for Terra that allows a single parcel to split into N sub-parcels, or N parcels to merge into one, while correctly migrating all dependent state (rights, attestations, infrastructure flags, disputes). Every current instruction assumes exactly one parcel. Real land routinely splits or merges: a farmer subdivides a field for sale, two adjacent lots merge into one development site. Rights, attestations, vaults, and disputes must migrate correctly. This RFC closes that gap.

The protocol reuses the **Attestation quorum pattern** for surveyor sign-off on new geometry and reuses **Succession-style re-pointing logic** (the `remaining_accounts` pattern already proven in `claim_succession`) for migrating rights to new parcel PDAs. Surveyor attestations bind cryptographically verified off-chain survey data to the new geometry hashes. A `SubdivisionRecord` PDA tracks the full migration lineage, enabling any observer to walk the ancestry of a parcel and verify that no rights, attestations, or disputes were orphaned.

**Target phase:** 7 (Regional expansion) — genuinely common in real land transactions, but not needed for a single-region pilot with a handful of parcels.

## 3. Threat Model

### 3.1 Adversary Classes

| Class | Description | Mitigation |
|-------|-------------|------------|
| **Fraudulent surveyor** | Submits falsified geometry to gain land area | Surveyor attestation quorum (k-of-n validators must co-sign off-chain survey data); content_hash binding |
| **Rights hijacker** | Attempts to drop or misdirect rights during migration | Each right is explicitly re-pointed via `migrate_rights`; SubdivisionRecord tracks provenance; event emission for audit |
| **Orphaning attacker** | Tries to leave attestations or disputes dangling on a dead parcel | Attestation migration closes old attestations and creates new ones; dispute records are closed or re-pointed by the adjudicator |
| **Amalgamation griefing** | Merges parcels with conflicting ownership to freeze a legitimate owner out | Conflicting rights must be explicitly resolved (revoked or re-granted) before amalgamation is allowed |
| **几何欺诈 (Geometry fraud)** | Claims subdivision produces more area than the parent parcel | Area proportions are declared on-chain and verified by the surveyor quorum; cascading checks reject if sum of sub-parcel areas exceeds parent |
| **Double-migration** | Attempts to subdivide a parcel that is already in-flight for subdivision | Status gate: only REGISTERED parcels can be subdivided; SubdivisionRecord tracks in-flight operations |

### 3.2 In Scope

- Parcel subdivision (1 → N parcels) with rights, attestation, and infrastructure migration
- Parcel amalgamation (N → 1 parcel) with rights merging and conflict resolution
- Surveyor attestation binding for new geometry (reuses existing Attestation pattern)
- Rights migration helper instruction (reuses Succession remaining_accounts pattern)
- SubdivisionRecord PDA for lineage tracking and auditability
- Event emission for full on-chain audit trail

### 3.3 Out of Scope

- Vault shard migration (covered by RFC-003 rotation protocol)
- Dispute filing and adjudication (covered by RFC-007; disputes are re-pointed but not created here)
- Identity / succession flows (covered by RFC-001/002)
- Off-chain survey data storage (application-layer concern; only content_hash lives on-chain)

## 4. Cryptographic Choices

### 4.1 Hashing: SHA-256

- Geometry hashes (`geometry_hash`) are SHA-256 digests of canonicalized GeoJSON/WKT parcel boundaries.
- Content hashes (`content_hash`) anchor off-chain survey documents (coordinate files, surveyor certificates).
- SubdivisionRecord lineage hashes are SHA-256 over the tuple `(parent_id, child_ids, timestamp)`.

### 4.2 Signatures: Ed25519

- Solana native — all validator and authority keys are Ed25519.
- Surveyor attestations require k-of-n Ed25519 signatures off-chain, verified against the on-chain validator set.
- Owner co-signature is required for both subdivision and amalgamation (the parcel owner must initiate).

### 4.3 Surveyor Quorum: Reused Attestation Pattern

- The existing `Attestation` PDA (`["attestation", parcel, specifier]`) is reused to record surveyor sign-off.
- `specifier` = SHA-256 of the survey session ID or ceremony identifier.
- `content_hash` = SHA-256 of the canonicalized survey output (new geometry coordinates, area calculations, boundary descriptions).
- `required` = k (threshold of validators who must co-sign).
- `validators` = the declared surveyor/validator set for this subdivision event.

### 4.4 Area Representation

- Area is not stored on-chain (geometric computation is an off-chain concern).
- The on-chain record stores only geometry hashes and the SubdivisionRecord, which declares the child parcel IDs and the original parcel ID.
- Area proportionality is verified off-chain by the surveyor quorum before they co-sign the attestation.

## 5. Data Model

### 5.1 Parcel (existing, unchanged)

**PDA seed:** `["parcel", id]`

| Field | Type | Description |
|-------|------|-------------|
| `id` | `[u8; 32]` | Unique 32-byte identifier (PDA seed, immutable) |
| `owner` | `Pubkey` | Current owner wallet |
| `name` | `String` | Human-readable parcel name (max 64 bytes) |
| `geometry_hash` | `[u8; 32]` | SHA-256 of canonicalized boundary geometry |
| `status` | `u8` | REGISTERED(1), FOR_SALE(2), etc. |
| `rights_count` | `u8` | Monotonic nonce for Rights PDA seeds |
| `infrastructure_flags` | `u16` | Bitmask of available infrastructure |
| `access_hash` | `[u8; 32]` | SHA-256 of off-chain infra/access validation |
| `created_at` | `i64` | Creation timestamp |
| `updated_at` | `i64` | Last update timestamp |

### 5.2 SubdivisionRecord (new PDA)

**PDA seed:** `["subdivision", original_parcel, sub_parcel]`

One SubdivisionRecord is created per sub-parcel, linking it back to the original parcel. This enables walking the full ancestry graph.

| Field | Type | Description |
|-------|------|-------------|
| `original_parcel` | `Pubkey` | The parent parcel's PDA key |
| `sub_parcel` | `Pubkey` | The child sub-parcel's PDA key |
| `original_geometry_hash` | `[u8; 32]` | Parent's geometry hash at time of subdivision |
| `new_geometry_hash` | `[u8; 32]` | Child's geometry hash |
| `surveyor_attestation` | `Pubkey` | Attestation PDA that recorded surveyor sign-off |
| `rights_migrated` | `bool` | Whether rights have been migrated to the sub-parcel |
| `attestations_migrated` | `bool` | Whether attestations have been migrated |
| `initiated_by` | `Pubkey` | Wallet that initiated the subdivision |
| `created_at` | `i64` | Subdivision timestamp |
| `completed_at` | `i64` | When migration finished (0 if in progress) |
| `status` | `u8` | 0=Pending, 1=Completed, 2=Failed |

### 5.3 AmalgamationRecord (new PDA)

**PDA seed:** `["amalgamation", result_parcel, source_parcel]`

One AmalgamationRecord is created per source parcel being merged into the result.

| Field | Type | Description |
|-------|------|-------------|
| `result_parcel` | `Pubkey` | The merged result parcel's PDA key |
| `source_parcel` | `Pubkey` | A source parcel being merged in |
| `source_geometry_hash` | `[u8; 32]` | Source's geometry hash at time of amalgamation |
| `result_geometry_hash` | `[u8; 32]` | Result's geometry hash |
| `rights_merged` | `bool` | Whether rights from this source have been merged |
| `initiated_by` | `Pubkey` | Wallet that initiated the amalgamation |
| `created_at` | `i64` | Amalgamation timestamp |
| `completed_at` | `i64` | When merge finished (0 if in progress) |
| `status` | `u8` | 0=Pending, 1=Completed, 2=Failed |

### 5.4 Rights (existing, unchanged)

**PDA seed:** `["rights", parcel, nonce]`

| Field | Type | Description |
|-------|------|-------------|
| `parcel` | `Pubkey` | The parcel this right is attached to |
| `rights_kind` | `u8` | OWNERSHIP(0), USAGE(1), EASEMENT(2), SERVITUDE(3), LIEN(4) |
| `holder` | `Pubkey` | Party holding the right |
| `granter` | `Pubkey` | Party who granted the right (parcel owner) |
| `created_at` | `i64` | Creation timestamp |
| `expires_at` | `i64` | Expiration timestamp (0 = no expiration) |
| `notes` | `String` | Human-readable notes (max 128 bytes) |

### 5.5 Attestation (existing, unchanged)

**PDA seed:** `["attestation", parcel, specifier]`

| Field | Type | Description |
|-------|------|-------------|
| `parcel` | `Pubkey` | The parcel this attestation is bound to |
| `specifier` | `[u8; 32]` | 32-byte specifier (survey session ID) |
| `content_hash` | `[u8; 32]` | SHA-256 of off-chain payload |
| `required` | `u8` | Required validator signatures |
| `count` | `u8` | Number of validator keys registered |
| `version` | `u8` | Monotonic rotation counter |
| `created_at` | `i64` | Creation timestamp |
| `updated_at` | `i64` | Last update timestamp |
| `validators` | `[Pubkey; MAX_VALIDATORS]` | Validator public keys |

### 5.6 Cascading State Inventory

Before implementing any subdivision or amalgamation instruction, map every account type that references `parcel_id` or `parcel` (Pubkey):

| Account Type | Reference Field | Migration Path |
|-------------|----------------|----------------|
| `Parcel` | `id` (PDA seed) | New PDA per sub-parcel; old parcel status → SUBDIVIDED |
| `Rights` | `parcel` (field) | Re-pointed via `migrate_rights` (close old, init new) |
| `Attestation` | `parcel` (field) | Old attestation closed; new attestation created per sub-parcel |
| `Dispute` | `parcel` (field) | Closed (cancelled) before subdivision proceeds |
| `DocumentAnchor` | `attestation` (PDA seed) | Migrated when parent attestation is closed |
| `Identity` | `parcel_count` | Decremented by N-1 on subdivision (N new parcels, 1 old closed) |

## 6. Instructions

### 6.1 `subdivide_parcel`

**Purpose:** Split one parcel into N sub-parcels. Requires surveyor attestation for new geometry. Creates new Parcel PDAs and SubdivisionRecord PDAs. Rights and attestations are NOT migrated in this instruction — a separate `migrate_rights` call handles that.

**Accounts:**
- `original_parcel` (mut, PDA `["parcel", id]`) — the parent parcel, status must be REGISTERED
- `sub_parcel` (init, PDA `["parcel", new_id]`) — one of the N child sub-parcels
- `subdivision_record` (init, PDA `["subdivision", original_parcel.key(), sub_parcel.key()]`)
- `surveyor_attestation` (readonly, PDA `["attestation", original_parcel.key(), specifier]`) — surveyor sign-off attestation
- `authority` (signer, mut — must be original parcel owner; pays rent)
- `system_program`

**Args:** `new_id: [u8; 32]`, `new_name: String`, `new_geometry_hash: [u8; 32]`, `specifier: [u8; 32]`

**Guards:**
- `original_parcel.status == parcel_status::REGISTERED`
- `authority.key() == original_parcel.owner`
- `new_id != [0; 32]`
- `new_geometry_hash != [0; 32]`
- `new_name` is non-empty and ≤ 64 bytes
- `surveyor_attestation.parcel == original_parcel.key()`
- `surveyor_attestation.content_hash != [0; 32]` (attestation must have been completed off-chain)
- `surveyor_attestation.count >= surveyor_attestation.required` (quorum must be met)
- No existing SubdivisionRecord for `(original_parcel.key(), sub_parcel.key())` (prevent double-subdivision of the same child)

**Effects:**
- Creates `sub_parcel` with `owner = original_parcel.owner`, `status = REGISTERED`, `geometry_hash = new_geometry_hash`, `name = new_name`
- Creates `subdivision_record` linking original → sub
- Sets `original_parcel.status = SUBDIVIDED` (new status constant: value 8)
- Sets `original_parcel.updated_at = now`

**Emits:** `ParcelSubdivided`

### 6.2 `amalgamate_parcels`

**Purpose:** Merge N source parcels into one result parcel. The result parcel must already exist (registered) or be the first source. Conflicting rights must be resolved before this instruction executes. Rights are NOT merged in this instruction — a separate `migrate_rights` call handles that.

**Accounts:**
- `result_parcel` (mut, PDA `["parcel", result_id]`) — the parcel that absorbs the others
- `source_parcel` (mut, PDA `["parcel", source_id]`) — one of the N parcels being merged
- `amalgamation_record` (init, PDA `["amalgamation", result_parcel.key(), source_parcel.key()]`)
- `authority` (signer, mut — must be owner of BOTH result and source parcels)
- `system_program`

**Args:** `new_geometry_hash: [u8; 32]`

**Guards:**
- `result_parcel.status == parcel_status::REGISTERED`
- `source_parcel.status == parcel_status::REGISTERED`
- `authority.key() == result_parcel.owner`
- `authority.key() == source_parcel.owner` (same owner for both — prevents hostile merges)
- `result_parcel.key() != source_parcel.key()` (cannot merge a parcel with itself)
- `new_geometry_hash != [0; 32]`
- No conflicting rights exist between the two parcels (verified by the caller: all EASEMENT/SERVITUDE/LIEN rights between the two parcels must have been revoked before calling this instruction)
- No active disputes on either parcel

**Effects:**
- Updates `result_parcel.geometry_hash = new_geometry_hash`
- Updates `result_parcel.updated_at = now`
- Creates `amalgamation_record` linking result → source
- Sets `source_parcel.status = AMALGAMATED` (new status constant: value 9)
- Sets `source_parcel.updated_at = now`

**Emits:** `ParcelsAmalgamated`

### 6.3 `migrate_rights`

**Purpose:** Migrate rights from an old parcel to a new parcel. Works for both subdivision (old → each sub) and amalgamation (each source → new result). Reuses the Succession `remaining_accounts` pattern: the caller supplies the Rights PDA accounts in `remaining_accounts`, and the instruction re-points each one.

**Accounts:**
- `old_parcel` (mut, PDA `["parcel", old_id]`) — the parcel losing rights
- `new_parcel` (mut, PDA `["parcel", new_id]`) — the parcel gaining rights
- `authority` (signer, mut — must be owner of old_parcel)
- `system_program`

**Args:** None (the Rights accounts are in `remaining_accounts`)

**Guards:**
- `authority.key() == old_parcel.owner`
- `old_parcel.key() != new_parcel.key()`
- Each account in `remaining_accounts` must:
  - Be owned by this program
  - Have a valid Rights discriminator
  - Have `rights.parcel == old_parcel.key()` (currently belong to the old parcel)

**Effects (per right in remaining_accounts):**
- Closes the old Rights account (returns lamports to authority)
- Creates a new Rights account with `parcel = new_parcel.key()` and all other fields copied from the old right
- Increments `new_parcel.rights_count` for each migrated right

**Emits:** `RightsMigrated` (once per right)

### 6.4 `migrate_attestations`

**Purpose:** Migrate attestations from an old parcel to a new parcel. Closes old attestation accounts and creates new ones for each sub-parcel. Used after subdivision to ensure the new parcels have their own attestation records.

**Accounts:**
- `old_parcel` (readonly, PDA `["parcel", old_id]`)
- `new_parcel` (readonly, PDA `["parcel", new_id]`)
- `old_attestation` (mut, PDA `["attestation", old_parcel.key(), specifier]`) — to be closed
- `new_attestation` (init, PDA `["attestation", new_parcel.key(), specifier]`) — created with same specifier
- `authority` (signer, mut — must be owner of old_parcel)
- `system_program`

**Args:** `specifier: [u8; 32]`

**Guards:**
- `authority.key() == old_parcel.owner`
- `old_attestation.parcel == old_parcel.key()`
- `old_attestation.count >= old_attestation.required` (attestation must be completed)
- No existing attestation with this specifier on `new_parcel`

**Effects:**
- Creates `new_attestation` with all fields copied from `old_attestation` except `parcel = new_parcel.key()`
- Closes `old_attestation` (returns lamports to authority)

**Emits:** `AttestationMigrated`

## 7. Off-Chain Protocol

### 7.1 Subdivision Ceremony

1. **Surveyor engagement:** The parcel owner engages a licensed surveyor (or surveyor team) to produce new boundary descriptions for the N sub-parcels.
2. **Survey data production:** The surveyor produces canonicalized geometry (GeoJSON/WKT) for each sub-parcel, plus area calculations, boundary descriptions, and any legal metadata.
3. **Content hash computation:** SHA-256 of the canonicalized survey output → `content_hash`.
4. **Validator quorum:** k-of-n validators (the surveyor attestation set) review the survey data off-chain and co-sign an Ed25519 attestation over the content_hash.
5. **Attestation registration:** The owner calls `attest` on-chain to create an Attestation PDA with the surveyor's content_hash, specifier, and validator set.
6. **Subdivision transaction:** The owner calls `subdivide_parcel` for each sub-parcel, passing the new geometry_hash, name, and referencing the surveyor attestation by specifier. This is done in a single Solana transaction (via CPI or as separate instructions in the same tx) for atomicity.
7. **Rights migration:** After subdivision, the owner calls `migrate_rights` to move rights from the old parcel to each sub-parcel. This can be batched across multiple sub-parcels in a single transaction.
8. **Attestation migration:** For each completed attestation on the old parcel, the owner calls `migrate_attestations` to close the old attestation and create a new one on the sub-parcel.
9. **Identity update:** The owner's Identity `parcel_count` is updated (decremented by N-1, then incremented by N, net effect: +0, but the accounting changes).
10. **Audit:** Any observer can walk the SubdivisionRecord chain to verify that no rights or attestations were lost.

### 7.2 Amalgamation Ceremony

1. **Owner verification:** The owner must own ALL parcels being merged. If different owners, a transfer must happen first.
2. **Rights conflict resolution:** Before amalgamation, all cross-parcel rights (easements, servitudes, liens between the parcels) must be revoked. This is an off-chain legal process documented by the owner.
3. **Surveyor engagement:** A surveyor produces new boundary geometry for the merged parcel.
4. **Content hash computation:** SHA-256 of the canonicalized merged geometry → `content_hash`.
5. **Attestation registration:** k-of-n validators co-sign the new geometry attestation.
6. **Amalgamation transaction:** The owner calls `amalgamate_parcels` for each source parcel, passing the new geometry_hash. All sources are merged in a single transaction.
7. **Rights migration:** The owner calls `migrate_rights` to move rights from each source parcel to the result parcel.
8. **Attestation migration:** Completed attestations on source parcels are migrated to the result parcel.
9. **Source parcel closure:** Source parcels are marked AMALGAMATED. Optionally, their lamports can be reclaimed.

### 7.3 Cascading State Migration Checklist

For every subdivision or amalgamation, verify ALL of the following before emitting the completion event:

- [ ] All Rights accounts on the old parcel have been migrated (no dangling Rights)
- [ ] All Attestation accounts on the old parcel have been migrated or explicitly closed
- [ ] All DocumentAnchor accounts referencing migrated attestations have been handled
- [ ] All active Disputes on the old parcel have been cancelled or re-pointed
- [ ] The Identity `parcel_count` has been updated
- [ ] The old parcel status has been updated (SUBDIVIDED or AMALGAMATED)
- [ ] No other account types reference the old parcel_id (exhaustive search)

### 7.4 Side-Channel Requirements

- **Surveyor coordination:** Surveyor teams coordinate via secure channels (Signal, in-person). Survey data is transmitted encrypted or in person.
- **Legal documentation:** Amalgamation requires documented consent from all parcel owners (even if the same person, for audit trail).
- **Boundary disputes:** Any boundary disputes must be resolved before subdivision proceeds (via RFC-007 dispute flow or off-chain legal process).

## 8. Collusion Resistance

### 8.1 Surveyor Attestation Quorum

- Subdivision requires k-of-n validator signatures on the surveyor attestation (same threshold model as existing Attestation pattern).
- A single rogue surveyor cannot unilaterally alter geometry — k validators must co-sign.
- The surveyor attestation is bound to the specific parcel via the Attestation PDA seed, preventing cross-parcel substitution.

### 8.2 Owner Co-Sign Requirement

- Both subdivision and amalgamation require the parcel owner's signature.
- Amalgamation additionally requires the owner of BOTH parcels (prevents hostile merges).
- A rogue validator cannot subdivide or amalgamate a parcel without the owner's cooperation.

### 8.3 Migration Audit Trail

- SubdivisionRecord and AmalgamationRecord PDAs create an immutable on-chain lineage chain.
- Any observer can walk the graph: `original_parcel → subdivision_records → sub_parcels → ...`
- RightsMigrated events emit per-right, enabling detection of dropped or misdirected rights.
- AttestationMigrated events emit per-attestation, enabling detection of orphaned attestations.

### 8.4 Status Gates

- Only REGISTERED parcels can be subdivided or amalgamated.
- DISPUTED, FROZEN, FOR_SALE, or TRANSFERRED parcels cannot be geometrically altered.
- This prevents subdividing a parcel mid-dispute to fragment accountability.

### 8.5 No Self-Dealing in Migration

- The authority must own the old parcel (enforced on-chain).
- In amalgamation, the authority must own BOTH parcels (enforced on-chain).
- A validator who is also the parcel owner cannot both propose and validate the same subdivision (self-dealing check via remaining_accounts signer verification).

## 9. Liveness Guarantees

### 9.1 Atomic Subdivision

Subdivision of a single parcel into N sub-parcels should be done in a single Solana transaction. If the transaction fails midway (e.g., rent-exceeds-budget), no partial state is committed — Solana transactions are atomic.

### 9.2 Rights Migration Liveness

Rights migration is a separate instruction. If it fails (e.g., account rent issues), the old parcel remains SUBDIVIDED and the rights remain on the old parcel. The owner can retry `migrate_rights` until successful. No rights are lost — they are still valid on the old parcel until explicitly migrated.

### 9.3 Amalgamation Liveness

Amalgamation can only proceed when all source parcels are REGISTERED. If one source parcel is in a non-REGISTERED status (e.g., DISPUTED), the amalgamation is blocked until that status is resolved. This is by design — geometric changes during disputes would fragment accountability.

### 9.4 Recovery from Partial Migration

If a subdivision or amalgamation fails partway through:
- The SubdivisionRecord or AmalgamationRecord remains in `Pending` status.
- The owner can re-attempt the migration.
- If the owner is unable to complete migration (e.g., key loss), succession/recovery flows can take over and complete the migration on their behalf.

## 10. Storage Architecture

### 10.1 On-Chain State

- **Parcel PDAs:** One per sub-parcel (new PDAs with unique 32-byte IDs).
- **SubdivisionRecord PDAs:** One per sub-parcel, linking to the original.
- **AmalgamationRecord PDAs:** One per source parcel, linking to the result.
- **Rights PDAs:** Closed on old parcel, re-created on new parcel (lamports reclaimed).
- **Attestation PDAs:** Closed on old parcel, re-created on new parcel (lamports reclaimed).

### 10.2 Off-Chain State

- **Survey data:** Canonicalized geometry, area calculations, boundary descriptions — stored off-chain (IPFS, Arweave, or local database).
- **Content hashes:** SHA-256 digests of survey data stored on-chain in Attestation PDAs.
- **Lineage graph:** Constructed by walking SubdivisionRecord and AmalgamationRecord PDAs on-chain.

### 10.3 CID Verification Protocol

1. Fetch survey data from IPFS/Arweave using the CID referenced in the attestation.
2. Compute SHA-256 of the fetched bytes.
3. Compare against `attestation.content_hash` (on-chain).
4. If they match, the survey data is authentic and untampered.
5. Anyone can perform this verification — no trust in the API required.

## 11. Replay / Nonce Hygiene

### 11.1 PDA Uniqueness

- SubdivisionRecord PDA: `["subdivision", original_parcel.key(), sub_parcel.key()]` — unique per (parent, child) pair.
- AmalgamationRecord PDA: `["amalgamation", result_parcel.key(), source_parcel.key()]` — unique per (result, source) pair.
- Prevents replaying the same subdivision or amalgamation event.

### 11.2 Specifier Reuse Prevention

- Attestation specifiers are unique per parcel (`["attestation", parcel, specifier]`).
- When migrating an attestation from an old parcel to a new one, the same specifier is used but the parcel key changes, creating a new unique PDA.
- The guard `no existing attestation with this specifier on new_parcel` prevents double-migration of the same attestation.

### 11.3 Status Idempotency

- Attempting to subdivide an already-SUBDIVIDED parcel is rejected by the `status == REGISTERED` guard.
- Attempting to amalgamate an already-AMALGAMATED parcel is rejected by the `status == REGISTERED` guard.
- Attempting to migrate rights from a parcel that has no rights is a no-op (no accounts in remaining_accounts).

### 11.4 Nonce Exhaustion

- Each migrated right increments `new_parcel.rights_count`.
- If `rights_count` reaches u8::MAX (255), no more rights can be migrated. This is an extreme edge case — a parcel with 255 rights is implausible in practice.
- The existing `RightsLimitExceeded` error covers this case.

## 12. Post-Quantum Migration Path

### 12.1 Geometry Hash Agility

- The `geometry_hash` field is a `[u8; 32]` — agnostic to the hashing algorithm.
- If a post-quantum hash function is adopted (e.g., SHA-3, SPHINCS+), the same field can store the new hash type.
- The program only checks `geometry_hash != [0; 32]` — it does not enforce a specific hash algorithm.

### 12.2 Attestation Signature Agility

- The Attestation PDA stores validator public keys (`[Pubkey; MAX_VALIDATORS]`).
- If validator keys migrate to post-quantum signatures (e.g., Dilithium), the same field can hold the new key types.
- The off-chain verification logic must be updated, but the on-chain data model is agnostic.

### 12.3 SubdivisionRecord Lineage Integrity

- SubdivisionRecord lineage hashes are SHA-256 over `(parent_id, child_ids, timestamp)`.
- If the hash function changes, new SubdivisionRecords use the new hash; old records remain valid (content-addressed).
- The lineage chain can span hash algorithm transitions without breaking.

### 12.4 Timeline Considerations

- For the pilot (1-2 years), SHA-256 and Ed25519 are sufficient.
- Migration to post-quantum cryptography should happen before the system scales beyond the pilot region.
- The on-chain data model is designed to be algorithm-agnostic — only the off-chain verification logic needs updating.

## 13. Operational Security

### 13.1 Surveyor Verification

- Surveyors must be licensed and verified off-chain (by the AuthorityRegistry or local government).
- The on-chain attestation quorum ensures that k validators have reviewed the survey data.
- A fraudulent surveyor must collude with k-1 validators to falsify geometry — the same trust model as other attestation flows.

### 13.2 Owner Key Security

- The parcel owner's key is the root of trust for subdivision and amalgamation.
- If the owner's key is compromised, the attacker can subdivide or amalgamate the parcel.
- Mitigation: succession/recovery flows (RFC-001/002) can revoke the compromised key.

### 13.3 Batch Transaction Safety

- Subdivision of a parcel into N sub-parcels should be done in a single transaction to avoid partial state.
- If the transaction is too large (Solana transaction size limit ~1232 bytes), split into multiple transactions but ensure all are included in the same block (or use a program-derived authority to orchestrate).
- Rights migration can be done in subsequent transactions (no atomicity requirement with subdivision).

### 13.4 Audit Protocol

- After any subdivision or amalgamation, walk the SubdivisionRecord/AmalgamationRecord chain to verify:
  - All rights were migrated (no dangling Rights accounts on old parcels)
  - All attestations were migrated (no dangling Attestation accounts on old parcels)
  - No disputes were orphaned
  - Identity parcel_count is consistent
- This can be done by any observer using RPC calls — no special access required.

## 14. Test Vectors

### 14.1 Basic Subdivision (1 → 2)

- **Setup:** Parcel A (id=0xAA, owner=W1, status=REGISTERED, rights_count=2)
  - Right 0: OWNERSHIP held by W1
  - Right 1: EASEMENT held by W2
- **Action:** Subdivide A into B (id=0xBB) and C (id=0xCC)
- **Pre-conditions:**
  - Surveyor attestation exists on A with specifier=0x01, content_hash=0xSHA256(survey_data), required=2, count=2, validators=[V1, V2]
  - V1 and V2 have co-signed off-chain
- **Expected:**
  - Parcel B created: owner=W1, status=REGISTERED, geometry_hash=new_hash_B
  - Parcel C created: owner=W1, status=REGISTERED, geometry_hash=new_hash_C
  - SubdivisionRecord created: original_parcel=A, sub_parcel=B
  - SubdivisionRecord created: original_parcel=A, sub_parcel=C
  - Parcel A status → SUBDIVIDED (value 8)
  - `RightsMigrated` events emitted for Right 0 and Right 1 → re-pointed to B (or C, depending on migration split)
  - `ParcelSubdivided` event emitted

### 14.2 Subdivision Rejects Non-Owner

- **Setup:** Parcel A (owner=W1)
- **Action:** W2 calls `subdivide_parcel`
- **Expected:** `NotOwner` error

### 14.3 Subdivision Rejects Non-Registered Parcel

- **Setup:** Parcel A (status=FOR_SALE)
- **Action:** Owner calls `subdivide_parcel`
- **Expected:** `InvalidStatus` error

### 14.4 Subdivision Rejects Unmet Attestation Quorum

- **Setup:** Parcel A, surveyor attestation with required=2, count=1 (only 1 validator signed)
- **Action:** Owner calls `subdivide_parcel`
- **Expected:** Attestation quorum check fails (off-chain surveyor verification fails; on-chain, the attestation was never completed)

### 14.5 Basic Amalgamation (2 → 1)

- **Setup:** Parcel A (owner=W1, status=REGISTERED), Parcel B (owner=W1, status=REGISTERED)
- **Action:** Amalgamate A and B into A (A is the result)
- **Pre-conditions:**
  - No cross-parcel easements/servitudes between A and B
  - Surveyor attestation on merged geometry
- **Expected:**
  - Parcel A geometry_hash updated to merged geometry
  - AmalgamationRecord created: result_parcel=A, source_parcel=B
  - Parcel B status → AMALGAMATED (value 9)
  - `ParcelsAmalgamated` event emitted

### 14.6 Amalgamation Rejects Different Owners

- **Setup:** Parcel A (owner=W1), Parcel B (owner=W2)
- **Action:** W1 calls `amalgamate_parcels`
- **Expected:** `NotOwner` error (W1 does not own B)

### 14.7 Amalgamation Rejects Self-Merge

- **Setup:** Parcel A (owner=W1)
- **Action:** W1 calls `amalgamate_parcels` with result=A, source=A
- **Expected:** Error (result_parcel.key() == source_parcel.key())

### 14.8 Rights Migration (Subdivision Context)

- **Setup:** Parcel A with 3 rights (OWNERSHIP, USAGE, EASEMENT). Subdivided into B and C.
- **Action:** Owner calls `migrate_rights` with old_parcel=A, new_parcel=B, remaining_accounts=[Right0, Right1]
- **Expected:**
  - Right0 and Right1 closed on A (lamports returned)
  - New Right0 created on B: parcel=B, all other fields copied
  - New Right1 created on B: parcel=B, all other fields copied
  - B.rights_count incremented by 2
  - `RightsMigrated` events emitted for each

### 14.9 Rights Migration (Amalgamation Context)

- **Setup:** Parcel A (2 rights) and Parcel B (1 right) merged into A.
- **Action:** Owner calls `migrate_rights` with old_parcel=B, new_parcel=A, remaining_accounts=[Right0_B]
- **Expected:**
  - Right0_B closed on B
  - New right created on A: parcel=A, all other fields copied
  - A.rights_count incremented by 1
  - `RightsMigrated` event emitted

### 14.10 Attestation Migration

- **Setup:** Parcel A with completed attestation (specifier=0x01, required=2, count=2). Subdivided into B.
- **Action:** Owner calls `migrate_attestations` with old_parcel=A, new_parcel=B, specifier=0x01
- **Expected:**
  - Old attestation on A closed (lamports returned)
  - New attestation created on B: parcel=B, all other fields copied
  - `AttestationMigrated` event emitted

### 14.11 Duplicate Subdivision Record Rejected

- **Setup:** SubdivisionRecord already exists for (A, B)
- **Action:** Owner tries to subdivide A → B again
- **Expected:** SubdivisionRecord PDA init fails (already exists)

### 14.12 Collusion Scenario: Fraudulent Geometry

- **Setup:** Parcel A, surveyor attestation with required=2, count=2, validators=[V1, V2]
- **Scenario:** V1 and V2 collude to attest to false geometry that gives the owner 2x the area
- **Mitigation:** Off-chain area verification (the surveyor attestation is bound to a content_hash; anyone can verify the actual survey data against the hash). The on-chain program cannot verify area — it relies on the quorum model and audit trail. The SubdivisionRecord provides the lineage for post-hoc auditing.

### 14.13 Migration Completeness Check

- **Setup:** Parcel A subdivided into B and C. A has 2 rights, 1 attestation.
- **After migration:**
  - Walk all Rights accounts: none have `parcel == A.key()` (all migrated)
  - Walk all Attestation accounts: none have `parcel == A.key()` (all migrated)
  - SubdivisionRecord for (A, B): `rights_migrated = true`, `attestations_migrated = true`
  - SubdivisionRecord for (A, C): `rights_migrated = true`, `attestations_migrated = true`
  - Parcel A status = SUBDIVIDED
  - No disputes referencing A (cancelled before subdivision)
