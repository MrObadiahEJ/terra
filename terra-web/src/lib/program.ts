import { Program, AnchorProvider } from '@coral-xyz/anchor'
import { PublicKey } from '@solana/web3.js'
import { TERRA_PROGRAM_ID } from './constants'
import { connection, getActiveSigner } from './wallet'
import { strToBytes } from './codec'
import idl from '../idl/terra_registry.json'
import type { TerraRegistry } from '../idl/terraRegistry'

// Anchor account interfaces (decoded from the on-chain program).
export interface ParcelAccount {
  id: number[]
  owner: PublicKey
  name: string
  geometryHash: number[]
  status: number
  rightsCount: number
  infrastructureFlags: number
  accessHash: number[]
  createdAt: bigint | { toString(): string }
  updatedAt: bigint | { toString(): string }
}

export interface RightsAccount {
  parcel: PublicKey
  rightsKind: number
  holder: PublicKey
  granter: PublicKey
  createdAt: bigint | { toString(): string }
  expiresAt: bigint | { toString(): string }
  notes: string
}

let programSingleton: Program<TerraRegistry> | null = null

function getProvider(): AnchorProvider {
  const signer = getActiveSigner()
  if (!signer) {
    throw new Error('Wallet not connected')
  }
  return new AnchorProvider(connection, signer, { commitment: 'confirmed' })
}

// Program is built lazily once a wallet is connected (Anchor needs a payer).
// The program id is read from the IDL address field.
export function getProgram(): Program<TerraRegistry> {
  if (!programSingleton) {
    programSingleton = new Program<TerraRegistry>(
      idl as unknown as TerraRegistry,
      getProvider(),
    )
  }
  return programSingleton
}

// Derive the parcel PDA from its 32-byte id.
export function parcelPda(id: Uint8Array): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [strToBytes('parcel'), id],
    new PublicKey(TERRA_PROGRAM_ID),
  )
}

// Derive a rights PDA for a parcel account + nonce.
export function rightsPda(parcel: PublicKey, nonce: number): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [strToBytes('rights'), parcel.toBuffer(), Uint8Array.of(nonce)],
    new PublicKey(TERRA_PROGRAM_ID),
  )
}

// Derive an attestation PDA: ["attestation", parcel, specifier].
export function attestationPda(
  parcel: PublicKey,
  specifier: Uint8Array,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [strToBytes('attestation'), parcel.toBuffer(), specifier],
    new PublicKey(TERRA_PROGRAM_ID),
  )
}

// Derive an identity PDA: ["identity", identity_hash].
export function identityPda(identityHash: Uint8Array): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [strToBytes('identity'), identityHash],
    new PublicKey(TERRA_PROGRAM_ID),
  )
}

// Derive a succession PDA: ["succession", identity, successor].
export function successionPda(
  identity: PublicKey,
  successor: PublicKey,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [strToBytes('succession'), identity.toBuffer(), successor.toBuffer()],
    new PublicKey(TERRA_PROGRAM_ID),
  )
}

export { getProvider }
