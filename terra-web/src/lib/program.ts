import { Program, AnchorProvider } from '@coral-xyz/anchor'
import { PublicKey } from '@solana/web3.js'
import { TERRA_IDENTITY_PROGRAM_ID, TERRA_PROGRAM_ID } from './constants'
import { connection, getActiveSigner } from './wallet'
import { strToBytes } from './codec'
import idl from '../idl/terra_registry.json'
import identityIdl from '../idl/terra_identity.json'
import type { TerraRegistry } from '../idl/terraRegistry'
import type { TerraIdentityProgram } from '../idl/terraIdentity'

// Anchor account interfaces (decoded from the on-chain program).
export interface ParcelAccount {
  id: number[]
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

let programSingleton: { program: Program<TerraRegistry>; wallet: string } | null = null
let identityProgramSingleton: { program: Program<TerraIdentityProgram>; wallet: string } | null = null

function getProvider(): AnchorProvider {
  const signer = getActiveSigner()
  if (!signer) {
    throw new Error('Wallet not connected')
  }
  return new AnchorProvider(connection, signer, { commitment: 'confirmed' })
}

function activeWalletKey(): string {
  const signer = getActiveSigner()
  if (!signer) throw new Error('Wallet not connected')
  return signer.publicKey.toBase58()
}

// Program is built lazily once a wallet is connected (Anchor needs a payer).
// The program id is read from the IDL address field. The singleton is rebuilt
// when the active wallet changes so a switched/reconnected wallet never signs
// with a stale provider.
export function getProgram(): Program<TerraRegistry> {
  const wallet = activeWalletKey()
  if (!programSingleton || programSingleton.wallet !== wallet) {
    programSingleton = {
      program: new Program<TerraRegistry>(idl as unknown as TerraRegistry, getProvider()),
      wallet,
    }
  }
  return programSingleton.program
}

// Identity lives in a separate program (different program id + IDL).
export function getIdentityProgram(): Program<TerraIdentityProgram> {
  const wallet = activeWalletKey()
  if (!identityProgramSingleton || identityProgramSingleton.wallet !== wallet) {
    identityProgramSingleton = {
      program: new Program<TerraIdentityProgram>(
        identityIdl as unknown as TerraIdentityProgram,
        getProvider(),
      ),
      wallet,
    }
  }
  return identityProgramSingleton.program
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

// Canonical ownership Rights PDA: ["ownership", parcel]. This is the single
// source of truth for parcel ownership (not a parcel column).
export function ownershipPda(parcel: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [strToBytes('ownership'), parcel.toBuffer()],
    new PublicKey(TERRA_PROGRAM_ID),
  )
}

// Derive an identity PDA: ["identity", identity_hash] on terra_identity.
export function identityPda(identityHash: Uint8Array): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [strToBytes('identity'), identityHash],
    new PublicKey(TERRA_IDENTITY_PROGRAM_ID),
  )
}

// Derive a succession PDA: ["succession", identity, successor] on terra_identity.
export function successionPda(
  identity: PublicKey,
  successor: PublicKey,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [strToBytes('succession'), identity.toBuffer(), successor.toBuffer()],
    new PublicKey(TERRA_IDENTITY_PROGRAM_ID),
  )
}

export { getProvider }
