// Cryptographic primitives for the /mvp verifier (Intent 5, RFC-018 §4).
//
// Pure and dependency-light so the verifier's security properties can be
// unit-tested without React, RPC, or a wallet: hex codecs, the canonical
// residency message, and Ed25519 signature verification that rejects malformed
// keys/signatures instead of throwing.

import { PublicKey } from '@solana/web3.js'
import { ed25519 } from '@noble/curves/ed25519'

export function hexToBytes(hex: string): Uint8Array {
  if (hex.length % 2 !== 0 || !/^[0-9a-f]*$/i.test(hex)) return new Uint8Array()
  const out = new Uint8Array(hex.length / 2)
  for (let i = 0; i < out.length; i += 1) {
    out[i] = parseInt(hex.slice(i * 2, i * 2 + 2), 16)
  }
  return out
}

export function bytesToHex(bytes: Uint8Array | number[]): string {
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('')
}

export interface ResidencyMessageInput {
  ownerPubkey: string
  tenantPubkey: string
  parcelId: string
  timestamp: number
}

/** Canonical dual-sign residency message (RFC-018 §4, Step 2). */
export function residencyMessage(input: ResidencyMessageInput): Uint8Array {
  return new TextEncoder().encode(
    `RESIDENCY_PROOF\nowner:${input.ownerPubkey}\ntenant:${input.tenantPubkey}\nparcel:${input.parcelId}\ntimestamp:${input.timestamp}`,
  )
}

/**
 * Verify an Ed25519 signature over `message` against a base58 pubkey.
 * Returns false for malformed signatures, malformed pubkeys, or a validly
 * encoded signature that does not verify — never throws.
 */
export function verifyEd25519(
  pubkeyBase58: string,
  message: Uint8Array,
  signatureHex: string,
): boolean {
  try {
    const sig = hexToBytes(signatureHex)
    if (sig.length !== 64) return false
    const pubkey = new PublicKey(pubkeyBase58).toBytes()
    return ed25519.verify(sig, message, pubkey)
  } catch {
    return false
  }
}

/** Verify both signatures over the same canonical message. */
export function verifyResidencyProof(proof: {
  ownerPubkey: string
  tenantPubkey: string
  ownerSignature: string
  tenantSignature: string
  parcelId: string
  timestamp: number
}): { owner: boolean; tenant: boolean } {
  const message = residencyMessage(proof)
  return {
    owner: verifyEd25519(proof.ownerPubkey, message, proof.ownerSignature),
    tenant: verifyEd25519(proof.tenantPubkey, message, proof.tenantSignature),
  }
}
