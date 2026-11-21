import { describe, expect, it } from 'vitest'
import { ed25519 } from '@noble/curves/ed25519'
import { PublicKey } from '@solana/web3.js'
import {
  bytesToHex,
  hexToBytes,
  residencyMessage,
  canonicalVcMessage,
  verifyEd25519,
  verifyResidencyProof,
  verifyVcIssuerSignature,
  computeSdJwtDigest,
  verifySdJwtDisclosure,
  verifyTemporalValidity,
  verifyCompletePresentation,
  type VCInput,
  type ResidencyProofInput,
  type VerificationBundle,
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

    expect(verifyResidencyProof(proof)).toEqual({ owner: true, tenant: true, valid: true })
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
    ).toEqual({ owner: false, tenant: true, valid: false })
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

  it('verifies VC issuer Ed25519 signature over canonical VC payload', () => {
    const surveyor = keypair()
    const claimPk = keypair().publicBase58
    const obsPk = keypair().publicBase58

    const vc: VCInput = {
      vcType: 'ResidencyAttestation',
      issuer: surveyor.publicBase58,
      credentialSubject: claimPk,
      observation: obsPk,
      issuanceDate: 1_700_000_000,
      expirationDate: 1_800_000_000,
      sdJwtDigest: 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
    }
    const msg = canonicalVcMessage(vc)
    const sig = bytesToHex(ed25519.sign(msg, surveyor.secret))
    vc.issuerSignature = sig

    expect(verifyVcIssuerSignature(vc)).toBe(true)

    // Forged signature
    const attacker = keypair()
    const forgedSig = bytesToHex(ed25519.sign(msg, attacker.secret))
    expect(verifyVcIssuerSignature(vc, forgedSig)).toBe(false)
  })

  it('verifies SD-JWT disclosure digest against canonical preimage', async () => {
    const surveyor = keypair()
    const claimPk = keypair().publicBase58
    const obsPk = keypair().publicBase58

    const issuanceDate = 1_700_000_000
    const expirationDate = 1_800_000_000
    const vcType = 'ResidencyAttestation'

    const expectedDigest = await computeSdJwtDigest({
      claim: claimPk,
      validator: surveyor.publicBase58,
      observation: obsPk,
      credentialSubject: claimPk,
      vcType,
      issuanceDate,
      expirationDate,
    })

    const vc: VCInput = {
      vcType,
      issuer: surveyor.publicBase58,
      credentialSubject: claimPk,
      observation: obsPk,
      issuanceDate,
      expirationDate,
      sdJwtDigest: expectedDigest,
    }

    const res = await verifySdJwtDisclosure(vc)
    expect(res.valid).toBe(true)
    expect(res.matchesDeclared).toBe(true)

    // Tampered digest
    const tampered = { ...vc, sdJwtDigest: '00'.repeat(32) }
    const tamperedRes = await verifySdJwtDisclosure(tampered)
    expect(tamperedRes.valid).toBe(false)
  })

  it('validates temporal validity and freshness windows', () => {
    const now = 1_750_000_000
    const vc: VCInput = {
      vcType: 'ResidencyAttestation',
      issuer: keypair().publicBase58,
      credentialSubject: keypair().publicBase58,
      observation: keypair().publicBase58,
      issuanceDate: now - 3600,
      expirationDate: now + 3600,
      sdJwtDigest: '00'.repeat(32),
    }
    const proof: ResidencyProofInput = {
      ownerPubkey: keypair().publicBase58,
      tenantPubkey: keypair().publicBase58,
      parcelId: 'p1',
      ownerSignature: '00'.repeat(64),
      tenantSignature: '00'.repeat(64),
      timestamp: now - 100, // 100s ago (within 600s)
    }

    const okRes = verifyTemporalValidity(vc, proof, now, 600)
    expect(okRes.valid).toBe(true)

    // Expired VC
    const expiredRes = verifyTemporalValidity({ ...vc, expirationDate: now - 1 }, proof, now, 600)
    expect(expiredRes.valid).toBe(false)
    expect(expiredRes.vcExpired).toBe(true)

    // Stale residency timestamp (> 600s)
    const staleRes = verifyTemporalValidity(vc, { ...proof, timestamp: now - 1000 }, now, 600)
    expect(staleRes.valid).toBe(false)
    expect(staleRes.timestampFresh).toBe(false)
  })

  it('executes complete presentation verification', async () => {
    const surveyor = keypair()
    const owner = keypair()
    const tenant = keypair()
    const claimPk = keypair().publicBase58
    const obsPk = keypair().publicBase58

    const now = 1_750_000_000
    const issuanceDate = now - 3600
    const expirationDate = now + 86400 * 365
    const vcType = 'ResidencyAttestation'

    const digest = await computeSdJwtDigest({
      claim: claimPk,
      validator: surveyor.publicBase58,
      observation: obsPk,
      credentialSubject: claimPk,
      vcType,
      issuanceDate,
      expirationDate,
    })

    const vc: VCInput = {
      vcType,
      issuer: surveyor.publicBase58,
      credentialSubject: claimPk,
      observation: obsPk,
      issuanceDate,
      expirationDate,
      sdJwtDigest: digest,
    }
    vc.issuerSignature = bytesToHex(ed25519.sign(canonicalVcMessage(vc), surveyor.secret))

    const resProofInput: ResidencyMessageInput = {
      ownerPubkey: owner.publicBase58,
      tenantPubkey: tenant.publicBase58,
      parcelId: 'parcel-soa-1',
      timestamp: now - 30,
    }
    const msg = residencyMessage(resProofInput)
    const residencyProof: ResidencyProofInput = {
      ...resProofInput,
      ownerSignature: bytesToHex(ed25519.sign(msg, owner.secret)),
      tenantSignature: bytesToHex(ed25519.sign(msg, tenant.secret)),
    }

    const bundle: VerificationBundle = {
      parcelId: 'parcel-soa-1',
      vc,
      residencyProof,
    }

    const report = await verifyCompletePresentation(bundle, undefined, undefined, now)
    expect(report.overallValid).toBe(true)
    expect(report.issuerSignature.valid).toBe(true)
    expect(report.sdJwtDisclosure.valid).toBe(true)
    expect(report.residencyDualSign.valid).toBe(true)
    expect(report.temporalValidity.valid).toBe(true)
  })
})
