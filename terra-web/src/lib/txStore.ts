// Demo transaction feed — a locally simulated Solana-style explorer.
//
// Every Lab experiment, demo registration and the /transactions composer emit
// entries here, so the frontend shows realistic signatures, program logs,
// accounts, fees and compute units without a wallet, RPC or devnet.
import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import { TERRA_PROGRAM_ID } from './constants'

export type TxStatus = 'confirmed' | 'failed'

export interface TxAccount {
  pubkey: string
  role: 'signer' | 'writable' | 'readonly'
}

export interface DemoTx {
  sig: string
  slot: number
  blockTime: number
  status: TxStatus
  program: string
  instruction: string
  summary: string
  feeLamports: number
  computeUnits: number
  signer: string
  accounts: TxAccount[]
  logs: string[]
  error?: { code: string | null; name: string }
}

const B58 = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz'

/** Base58-encode raw bytes (no deps — mirrors Solana's address alphabet). */
function b58encode(bytes: Uint8Array): string {
  let zeros = 0
  while (zeros < bytes.length && bytes[zeros] === 0) zeros++
  const digits = [0]
  for (const byte of bytes) {
    let carry = byte
    for (let i = 0; i < digits.length; i++) {
      const v = digits[i] * 256 + carry
      digits[i] = v % 58
      carry = Math.floor(v / 58)
    }
    while (carry > 0) {
      digits.push(carry % 58)
      carry = Math.floor(carry / 58)
    }
  }
  return '1'.repeat(zeros) + digits.reverse().map((d) => B58[d]).join('')
}

function randomBytes(n: number): Uint8Array {
  const b = new Uint8Array(n)
  if (typeof crypto !== 'undefined' && crypto.getRandomValues) crypto.getRandomValues(b)
  else for (let i = 0; i < n; i++) b[i] = Math.floor(Math.random() * 256)
  return b
}

/** A demo 64-byte transaction signature (88-ish base58 chars, like mainnet). */
export function fakeSignature(): string {
  return b58encode(randomBytes(64))
}

/** A demo 32-byte public key / PDA. */
export function fakePubkey(): string {
  return b58encode(randomBytes(32))
}

export const DEMO_SIGNER = 'demo111111111111111111111111111111111111111'

// Stable-ish demo PDAs used across seeds, composer and Lab reports.
export const DEMO_ACCOUNTS = {
  parcelPda: fakePubkey(),
  spatialPda: fakePubkey(),
  claimPda: fakePubkey(),
  bindingPda: fakePubkey(),
  profilePda: fakePubkey(),
}

/** Parse "ErrorName (6236): …" the exact TerraError phrasing used everywhere. */
export function parseTerraError(summary: string): { code: string; name: string } | undefined {
  const m = /\b([A-Za-z][A-Za-z0-9_]*)\s*\((\d{4})\)/.exec(summary)
  if (!m) return undefined
  return { name: m[1], code: m[2] }
}

const IX_TITLE: Record<string, string> = {
  register_parcel: 'RegisterParcel',
  init_spatial_asset: 'InitSpatialAsset',
  append_geometry_version: 'AppendGeometryVersion',
  verify_geometry_version: 'VerifyGeometryVersion',
  create_claim: 'CreateClaim',
  add_evidence_artifact: 'AddEvidenceArtifact',
  submit_observation_v2: 'SubmitObservationV2',
  submit_attestation: 'SubmitAttestation',
  verify_claim: 'VerifyClaim',
  file_challenge: 'FileChallenge',
  record_challenge_vote: 'RecordChallengeVote',
  record_cross_border_verification: 'RecordCrossBorderVerification',
  transfer_rights: 'TransferRights',
  set_validator_jurisdiction: 'SetValidatorJurisdiction',
}

const IX_OK_LOG: Record<string, string> = {
  register_parcel: 'parcel status -> Registered',
  init_spatial_asset: 'SpatialAsset PDA ["spatial_asset", parcel] initialized',
  append_geometry_version: 'GeometryVersion cursor incremented',
  verify_geometry_version: 'geometry version marked verified',
  verify_claim: 'claim status -> VERIFIED',
  record_cross_border_verification: 'CrossBorderSpanRecord written',
}

function accountsFor(instruction: string, signer: string): TxAccount[] {
  const program: TxAccount = { pubkey: TERRA_PROGRAM_ID, role: 'readonly' }
  const spatial = ['init_spatial_asset', 'append_geometry_version', 'verify_geometry_version']
  const claims = [
    'create_claim',
    'add_evidence_artifact',
    'submit_observation_v2',
    'submit_attestation',
    'verify_claim',
    'file_challenge',
    'record_challenge_vote',
  ]
  const pdas: TxAccount[] = []
  if (spatial.includes(instruction)) {
    pdas.push({ pubkey: DEMO_ACCOUNTS.spatialPda, role: 'writable' })
    pdas.push({ pubkey: DEMO_ACCOUNTS.parcelPda, role: 'readonly' })
  } else if (claims.includes(instruction)) {
    pdas.push({ pubkey: DEMO_ACCOUNTS.claimPda, role: 'writable' })
  } else if (instruction === 'record_cross_border_verification') {
    pdas.push({ pubkey: DEMO_ACCOUNTS.bindingPda, role: 'writable' })
    pdas.push({ pubkey: DEMO_ACCOUNTS.profilePda, role: 'readonly' })
  } else {
    pdas.push({ pubkey: DEMO_ACCOUNTS.parcelPda, role: 'writable' })
  }
  return [{ pubkey: signer, role: 'signer' }, ...pdas, program]
}

interface TxState {
  txs: DemoTx[]
  push: (tx: DemoTx) => void
  clear: () => void
  reseed: () => void
}

const MAX_TXS = 200

function makeSeeds(): DemoTx[] {
  const now = Date.now()
  const rows: Array<[string, boolean, string, number]> = [
    ['register_parcel', true, 'Soa Demo Block A · 589324 m² · status = Registered', 64],
    ['init_spatial_asset', true, 'SpatialAssetCreated — PDA ["spatial_asset", parcel] initialized', 58],
    ['append_geometry_version', true, 'GeometryVersion 0 anchored — source = SURVEY (1) · digest 5f20a92f24f0545f…', 51],
    ['verify_geometry_version', false, 'GeometryAlreadyVerified (6236): version 0 already verified by validator', 44],
    ['submit_observation_v2', true, 'observation_count = 1 → claim.status = UNDER_VERIFICATION', 37],
    ['submit_attestation', true, 'validator=1 result=CONFIRMED confidence=95 → attestation_count=1/2', 30],
    ['verify_claim', true, 'attestation_count=2/2 → claim.status = VERIFIED (version bumped)', 22],
    ['record_cross_border_verification', false, 'Rejected — SameJurisdiction (6227): CM → CM', 17],
    ['transfer_rights', true, 'rights transferred — new holder demo111…', 11],
    ['set_validator_jurisdiction', true, 'ValidatorProfile.jurisdiction = NG (bytes 78, 71)', 6],
    ['file_challenge', true, 'required_votes=2 review_deadline +14d → claim.status = CHALLENGED', 2],
  ]
  return rows.map(([instruction, ok, summary, minutesAgo], i) => {
    const err = ok ? undefined : parseTerraError(summary)
    const cu = ok ? 9000 + ((i * 4173) % 31000) : 6000 + ((i * 991) % 12000)
    return {
      sig: fakeSignature(),
      slot: 284_112_041 + i * 3,
      blockTime: now - minutesAgo * 60_000,
      status: (ok ? 'confirmed' : 'failed') as TxStatus,
      program: TERRA_PROGRAM_ID,
      instruction,
      summary,
      feeLamports: 5000,
      computeUnits: cu,
      signer: DEMO_SIGNER,
      accounts: accountsFor(instruction, DEMO_SIGNER),
      logs: buildLogs(instruction, ok, summary, cu, err),
      error: err ? { code: err.code, name: err.name } : undefined,
    }
  })
}

function buildLogs(
  instruction: string,
  ok: boolean,
  summary: string,
  cu: number,
  err?: { code: string; name: string },
): string[] {
  const title = IX_TITLE[instruction] ?? instruction
  const lines = [`Program log: Instruction: ${title}`]
  if (ok) {
    lines.push(`Program log: ${IX_OK_LOG[instruction] ?? 'instruction ok'}`)
    if (summary.length <= 96 && !summary.startsWith('Program')) {
      lines.push(`Program log: ${summary}`)
    }
    lines.push(`Program ${TERRA_PROGRAM_ID} consumed ${cu} of 200000 compute units`)
    lines.push(`Program ${TERRA_PROGRAM_ID} success`)
  } else if (err) {
    lines.push(
      `Program log: AnchorError occurred. Error Code: ${err.name}. Error Number: ${err.code}. Error Message: see TerraError variant.`,
    )
    lines.push(`Program ${TERRA_PROGRAM_ID} consumed ${cu} of 200000 compute units`)
    lines.push(
      `Program ${TERRA_PROGRAM_ID} failed: custom program error: 0x${Number(err.code).toString(16).toUpperCase()}`,
    )
  } else {
    const name = /\b([A-Za-z][A-Za-z0-9_]*)\b/.exec(summary)?.[1] ?? 'TransactionError'
    lines.push(`Program log: simulation failed before execution`)
    lines.push(`Transaction simulation failed: ${name}`)
  }
  return lines
}

export const useTxStore = create<TxState>()(
  persist(
    (set, get) => ({
      txs: makeSeeds(),
      push: (tx) => {
        const next = [tx, ...get().txs].slice(0, MAX_TXS)
        set({ txs: next })
      },
      clear: () => set({ txs: [] }),
      reseed: () => set({ txs: makeSeeds() }),
    }),
    { name: 'terra.tx.v1' },
  ),
)

export interface ReportOpts {
  signer?: string
  sig?: string
  instructionLabel?: string
}

/**
 * Build + publish one demo transaction. `summary` may carry the exact
 * TerraError phrasing ("QuorumNotReached (6137): …") — the code is parsed
 * out and rendered as an Anchor-style failed log.
 */
export function reportTx(instruction: string, ok: boolean, summary: string, opts: ReportOpts = {}): DemoTx {
  const state = useTxStore.getState()
  const signer = opts.signer ?? DEMO_SIGNER
  const err = ok ? undefined : parseTerraError(summary)
  const computeUnits = ok ? 8000 + Math.floor(Math.random() * 36000) : 5000 + Math.floor(Math.random() * 14000)
  const lastSlot = state.txs[0]?.slot ?? 284_112_040
  const tx: DemoTx = {
    sig: opts.sig ?? fakeSignature(),
    slot: lastSlot + 1 + Math.floor(Math.random() * 2),
    blockTime: Date.now(),
    status: ok ? 'confirmed' : 'failed',
    program: TERRA_PROGRAM_ID,
    instruction,
    summary,
    feeLamports: 5000,
    computeUnits,
    signer,
    accounts: accountsFor(instruction, signer),
    logs: buildLogs(instruction, ok, summary, computeUnits, err),
    error: err ? { code: err.code, name: err.name } : undefined,
  }
  state.push(tx)
  return tx
}
