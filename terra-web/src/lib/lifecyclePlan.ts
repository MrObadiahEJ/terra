// Pure, testable half of the Sprint B on-chain lifecycle: ids, PDAs, digests,
// step metadata and local operator-key derivation. No React, no RPC.
import { Keypair, PublicKey } from '@solana/web3.js'
import { TERRA_PROGRAM_ID } from './constants'

export const PROGRAM_ID = new PublicKey(TERRA_PROGRAM_ID)

export function u8(n: number): Uint8Array {
  return Uint8Array.of(n)
}

export function u32le(n: number): Uint8Array {
  const out = new Uint8Array(4)
  new DataView(out.buffer).setUint32(0, n, true)
  return out
}

export function u64le(n: number): Uint8Array {
  const out = new Uint8Array(8)
  new DataView(out.buffer).setBigUint64(0, BigInt(n), true)
  return out
}

export function bytes(...parts: (Uint8Array | number[])[]): Uint8Array {
  const chunks = parts.map((p) => (p instanceof Uint8Array ? p : Uint8Array.from(p)))
  const out = new Uint8Array(chunks.reduce((n, c) => n + c.length, 0))
  let offset = 0
  for (const chunk of chunks) {
    out.set(chunk, offset)
    offset += chunk.length
  }
  return out
}

/** SHA-256 over the concatenation of the given parts (strings are UTF-8). */
export async function digest(parts: (Uint8Array | string)[]): Promise<Uint8Array> {
  const chunks = parts.map((p) => (typeof p === 'string' ? new TextEncoder().encode(p) : p))
  const input = bytes(...chunks)
  const hash = await crypto.subtle.digest('SHA-256', input as BufferSource)
  return new Uint8Array(hash)
}

/** Exact attestation preimage from `verification/attestation.rs`. */
export async function sdJwtDigest(
  claim: PublicKey,
  validator: PublicKey,
  observation: PublicKey,
  credentialSubject: PublicKey,
  vcType: string,
  issuanceDate: number,
  expirationDate: number,
): Promise<Uint8Array> {
  return digest([
    'terra:vc:sd-jwt:v1',
    claim.toBuffer(),
    validator.toBuffer(),
    observation.toBuffer(),
    credentialSubject.toBuffer(),
    new TextEncoder().encode(vcType),
    u64le(issuanceDate),
    u64le(expirationDate),
  ])
}

function pda(seeds: (Uint8Array | string)[]): PublicKey {
  const parts = seeds.map((seed) => (typeof seed === 'string' ? new TextEncoder().encode(seed) : seed))
  return PublicKey.findProgramAddressSync(parts, PROGRAM_ID)[0]
}

export interface LifecycleIds {
  parcelId: Uint8Array
  claimId: Uint8Array
  taskId: Uint8Array
  disputeParcelId: Uint8Array
  caseHash: Uint8Array
}

export async function deriveLifecycleIds(seed: string): Promise<LifecycleIds> {
  const id = (label: string) => digest([`terra:lifecycle:v1:${seed}:${label}`])
  const [parcelId, claimId, taskId, disputeParcelId, caseHash] = await Promise.all([
    id('parcel'),
    id('claim'),
    id('task'),
    id('dispute-parcel'),
    id('dispute-case'),
  ])
  return { parcelId, claimId, taskId, disputeParcelId, caseHash }
}

export interface LifecyclePdas {
  registry: PublicKey
  feePolicy: PublicKey
  coverageIncentive: PublicKey
  treasury: PublicKey
  profile: (validator: PublicKey) => PublicKey
  availability: (validator: PublicKey) => PublicKey
  reputation: (validator: PublicKey) => PublicKey
  endorsement: (validator: PublicKey) => PublicKey
  parcel: PublicKey
  ownership: PublicKey
  usageRight: PublicKey
  claim: PublicKey
  evidence: PublicKey
  observation: (validator: PublicKey) => PublicKey
  attestation: (validator: PublicKey) => PublicKey
  task: PublicKey
  requirement: PublicKey
  quote: PublicKey
  taskEscrow: PublicKey
  taskVault: PublicKey
  assignment: (validator: PublicKey) => PublicKey
  rewardAllocation: (validator: PublicKey) => PublicKey
  asset: PublicKey
  geometryVersion: PublicKey
  snapshotMeta: PublicKey
  snapshot: PublicKey
  disputeParcel: PublicKey
  disputeOwnership: PublicKey
  dispute: PublicKey
}

export function deriveLifecyclePdas(ids: LifecycleIds): LifecyclePdas {
  const registry = pda(['validator_registry'])
  const parcel = pda(['parcel', ids.parcelId])
  const claim = pda(['claim', parcel.toBuffer(), ids.claimId])
  const task = pda(['task', ids.taskId])
  const taskEscrow = pda(['task_escrow', ids.taskId])
  const asset = pda(['spatial_asset', parcel.toBuffer()])
  const disputeParcel = pda(['parcel', ids.disputeParcelId])

  return {
    registry,
    feePolicy: pda(['fee_policy']),
    coverageIncentive: pda(['coverage_incentive']),
    treasury: pda(['task_treasury']),
    profile: (validator) => pda(['validator_profile', validator.toBuffer()]),
    availability: (validator) => pda(['validator_availability', validator.toBuffer()]),
    reputation: (validator) => pda(['validator_reputation', validator.toBuffer()]),
    endorsement: (validator) =>
      pda(['validator_endorsement', registry.toBuffer(), validator.toBuffer()]),
    parcel,
    ownership: pda(['ownership', parcel.toBuffer()]),
    usageRight: pda(['rights', parcel.toBuffer(), u8(0)]),
    claim,
    evidence: pda(['evidence', claim.toBuffer(), u8(0)]),
    observation: (validator) => pda(['observation', claim.toBuffer(), validator.toBuffer()]),
    attestation: (validator) =>
      pda(['verification_attestation', claim.toBuffer(), validator.toBuffer()]),
    task,
    requirement: pda(['task_requirement', ids.taskId, u8(0)]),
    quote: pda(['resource_quote', ids.taskId]),
    taskEscrow,
    taskVault: pda(['task_escrow_vault', taskEscrow.toBuffer()]),
    assignment: (validator) => pda(['task_assignment', ids.taskId, validator.toBuffer()]),
    rewardAllocation: (validator) =>
      pda(['reward_allocation', ids.taskId, validator.toBuffer()]),
    asset,
    geometryVersion: pda(['geometry_version', asset.toBuffer(), u32le(0)]),
    snapshotMeta: pda(['spatial_snapshot_meta', asset.toBuffer()]),
    snapshot: pda(['spatial_snapshot', asset.toBuffer(), u32le(0)]),
    disputeParcel,
    disputeOwnership: pda(['ownership', disputeParcel.toBuffer()]),
    dispute: pda(['dispute', disputeParcel.toBuffer(), ids.caseHash]),
  }
}

export interface OperatorKeypairs {
  validatorOne: Keypair
  validatorTwo: Keypair
  tenant: Keypair
}

/** Deterministic local operator keys derived from the stored lifecycle seed. */
export async function deriveOperatorKeypairs(seed: string): Promise<OperatorKeypairs> {
  const key = async (label: string) => {
    const seedBytes = await digest([`terra:lifecycle:v1:${seed}:operator:${label}`])
    return Keypair.fromSeed(seedBytes)
  }
  const [validatorOne, validatorTwo, tenant] = await Promise.all([
    key('validator-one'),
    key('validator-two'),
    key('tenant'),
  ])
  return { validatorOne, validatorTwo, tenant }
}

export type LifecycleStepId =
  | 'registry'
  | 'validators'
  | 'parcel'
  | 'claim'
  | 'attestations'
  | 'task'
  | 'geometry'
  | 'proof'
  | 'dispute'

export interface LifecycleStepDefinition {
  id: LifecycleStepId
  title: string
  description: string
}

export const LIFECYCLE_STEPS: LifecycleStepDefinition[] = [
  {
    id: 'registry',
    title: 'Create validator registry',
    description: 'Bootstrap the protocol registry with the connected wallet as admin.',
  },
  {
    id: 'validators',
    title: 'Register two validators',
    description: 'Local operator keys get profiles, online availability and reputation.',
  },
  {
    id: 'parcel',
    title: 'Register parcel + grant a right',
    description: 'Writes the parcel, its canonical ownership right and a USAGE right.',
  },
  {
    id: 'claim',
    title: 'Anchor evidence and observations',
    description: 'Claim, evidence hash, and one independent observation per validator.',
  },
  {
    id: 'attestations',
    title: 'Attest and verify by quorum',
    description: 'Two signed attestations move the claim to VERIFIED.',
  },
  {
    id: 'task',
    title: 'Fund, route and reward verification',
    description: 'Escrow payment, routing, validator result and reward release.',
  },
  {
    id: 'geometry',
    title: 'Anchor verified geometry + snapshot',
    description: '2-of-2 geometry attestation and a frozen spatial snapshot.',
  },
  {
    id: 'proof',
    title: 'Export verification proof',
    description: 'Download the on-chain state with every transaction signature.',
  },
  {
    id: 'dispute',
    title: 'Open dispute and freeze parcel',
    description: 'A second parcel is disputed, then frozen by two validator signatures.',
  },
]

export function isLifecycleStepId(value: string): value is LifecycleStepId {
  return LIFECYCLE_STEPS.some((step) => step.id === value)
}

// --- local session persistence --------------------------------------------

const SEED_KEY = 'terra.lifecycle.seed.v1'
const PROGRESS_KEY = 'terra.lifecycle.progress.v1'

export interface LifecycleTx {
  label: string
  signature: string
}

export interface LifecycleProgress {
  seed: string
  done: Partial<Record<LifecycleStepId, boolean>>
  txs: Partial<Record<LifecycleStepId, LifecycleTx[]>>
  updatedAt: number
}

function randomHex(bytesCount: number): string {
  const out = new Uint8Array(bytesCount)
  crypto.getRandomValues(out)
  return Array.from(out, (b) => b.toString(16).padStart(2, '0')).join('')
}

export function getOrCreateLifecycleSeed(): string {
  try {
    const stored = localStorage.getItem(SEED_KEY)
    if (stored && /^[0-9a-f]{32,128}$/.test(stored)) return stored
    const seed = randomHex(24)
    localStorage.setItem(SEED_KEY, seed)
    return seed
  } catch {
    return randomHex(24)
  }
}

export function loadLifecycleProgress(seed: string): LifecycleProgress {
  try {
    const raw = localStorage.getItem(PROGRESS_KEY)
    if (raw) {
      const parsed = JSON.parse(raw) as LifecycleProgress
      if (parsed.seed === seed) {
        return {
          seed,
          done: parsed.done ?? {},
          txs: parsed.txs ?? {},
          updatedAt: parsed.updatedAt ?? 0,
        }
      }
    }
  } catch {
    // fall through to a fresh record
  }
  return { seed, done: {}, txs: {}, updatedAt: 0 }
}

export function saveLifecycleProgress(progress: LifecycleProgress): void {
  try {
    localStorage.setItem(PROGRESS_KEY, JSON.stringify(progress))
  } catch {
    // local-only convenience; failing to persist must not fail the chain work
  }
}

export function resetLifecycleSession(): void {
  try {
    localStorage.removeItem(SEED_KEY)
    localStorage.removeItem(PROGRESS_KEY)
  } catch {
    // ignore
  }
}
