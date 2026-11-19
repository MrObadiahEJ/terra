import { describe, expect, it } from 'vitest'
import { ed25519 } from '@noble/curves/ed25519'
import { PublicKey } from '@solana/web3.js'
import {
  bytesToHex,
  hexToBytes,
  residencyMessage,
  verifyEd25519,
  verifyResidencyProof,
} from './mvpVerifier'

function keypair() {
  const secret = ed25519.utils.randomPrivateKey()
  const publicBytes = ed25519.getPublicKey(secret)
  return {
    secret,
    publicBytes,
    publicBase58: new PublicKey(publicBytes).toBase58(),
  }
}

const claim = {
  ownerPubkey: '',
  tenantPubkey: '',
  parcelId: 'e2e-verify-parcel',
  timestamp: 1_764_000_000,
}

describe('MVP residency verifier', () => {
  it('accepts owner and tenant signatures over the same canonical message', () => {
    const owner = keypair()
    const tenant = keypair()
    const input = { ...claim, ownerPubkey: owner.publicBase58, tenantPubkey: tenant.publicBase58 }
    const message = residencyMessage(input)

    const proof = {
      ...input,
      ownerSignature: bytesToHex(ed25519.sign(message, owner.secret)),
      tenantSignature: bytesToHex(ed25519.sign(message, tenant.secret)),
    }

    expect(verifyResidencyProof(proof)).toEqual({ owner: true, tenant: true })
  })

  it('rejects a forged signature: signed by a different key', () => {
    const owner = keypair()
    const tenant = keypair()
    const attacker = keypair()
    const input = { ...claim, ownerPubkey: owner.publicBase58, tenantPubkey: tenant.publicBase58 }
    const message = residencyMessage(input)

    // Attacker signs the correct bytes, but claims to be the owner.
    const forged = bytesToHex(ed25519.sign(message, attacker.secret))
    expect(verifyEd25519(owner.publicBase58, message, forged)).toBe(false)

    // A genuinely valid tenant signature must not rescue the forged owner.
    const tenantSignature = bytesToHex(ed25519.sign(message, tenant.secret))
    expect(
      verifyResidencyProof({
        ...input,
        ownerSignature: forged,
        tenantSignature,
      }),
    ).toEqual({ owner: false, tenant: true })
  })

  it('rejects a signature over a different parcel (replay)', () => {
    const owner = keypair()
    const tenant = keypair()
    const input = { ...claim, ownerPubkey: owner.publicBase58, tenantPubkey: tenant.publicBase58 }
    const otherMessage = residencyMessage({ ...input, parcelId: 'someone-elses-parcel' })

    const signature = bytesToHex(ed25519.sign(otherMessage, owner.secret))
    expect(verifyEd25519(owner.publicBase58, residencyMessage(input), signature)).toBe(false)
    expect(verifyEd25519(owner.publicBase58, otherMessage, signature)).toBe(true)
    expect(tenant.publicBase58).not.toBe(owner.publicBase58)
  })

  it('rejects malformed signatures and pubkeys without throwing', () => {
    const owner = keypair()
    const message = residencyMessage({ ...claim, ownerPubkey: owner.publicBase58, tenantPubkey: owner.publicBase58 })

    expect(verifyEd25519(owner.publicBase58, message, '')).toBe(false)
    expect(verifyEd25519(owner.publicBase58, message, 'zz')).toBe(false)
    expect(verifyEd25519(owner.publicBase58, message, 'ab'.repeat(63))).toBe(false)
    expect(verifyEd25519('not-a-pubkey', message, 'ab'.repeat(64))).toBe(false)
    expect(hexToBytes('abc')).toEqual(new Uint8Array())
  })
})
