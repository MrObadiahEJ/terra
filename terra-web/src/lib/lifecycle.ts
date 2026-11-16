// Sprint B on-chain lifecycle runner. Every step builds instructions from the
// generated IDL and lands them on the configured RPC; there is no simulated or
// server-prepared path here.
import { AnchorProvider, BN, Program } from '@coral-xyz/anchor'
import type { AnchorWallet } from '@solana/wallet-adapter-react'
import {
  Connection,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
} from '@solana/web3.js'
import { TERRA_CLUSTER, TERRA_PROGRAM_ID } from './constants'
import idl from '../idl/terra_registry.json'
import type { TerraRegistry } from '../idl/terraRegistry'
import {
  attestationDigest,
  deriveLifecycleIds,
  deriveLifecyclePdas,
  deriveOperatorKeypairs,
  digest,
  type LifecycleIds,
  type LifecyclePdas,
  type LifecycleProgress,
  type LifecycleStepId,
  type LifecycleTx,
  type OperatorKeypairs,
} from './lifecyclePlan'

type Json = null | boolean | number | string | Json[] | { [key: string]: Json }

interface RegistryAccount {
  admin: PublicKey
  validators: PublicKey[]
}
interface ParcelAccount {
  status: number
  rightsCount: number
}
interface RightsAccount {
  holder: PublicKey
  rightsKind: number
}
interface ClaimAccount {
  status: number
  evidenceCount: number
  observationCount: number
  attestationCount: number
  requiredAttestations: number
}
interface TaskAccount {
  status: number
}
interface AssignmentAccount {
  status: number
}
interface AssetAccount {
  geometryVersionCount: number
}
interface GeometryVersionAccount {
  verified: boolean
  attestCount: number
  attestors: PublicKey[]
}
interface SnapshotMetaAccount {
  count: number
}
interface DisputeAccount {
  status: number
  presentCount: number
}
interface TaskEscrowAccount {
  status: number
  amount: BN
}
interface RewardAllocationAccount {
  basePaid: BN
}

interface AccountReader<T> {
  fetch(address: PublicKey): Promise<T>
  fetchNullable(address: PublicKey): Promise<T | null>
}

function programFor(connection: Connection, wallet: AnchorWallet): Program<TerraRegistry> {
  const provider = new AnchorProvider(connection, wallet, { commitment: 'confirmed' })
  return new Program<TerraRegistry>(idl as unknown as TerraRegistry, provider)
}

function reader<T>(program: Program<TerraRegistry>, name: string): AccountReader<T> {
  const acct = (program as any).account[name]
  return acct as AccountReader<T>
}

async function sendTransaction(
  connection: Connection,
  wallet: AnchorWallet,
  instructions: Transaction['instructions'],
  signers: { publicKey: PublicKey; secretKey: Uint8Array }[],
): Promise<string> {
  const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash('confirmed')
  const tx = new Transaction({ feePayer: wallet.publicKey, recentBlockhash: blockhash })
  tx.add(...instructions)
  if (signers.length > 0) tx.partialSign(...signers)

  const signed = await wallet.signTransaction(tx)
  const signature = await connection.sendRawTransaction(signed.serialize(), {
    skipPreflight: false,
    maxRetries: 3,
  })
  const confirmation = await connection.confirmTransaction(
    { signature, blockhash, lastValidBlockHeight },
    'confirmed',
  )
  if (confirmation.value.err) {
    throw new Error(`Transaction ${signature} failed: ${JSON.stringify(confirmation.value.err)}`)
  }
  return signature
}

async function ensureFunded(
  connection: Connection,
  keypair: { publicKey: PublicKey },
  minimumSol: number,
): Promise<string | null> {
  const balance = await connection.getBalance(keypair.publicKey)
  if (balance >= minimumSol * LAMPORTS_PER_SOL) return null
  const signature = await connection.requestAirdrop(
    keypair.publicKey,
    minimumSol * LAMPORTS_PER_SOL,
  )
  await connection.confirmTransaction(signature, 'confirmed')
  return signature
}

function hex(bytes: Uint8Array): string {
  return Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('')
}

function toJson(value: unknown): Json {
  if (value === null || value === undefined) return null
  if (typeof value === 'string' || typeof value === 'number' || typeof value === 'boolean') {
    return value
  }
  if (typeof value === 'bigint') return value.toString()
  if (BN.isBN(value as BN)) return (value as BN).toString()
  if (value instanceof PublicKey) return value.toBase58()
  if (value instanceof Uint8Array) return Array.from(value)
  if (Array.isArray(value)) return value.map(toJson)
  if (typeof value === 'object') {
    const out: Record<string, Json> = {}
    for (const [key, entry] of Object.entries(value)) out[key] = toJson(entry)
    return out
  }
  return String(value)
}

export interface LifecycleContext {
  connection: Connection
  wallet: AnchorWallet
  progress: LifecycleProgress
}

export interface LifecycleProof {
  schema: 'terra:verification-proof:v1'
  exportedAt: string
  network: string
  program: string
  ids: Record<string, string>
  pdas: Record<string, string>
  accounts: Record<string, Json>
  transactions: Partial<Record<LifecycleStepId, LifecycleTx[]>>
}

export interface LifecycleStepResult {
  txs: LifecycleTx[]
  proof?: LifecycleProof
}

interface StepState {
  program: Program<TerraRegistry>
  connection: Connection
  wallet: AnchorWallet
  admin: PublicKey
  ids: LifecycleIds
  pdas: LifecyclePdas
  operators: OperatorKeypairs
  txs: LifecycleTx[]
  allTxs: Partial<Record<LifecycleStepId, LifecycleTx[]>>
}

async function deriveOperatorKeys(seed: string) {
  return deriveOperatorKeypairs(seed)
}

async function tx(
  state: StepState,
  label: string,
  instructions: Transaction['instructions'],
  signers: { publicKey: PublicKey; secretKey: Uint8Array }[] = [],
): Promise<string> {
  const signature = await sendTransaction(state.connection, state.wallet, instructions, signers)
  state.txs.push({ label, signature })
  return signature
}

async function registryStep(state: StepState): Promise<void> {
  const registry = reader<RegistryAccount>(state.program, 'validatorRegistry')
  const existing = await registry.fetchNullable(state.pdas.registry)
  if (existing) {
    if (!existing.admin.equals(state.admin)) {
      throw new Error(
        `Registry already exists with admin ${existing.admin.toBase58()}. ` +
          `Connect that wallet or reset the local Surfnet.`,
      )
    }
    return
  }

  const ix = await state.program.methods
    .createRegistry()
    .accounts({
      registry: state.pdas.registry,
      admin: state.admin,
      systemProgram: SystemProgram.programId,
    } as never)
    .instruction()
  await tx(state, 'create_registry', [ix])
}

async function validatorsStep(state: StepState): Promise<void> {
  const registry = reader<RegistryAccount>(state.program, 'validatorRegistry')
  const registryAccount = await registry.fetch(state.pdas.registry)
  if (!registryAccount.admin.equals(state.admin)) {
    throw new Error('Connected wallet is not the registry admin')
  }

  const profiles = reader<unknown>(state.program, 'validatorProfile')
  const availabilities = reader<unknown>(state.program, 'validatorAvailability')
  const reputations = reader<unknown>(state.program, 'validatorReputation')

  for (const [label, validator] of [
    ['validator-one', state.operators.validatorOne],
    ['validator-two', state.operators.validatorTwo],
  ] as const) {
    const airdrop = await ensureFunded(state.connection, validator, 20)
    if (airdrop) state.txs.push({ label: `airdrop_${label}`, signature: airdrop })

    if (!registryAccount.validators.some((key) => key.equals(validator.publicKey))) {
      const ix = await state.program.methods
        .addValidatorToRegistry(validator.publicKey)
        .accounts({
          registry: state.pdas.registry,
          adminSigner: state.admin,
          endorsement: state.pdas.endorsement(validator.publicKey),
          validator: validator.publicKey,
          systemProgram: SystemProgram.programId,
        } as never)
        .instruction()
      await tx(state, `add_validator_${label}`, [ix])
    }

    if (!(await profiles.fetchNullable(state.pdas.profile(validator.publicKey)))) {
      const identityHash = await digest([`terra:lifecycle:validator:${label}`])
      const ix = await state.program.methods
        .initValidatorProfile(Array.from(identityHash), 'Terra lifecycle operator validator')
        .accounts({
          profile: state.pdas.profile(validator.publicKey),
          wallet: validator.publicKey,
          systemProgram: SystemProgram.programId,
        } as never)
        .instruction()
      await tx(state, `init_profile_${label}`, [ix], [validator])
    }

    if (!(await availabilities.fetchNullable(state.pdas.availability(validator.publicKey)))) {
      const ix = await state.program.methods
        .setValidatorAvailability(0) // AVAILABILITY_ONLINE
        .accounts({
          availability: state.pdas.availability(validator.publicKey),
          profile: state.pdas.profile(validator.publicKey),
          wallet: validator.publicKey,
          systemProgram: SystemProgram.programId,
        } as never)
        .instruction()
      await tx(state, `set_availability_${label}`, [ix], [validator])
    }

    if (!(await reputations.fetchNullable(state.pdas.reputation(validator.publicKey)))) {
      const ix = await state.program.methods
        .initializeValidatorReputation()
        .accounts({
          reputation: state.pdas.reputation(validator.publicKey),
          validator: validator.publicKey,
          registry: state.pdas.registry,
          payer: state.admin,
          systemProgram: SystemProgram.programId,
        } as never)
        .instruction()
      await tx(state, `init_reputation_${label}`, [ix])
    }
  }
}

async function parcelStep(state: StepState): Promise<void> {
  const parcels = reader<ParcelAccount>(state.program, 'parcel')
  const rights = reader<RightsAccount>(state.program, 'rights')

  if (!(await parcels.fetchNullable(state.pdas.parcel))) {
    const geometryHash = await digest(['terra:lifecycle:geometry:v0'])
    const ix = await state.program.methods
      .registerParcel(
        Array.from(state.ids.parcelId),
        'Terra lifecycle parcel',
        Array.from(geometryHash),
      )
      .accounts({
        parcel: state.pdas.parcel,
        owner: state.admin,
        ownership: state.pdas.ownership,
        systemProgram: SystemProgram.programId,
      } as never)
      .instruction()
    await tx(state, 'register_parcel', [ix])
  }

  if (!(await rights.fetchNullable(state.pdas.usageRight))) {
    const ix = await state.program.methods
      .grantRight(
        0,
        1, // RIGHTS.USAGE
        state.operators.tenant.publicKey,
        new BN(0),
        'Terra lifecycle usage right',
      )
      .accounts({
        parcel: state.pdas.parcel,
        ownership: state.pdas.ownership,
        rights: state.pdas.usageRight,
        owner: state.admin,
        systemProgram: SystemProgram.programId,
      } as never)
      .instruction()
    await tx(state, 'grant_usage_right', [ix])
  }
}

async function claimStep(state: StepState): Promise<void> {
  const claims = reader<ClaimAccount>(state.program, 'claim')
  const evidence = reader<unknown>(state.program, 'evidence')
  const observations = reader<unknown>(state.program, 'observation')

  if (!(await claims.fetchNullable(state.pdas.claim))) {
    const statementHash = await digest(['terra:lifecycle:claim:boundary'])
    const ix = await state.program.methods
      .createClaim(
        Array.from(state.ids.claimId),
        1, // CLAIM_TYPE.BOUNDARY
        Array.from(statementHash),
        0,
        [0, 0],
      )
      .accounts({
        claim: state.pdas.claim,
        parcel: state.pdas.parcel,
        submitter: state.admin,
        systemProgram: SystemProgram.programId,
      } as never)
      .instruction()
    await tx(state, 'create_claim', [ix])
  }

  if (!(await evidence.fetchNullable(state.pdas.evidence))) {
    const contentHash = await digest(['terra:lifecycle:evidence:survey'])
    const observedAt = Math.floor(Date.now() / 1000)
    const ix = await state.program.methods
      .addEvidence(
        0, // PERSON_SUBMISSION
        Array.from(contentHash),
        'ipfs://terra-lifecycle/evidence-0',
        new BN(observedAt),
      )
      .accounts({
        evidence: state.pdas.evidence,
        claim: state.pdas.claim,
        submitter: state.admin,
        systemProgram: SystemProgram.programId,
      } as never)
      .instruction()
    await tx(state, 'add_evidence', [ix])
  }

  for (const [label, validator] of [
    ['validator-one', state.operators.validatorOne],
    ['validator-two', state.operators.validatorTwo],
  ] as const) {
    if (await observations.fetchNullable(state.pdas.observation(validator.publicKey))) continue
    const findings = await digest([`terra:lifecycle:observation:${label}`])
    const signature = await digest([`terra:lifecycle:observation-signature:${label}`])
    const ix = await state.program.methods
      .submitObservation(
        [new BN(-6_200_000_000), new BN(106_800_000_000)],
        0,
        Array.from(findings),
        90,
        Array.from(signature),
      )
      .accounts({
        observation: state.pdas.observation(validator.publicKey),
        claim: state.pdas.claim,
        validator: validator.publicKey,
        systemProgram: SystemProgram.programId,
      } as never)
      .instruction()
    await tx(state, `submit_observation_${label}`, [ix], [validator])
  }
}

async function attestationsStep(state: StepState): Promise<void> {
  const claims = reader<ClaimAccount>(state.program, 'claim')
  const attestations = reader<unknown>(state.program, 'verificationAttestation')

  for (const [label, validator] of [
    ['validator-one', state.operators.validatorOne],
    ['validator-two', state.operators.validatorTwo],
  ] as const) {
    if (await attestations.fetchNullable(state.pdas.attestation(validator.publicKey))) continue
    const signature = await attestationDigest(
      state.pdas.claim,
      validator.publicKey,
      state.pdas.observation(validator.publicKey),
      0, // CONFIRMED
      95,
    )
    const ix = await state.program.methods
      .submitVerificationAttestation(0, 95, Array.from(signature))
      .accounts({
        attestation: state.pdas.attestation(validator.publicKey),
        claim: state.pdas.claim,
        observation: state.pdas.observation(validator.publicKey),
        validator: validator.publicKey,
        systemProgram: SystemProgram.programId,
      } as never)
      .remainingAccounts([
        {
          pubkey: state.pdas.reputation(validator.publicKey),
          isWritable: false,
          isSigner: false,
        },
      ])
      .instruction()
    await tx(state, `submit_attestation_${label}`, [ix], [validator])
  }

  const claim = await claims.fetch(state.pdas.claim)
  if (claim.status !== 2) {
    const ix = await state.program.methods
      .verifyClaim()
      .accounts({ claim: state.pdas.claim } as never)
      .instruction()
    await tx(state, 'verify_claim', [ix])
  }
}

async function taskStep(state: StepState): Promise<void> {
  const tasks = reader<TaskAccount>(state.program, 'verificationTask')
  const requirements = reader<unknown>(state.program, 'taskRequirement')
  const feePolicies = reader<unknown>(state.program, 'feePolicy')
  const quotes = reader<unknown>(state.program, 'resourceQuote')
  const coverage = reader<unknown>(state.program, 'coverageIncentive')
  const escrows = reader<TaskEscrowAccount>(state.program, 'taskEscrow')
  const assignments = reader<AssignmentAccount>(state.program, 'taskAssignment')
  const rewards = reader<RewardAllocationAccount>(state.program, 'rewardAllocation')
  const validator = state.operators.validatorOne
  const assignmentPda = state.pdas.assignment(validator.publicKey)

  if (!(await tasks.fetchNullable(state.pdas.task))) {
    const deadline = Math.floor(Date.now() / 1000) + 3_600
    const descriptionHash = await digest(['terra:lifecycle:task:boundary'])
    const ix = await state.program.methods
      .createVerificationTask(
        Array.from(state.ids.taskId),
        state.pdas.claim,
        1, // TASK_CLASS.REMOTE
        new BN(100_000_000),
        new BN(deadline),
        Array.from(descriptionHash),
        1,
      )
      .accounts({
        task: state.pdas.task,
        requester: state.admin,
        systemProgram: SystemProgram.programId,
      } as never)
      .instruction()
    await tx(state, 'create_verification_task', [ix])
  }

  if (!(await requirements.fetchNullable(state.pdas.requirement))) {
    const ix = await state.program.methods
      .addTaskRequirement(0, 255, 0, 0, [0, 0], 0, 0, 0, 0, 9_000)
      .accounts({
        requirement: state.pdas.requirement,
        task: state.pdas.task,
        requester: state.admin,
        systemProgram: SystemProgram.programId,
      } as never)
      .instruction()
    await tx(state, 'add_task_requirement', [ix])
  }

  if (!(await feePolicies.fetchNullable(state.pdas.feePolicy))) {
    const ix = await state.program.methods
      .setFeePolicy(100)
      .accounts({
        feePolicy: state.pdas.feePolicy,
        registry: state.pdas.registry,
        admin: state.admin,
        systemProgram: SystemProgram.programId,
      } as never)
      .instruction()
    await tx(state, 'set_fee_policy', [ix])
  }

  if (!(await quotes.fetchNullable(state.pdas.quote))) {
    const ix = await state.program.methods
      .quoteTaskResources(new BN(10_000_000))
      .accounts({
        task: state.pdas.task,
        quote: state.pdas.quote,
        quoter: state.admin,
        systemProgram: SystemProgram.programId,
      } as never)
      .instruction()
    await tx(state, 'quote_task_resources', [ix])
  }

  if (!(await coverage.fetchNullable(state.pdas.coverageIncentive))) {
    const ix = await state.program.methods
      .setCoverageIncentive(100, 100, 100, 100, 500)
      .accounts({
        coverageIncentive: state.pdas.coverageIncentive,
        registry: state.pdas.registry,
        admin: state.admin,
        systemProgram: SystemProgram.programId,
      } as never)
      .instruction()
    await tx(state, 'set_coverage_incentive', [ix])
  }

  if (!(await escrows.fetchNullable(state.pdas.taskEscrow))) {
    const ix = await state.program.methods
      .fundTaskEscrow()
      .accounts({
        task: state.pdas.task,
        quote: state.pdas.quote,
        feePolicy: state.pdas.feePolicy,
        taskEscrow: state.pdas.taskEscrow,
        vault: state.pdas.taskVault,
        treasury: state.pdas.treasury,
        requester: state.admin,
        systemProgram: SystemProgram.programId,
      } as never)
      .instruction()
    await tx(state, 'fund_task_escrow', [ix])
  }

  let assignment = await assignments.fetchNullable(assignmentPda)
  if (!assignment) {
    const ix = await state.program.methods
      .routeTask(Array.from(state.ids.taskId), 0, [validator.publicKey], validator.publicKey, 1)
      .accounts({
        assignment: assignmentPda,
        task: state.pdas.task,
        requirement: state.pdas.requirement,
        chosen: validator.publicKey,
        requester: state.admin,
        systemProgram: SystemProgram.programId,
      } as never)
      .remainingAccounts([
        { pubkey: state.pdas.profile(validator.publicKey), isWritable: false, isSigner: false },
        {
          pubkey: state.pdas.availability(validator.publicKey),
          isWritable: false,
          isSigner: false,
        },
        { pubkey: state.pdas.reputation(validator.publicKey), isWritable: false, isSigner: false },
      ])
      .instruction()
    await tx(state, 'route_task', [ix])
    assignment = await assignments.fetch(assignmentPda)
  }

  if (assignment.status === 0) {
    const resultHash = await digest(['terra:lifecycle:task:result:pass'])
    const ix = await state.program.methods
      .submitTaskResult(Array.from(state.ids.taskId), 1, Array.from(resultHash))
      .accounts({
        assignment: assignmentPda,
        task: state.pdas.task,
        validator: validator.publicKey,
      } as never)
      .instruction()
    await tx(state, 'submit_task_result', [ix], [validator])
    assignment = await assignments.fetch(assignmentPda)
  }

  if (assignment.status === 1) {
    const ix = await state.program.methods
      .claimTaskReward()
      .accounts({
        task: state.pdas.task,
        assignment: assignmentPda,
        taskEscrow: state.pdas.taskEscrow,
        vault: state.pdas.taskVault,
        coverageIncentive: state.pdas.coverageIncentive,
        rewardAllocation: state.pdas.rewardAllocation(validator.publicKey),
        treasury: state.pdas.treasury,
        validator: validator.publicKey,
        systemProgram: SystemProgram.programId,
      } as never)
      .instruction()
    await tx(state, 'claim_task_reward', [ix], [validator])
  }

  const escrow = await escrows.fetch(state.pdas.taskEscrow)
  if (escrow.status !== 2) throw new Error('Task escrow did not reach the released state')
  const reward = await rewards.fetch(state.pdas.rewardAllocation(validator.publicKey))
  if (reward.basePaid.isZero()) throw new Error('Reward allocation is empty after payout')
}

async function geometryStep(state: StepState): Promise<void> {
  const assets = reader<AssetAccount>(state.program, 'spatialAsset')
  const versions = reader<GeometryVersionAccount>(state.program, 'geometryVersion')
  const metas = reader<SnapshotMetaAccount>(state.program, 'spatialSnapshotMeta')
  const snapshots = reader<unknown>(state.program, 'spatialSnapshot')

  if (!(await assets.fetchNullable(state.pdas.asset))) {
    const ix = await state.program.methods
      .initSpatialAsset(0, 0, 0)
      .accounts({
        asset: state.pdas.asset,
        parcel: state.pdas.parcel,
        registrar: state.admin,
        systemProgram: SystemProgram.programId,
      } as never)
      .instruction()
    await tx(state, 'init_spatial_asset', [ix])
  }

  if (!(await versions.fetchNullable(state.pdas.geometryVersion))) {
    const geometryHash = await digest(['terra:lifecycle:geometry:verified:v0'])
    const ix = await state.program.methods
      .appendGeometryVersion(
        Array.from(geometryHash),
        0,
        0,
        'ipfs://terra-lifecycle/geometry-v0',
        0,
        0,
        0,
      )
      .accounts({
        asset: state.pdas.asset,
        geometryVersion: state.pdas.geometryVersion,
        payer: state.admin,
        systemProgram: SystemProgram.programId,
      } as never)
      .instruction()
    await tx(state, 'append_geometry_version', [ix])
  }

  let version = await versions.fetch(state.pdas.geometryVersion)
  for (const [label, validator] of [
    ['validator-one', state.operators.validatorOne],
    ['validator-two', state.operators.validatorTwo],
  ] as const) {
    const alreadyAttested = version.attestors
      .slice(0, Math.min(version.attestCount, version.attestors.length))
      .some((key) => key.equals(validator.publicKey))
    if (alreadyAttested || version.verified) continue
    const ix = await state.program.methods
      .verifyGeometryVersion()
      .accounts({
        geometryVersion: state.pdas.geometryVersion,
        asset: state.pdas.asset,
        profile: state.pdas.profile(validator.publicKey),
        validator: validator.publicKey,
      } as never)
      .instruction()
    await tx(state, `verify_geometry_${label}`, [ix], [validator])
    version = await versions.fetch(state.pdas.geometryVersion)
  }
  if (!version.verified) throw new Error('Geometry version did not reach 2-of-2 verification')

  const meta = await metas.fetchNullable(state.pdas.snapshotMeta)
  if ((!meta || meta.count === 0) && !(await snapshots.fetchNullable(state.pdas.snapshot))) {
    const ix = await state.program.methods
      .createSpatialSnapshot('Terra lifecycle baseline')
      .accounts({
        asset: state.pdas.asset,
        meta: state.pdas.snapshotMeta,
        snapshot: state.pdas.snapshot,
        payer: state.admin,
        systemProgram: SystemProgram.programId,
      } as never)
      .instruction()
    await tx(state, 'create_spatial_snapshot', [ix])
  }
}

async function disputeStep(state: StepState): Promise<void> {
  const parcels = reader<ParcelAccount>(state.program, 'parcel')
  const disputes = reader<DisputeAccount>(state.program, 'dispute')
  const rights = reader<RightsAccount>(state.program, 'rights')

  if (!(await parcels.fetchNullable(state.pdas.disputeParcel))) {
    const geometryHash = await digest(['terra:lifecycle:dispute:geometry'])
    const ix = await state.program.methods
      .registerParcel(
        Array.from(state.ids.disputeParcelId),
        'Terra lifecycle dispute parcel',
        Array.from(geometryHash),
      )
      .accounts({
        parcel: state.pdas.disputeParcel,
        owner: state.admin,
        ownership: state.pdas.disputeOwnership,
        systemProgram: SystemProgram.programId,
      } as never)
      .instruction()
    await tx(state, 'register_dispute_parcel', [ix])
  }

  const v1 = state.operators.validatorOne
  const v2 = state.operators.validatorTwo
  const validators = [
    v1.publicKey,
    v2.publicKey,
    ...Array.from({ length: 6 }, () => PublicKey.default),
  ]

  if (!(await disputes.fetchNullable(state.pdas.dispute))) {
    const ix = await state.program.methods
      .fileDispute(Array.from(state.ids.caseHash), 2, validators)
      .accounts({
        dispute: state.pdas.dispute,
        parcel: state.pdas.disputeParcel,
        ownership: state.pdas.disputeOwnership,
        registry: state.pdas.registry,
        filer: state.admin,
        systemProgram: SystemProgram.programId,
      } as never)
      .instruction()
    await tx(state, 'file_dispute', [ix])
  }

  const dispute = await disputes.fetch(state.pdas.dispute)
  if (dispute.status === 0) {
    const ix = await state.program.methods
      .freezeParcel()
      .accounts({
        dispute: state.pdas.dispute,
        parcel: state.pdas.disputeParcel,
        validator: v1.publicKey,
      } as never)
      .remainingAccounts([
        { pubkey: v1.publicKey, isWritable: false, isSigner: true },
        { pubkey: v2.publicKey, isWritable: false, isSigner: true },
      ])
      .instruction()
    await tx(state, 'freeze_parcel', [ix], [v1, v2])
  }

  const frozen = await disputes.fetch(state.pdas.dispute)
  if (frozen.status !== 1) throw new Error('Dispute did not reach the FROZEN state')
  const parcel = await parcels.fetch(state.pdas.disputeParcel)
  if (parcel.status !== 5) throw new Error('Parcel did not reach the FROZEN status')
  const ownership = await rights.fetch(state.pdas.disputeOwnership)
  if (!ownership.holder.equals(state.admin)) throw new Error('Dispute parcel ownership changed')
}

async function proofStep(state: StepState): Promise<LifecycleProof> {
  const fetch = async <T>(name: string, address: PublicKey): Promise<Json> => {
    const value = await reader<T>(state.program, name).fetchNullable(address)
    return toJson(value)
  }

  const accounts: Record<string, Json> = {
    parcel: await fetch<ParcelAccount>('parcel', state.pdas.parcel),
    ownership: await fetch<RightsAccount>('rights', state.pdas.ownership),
    usageRight: await fetch<RightsAccount>('rights', state.pdas.usageRight),
    claim: await fetch<ClaimAccount>('claim', state.pdas.claim),
    evidence: await fetch<unknown>('evidence', state.pdas.evidence),
    observationOne: await fetch<unknown>(
      'observation',
      state.pdas.observation(state.operators.validatorOne.publicKey),
    ),
    observationTwo: await fetch<unknown>(
      'observation',
      state.pdas.observation(state.operators.validatorTwo.publicKey),
    ),
    attestationOne: await fetch<unknown>(
      'verificationAttestation',
      state.pdas.attestation(state.operators.validatorOne.publicKey),
    ),
    attestationTwo: await fetch<unknown>(
      'verificationAttestation',
      state.pdas.attestation(state.operators.validatorTwo.publicKey),
    ),
    task: await fetch<TaskAccount>('verificationTask', state.pdas.task),
    assignment: await fetch<AssignmentAccount>(
      'taskAssignment',
      state.pdas.assignment(state.operators.validatorOne.publicKey),
    ),
    taskEscrow: await fetch<TaskEscrowAccount>('taskEscrow', state.pdas.taskEscrow),
    rewardAllocation: await fetch<RewardAllocationAccount>(
      'rewardAllocation',
      state.pdas.rewardAllocation(state.operators.validatorOne.publicKey),
    ),
    spatialAsset: await fetch<AssetAccount>('spatialAsset', state.pdas.asset),
    geometryVersion: await fetch<GeometryVersionAccount>(
      'geometryVersion',
      state.pdas.geometryVersion,
    ),
    spatialSnapshot: await fetch<unknown>('spatialSnapshot', state.pdas.snapshot),
    disputeParcel: await fetch<ParcelAccount>('parcel', state.pdas.disputeParcel),
    dispute: await fetch<DisputeAccount>('dispute', state.pdas.dispute),
  }

  return {
    schema: 'terra:verification-proof:v1',
    exportedAt: new Date().toISOString(),
    network: TERRA_CLUSTER,
    program: TERRA_PROGRAM_ID,
    ids: {
      parcel: hex(state.ids.parcelId),
      claim: hex(state.ids.claimId),
      task: hex(state.ids.taskId),
      disputeParcel: hex(state.ids.disputeParcelId),
      disputeCase: hex(state.ids.caseHash),
    },
    pdas: {
      registry: state.pdas.registry.toBase58(),
      parcel: state.pdas.parcel.toBase58(),
      ownership: state.pdas.ownership.toBase58(),
      claim: state.pdas.claim.toBase58(),
      evidence: state.pdas.evidence.toBase58(),
      task: state.pdas.task.toBase58(),
      taskEscrow: state.pdas.taskEscrow.toBase58(),
      spatialAsset: state.pdas.asset.toBase58(),
      geometryVersion: state.pdas.geometryVersion.toBase58(),
      spatialSnapshot: state.pdas.snapshot.toBase58(),
      disputeParcel: state.pdas.disputeParcel.toBase58(),
      dispute: state.pdas.dispute.toBase58(),
    },
    accounts,
    transactions: state.allTxs,
  }
}

/** Run one lifecycle step, skipping writes that already exist on chain. */
export async function runLifecycleStep(
  step: LifecycleStepId,
  context: LifecycleContext,
): Promise<LifecycleStepResult> {
  const ids = await deriveLifecycleIds(context.progress.seed)
  const pdas = deriveLifecyclePdas(ids)
  const operators = await deriveOperatorKeys(context.progress.seed)
  const program = programFor(context.connection, context.wallet)
  const state: StepState = {
    program,
    connection: context.connection,
    wallet: context.wallet,
    admin: context.wallet.publicKey,
    ids,
    pdas,
    operators,
    txs: [],
    allTxs: context.progress.txs,
  }

  switch (step) {
    case 'registry':
      await registryStep(state)
      break
    case 'validators':
      await validatorsStep(state)
      break
    case 'parcel':
      await parcelStep(state)
      break
    case 'claim':
      await claimStep(state)
      break
    case 'attestations':
      await attestationsStep(state)
      break
    case 'task':
      await taskStep(state)
      break
    case 'geometry':
      await geometryStep(state)
      break
    case 'proof':
      return { txs: [], proof: await proofStep(state) }
    case 'dispute':
      await disputeStep(state)
      break
  }

  return { txs: state.txs }
}

/** Proof export before a dispute, with dispute accounts marked null. */
export async function exportLifecycleProof(context: LifecycleContext): Promise<LifecycleProof> {
  return (await runLifecycleStep('proof', context)).proof as LifecycleProof
}

export function proofFileName(proof: LifecycleProof): string {
  return `terra-proof-${proof.ids.parcel.slice(0, 12)}.json`
}

export function proofBlob(proof: LifecycleProof): Blob {
  return new Blob([JSON.stringify(proof, null, 2)], { type: 'application/json' })
}
