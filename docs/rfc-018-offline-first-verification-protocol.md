# RFC-018 — Offline-First Verification Protocol (no devnet required)

- Status: PROPOSED
- Depends on: RFC-012 (Phases 1–6), RFC-015, RFC-017
- Module: `terra-web` frontend + `terra-geo` SDK (no program adapter required)
- Chain-agnostic; no Solana RPC, no devnet, no wallet connection required
- Deterministic; every state transition is reproducible from a seed

---

## 0. Motivation

The Terra MVP must be verifiable without a live devnet connection. Sprint D
(devnet hosting, faucet rotation, managed PostGIS) is blocked by Q-001 (zero
SOL in the deployer wallet — founder action). Until devnet is available, all
protocol verification must run entirely offline in the browser using
deterministic simulation.

This RFC defines an offline-first verification protocol that replaces
live-chain interaction with local deterministic replay, content-addressed
verification, and seeded randomness. It is the verification layer for the MVP
flow (RFC-015 components, RFC-017 evidence intake) and requires **zero network
access after the initial page load**.

---

## 1. Design principles

1. **No network after load.** All verification happens in-browser using local
   data structures. No RPC calls, no wallet signing, no devnet transactions.
2. **Deterministic replay.** Given the same seed and event sequence, every
   state transition produces identical results across clients. Verification is
   reproducible by re-running the seed.
3. **Content-addressed trust.** Every artifact is identified by its SHA-256
   content digest. Verification recomputes the digest and compares; no
   trusted third party is needed.
4. **Signed manifest anchoring.** Surveyor-issued W3C VCs (SD-JWT) carry an
   on-chain-compatible digest that can be verified locally against the
   serialized credential. Selective disclosure reveals only chosen claims.
5. **Offline persistence.** All state (parcels, claims, evidence, attestations,
   tasks, results) persists in `localStorage` and survives page reloads.

---

## 2. Offline state model

All state lives in `localStorage` under the key `terra:offline:v1`. The state
is a single JSON object:

```json
{
  "version": 1,
  "seed": "hex-encoded-32-bytes",
  "parcels": { "<id>": Parcel },
  "claims": { "<id>": Claim },
  "evidence": { "<id>": Evidence },
  "observations": { "<id>": Observation },
  "tasks": { "<id>": Task },
  "attestations": { "<id>": Attestation },
  "residencyProofs": { "<id>": ResidencyProof },
  "verifications": { "<id>": VerificationResult }
}
```

### Parcel
```ts
interface Parcel {
  id: string           // hex-encoded 32-byte content digest
  name: string
  geometry: GeoJSON.Polygon
  owner: string        // public key base58
  status: 'pending' | 'registered' | 'for_sale' | 'transferred'
  createdAt: number    // unix ms
}
```

### Claim
```ts
interface Claim {
  id: string
  parcelId: string
  claimant: string     // public key base58
  claimType: 'ownership' | 'residency' | 'usage'
  status: 'submitted' | 'under_verification' | 'verified' | 'rejected'
  evidenceIds: string[]
  createdAt: number
}
```

### Evidence (RFC-017 intake pipeline)
```ts
interface Evidence {
  id: string
  claimId: string
  kind: 'photo' | 'document' | 'survey' | 'witness'
  contentDigest: string   // sha-256 hex of off-chain bytes (Irys/Arweave)
  storageReference: string // Irys/Arweave URI (off-chain blob)
  size: number
  mimeType: string
  submittedAt: number
  verified: boolean
}
```

### Observation
```ts
interface Observation {
  id: string
  claimId: string
  observer: string       // validator public key base58
  observationType: 'boundary_check' | 'evidence_review' | 'site_visit'
  result: 'confirmed' | 'disputed' | 'unable_to_verify'
  confidence: number     // 0–100
  signed: boolean        // whether the observer signed with their key
  createdAt: number
}
```

### Task (validator routing)
```ts
interface Task {
  id: string
  claimId: string
  taskType: 'evidence_verification' | 'boundary_survey' | 'residency_check'
  candidates: string[]   // validator public keys
  winner: string | null
  competitorCount: number
  vrfRandomness: string  // hex-encoded 32 bytes (Switchboard VRF / Pyth Entropy)
  status: 'pending' | 'assigned' | 'completed'
  createdAt: number
}
```

### Attestation (W3C VC + SD-JWT, RFC-015 §4)
```ts
interface Attestation {
  id: string
  claimId: string
  validator: string
  observationId: string
  credentialSubject: string
  vcType: 'VerificationAttestation' | 'ResidencyAttestation'
  issuanceDate: number
  expirationDate: number
  sdJwtDigest: string    // sha-256 hex of serialized SD-JWT
  selectiveDisclosure: boolean
  createdAt: number
}
```

### ResidencyProof (dual-sign)
```ts
interface ResidencyProof {
  id: string
  parcelId: string
  ownerPubkey: string
  tenantPubkey: string
  ownerSignature: string   // hex-encoded signature
  tenantSignature: string  // hex-encoded signature
  timestamp: number
  verified: boolean        // verified by mock verifier
}
```

---

## 3. Deterministic randomness (replaces blockhash)

Validator routing uses seeded randomness instead of blockhash. The seed is
established at protocol initialization and never changes:

```ts
// Seed derivation: user-provided or random at first load
const seed = localStorage.getItem('terra:offline:seed') ?? randomBytes(32)

// Per-task VRF randomness derived from seed + task id + counter
function deriveVrfRandomness(seed: Uint8Array, taskId: Uint8Array, counter: number): Uint8Array {
  return sha256([seed, taskId, u64le(counter)])
}

// Route seed: task_id ⊕ VRF randomness ⊕ slot
function routeSeed(taskId: Uint8Array, vrfRandomness: Uint8Array, slot: number): Uint8Array {
  return sha256([b"terra:route:v2", taskId, vrfRandomness, u64le(slot)])
}

// Winner selection: deterministic pick from eligible candidates
function selectWinner(seed: Uint8Array, candidates: Pubkey[]): Pubkey {
  const index = seedToIndex(seed, candidates.length)
  return candidates[index]
}
```

No blockhash, no Solana clock sysvar, no devnet. All randomness is derived
from the seed and is reproducible.

---

## 4. Verification flow (MVP: one parcel end-to-end)

### Step 1: Surveyor attests parcel → issues W3C VC

The surveyor creates a parcel and issues a Verifiable Credential:

1. Generate parcel content digest: `sha256(parcel_id || name || geometry_hash)`
2. Create SD-JWT with claims: `{ issuer, subject: parcel_id, type: "ResidencyAttestation", issuance_date, expiration_date }`
3. Compute `sd_jwt_digest = sha256(sd_jwt_bytes)`
4. Store attestation with `sd_jwt_digest` anchored locally

Verification (offline):
- Recompute `sha256(sd_jwt)` from stored SD-JWT
- Compare with stored `sd_jwt_digest`
- Verify issuer key is registered (local registry)
- Check `issuance_date <= now <= expiration_date`

### Step 2: Owner + tenant dual-sign residency proof

Both parties sign the same residency message:

```ts
const message = `RESIDENCY_PROOF\nowner:${ownerPubkey}\ntenant:${tenantPubkey}\nparcel:${parcelId}\ntimestamp:${unixSeconds}`
const ownerSig = sign(message, ownerKey)
const tenantSig = sign(message, tenantKey)
```

Verification (offline):
- Verify `ownerSignature` with `ownerPubkey`
- Verify `tenantSignature` with `tenantPubkey`
- Check both signatures are present and valid
- Check `parcelId` matches the VC credential subject

### Step 3: Mock verifier validates everything

The mock verifier performs all checks locally:

1. **VC digest check**: `sha256(stored SD-JWT) === sd_jwt_digest`
2. **VC validity window**: `issuance_date <= now <= expiration_date`
3. **Residency dual-sign**: both owner and tenant signatures valid
4. **Parcel binding**: VC subject === residency proof parcelId
5. **Result**: `valid` if all checks pass, `invalid` otherwise

No network, no RPC, no devnet. All checks are pure functions of local state.

---

## 5. Content-addressed storage (RFC-013 Phase C / RFC-017)

3D reconstruction blobs and evidence documents live off-chain (Irys/Arweave
compatible). On-chain/local state stores only:

- `contentDigest`: SHA-256 hex of the blob
- `h3Index`: H3 cell index for spatial lookup (Intent 4)
- `storageReference`: off-chain URI (not verified on-chain)

Verification recomputes `sha256(blob_bytes)` and compares to `contentDigest`.
Any mismatch rejects the artifact.

---

## 6. Test harness (no devnet)

A pure-JavaScript test runner executes the full MVP flow offline:

```ts
// Test: surveyor → VC → dual-sign → verify
const seed = randomBytes(32)
const parcel = createParcel(seed, 'test-parcel-1')
const vc = issueVC(seed, parcel, surveyorKey)
assert(verifyVC(vc, parcel))  // digest matches, validity window ok

const proof = dualSignResidency(seed, parcel, ownerKey, tenantKey)
assert(verifyResidency(proof, parcel))  // both signatures valid

const result = mockVerify(vc, proof)
assert(result === 'valid')  // all checks pass
```

All tests run in the browser or Node.js with no network access. Vitest is the
test runner (already configured in `terra-web`).

---

## 7. Security properties

| Property | Mechanism |
|---|---|
| Integrity | SHA-256 content digests on all artifacts |
| Authenticity | Ed25519 signatures on VCs and residency proofs |
| Replay resistance | Timestamps + nonces on all messages |
| Selective disclosure | SD-JWT reveals only chosen claims |
| Determinism | Seed-derived randomness; no blockhash, no clock |
| Offline verifiability | All verification is pure functions of local state |

---

## 8. Limitations

- No on-chain finality (by design; devnet blocked by Q-001)
- No slashing enforcement (requires live validators)
- No cross-client state sync (localStorage is per-browser)
- No real identity binding (surveyor keys are local-only)

These are acceptable for the MVP. Devnet deployment (Sprint D) adds live-chain
verification when funding is available.

---

## 9. Related

- RFC-015: Spatial component package (content-addressed artifacts)
- RFC-017: Secure evidence intake (quarantine, sandbox, rendition)
- RFC-013: Spatial intelligence pipeline (3D models, H3 indexing)
- INT-004: GIS bridge (server-side routing, mirror hash anchoring)
