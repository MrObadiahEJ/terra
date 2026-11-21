import { useCallback, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import {
  Activity,
  ArrowRight,
  CheckCircle2,
  Clock,
  ExternalLink,
  FileText,
  Fingerprint,
  MapPin,
  ShieldCheck,
  UserCheck,
  UserX,
  Wallet,
} from 'lucide-react'
import { PublicKey, SystemProgram, Transaction } from '@solana/web3.js'
import { BN, BorshCoder } from '@coral-xyz/anchor'
import { connection, useWallet } from '../lib/wallet'
import { TERRA_PROGRAM_ID } from '../lib/constants'
import idl from '../idl/terra_registry.json'
import { digest, sdJwtDigest } from '../lib/lifecyclePlan'
import {
  bytesToHex,
  canonicalVcMessage,
  residencyMessage,
  verifyCompletePresentation,
  type VerificationBundle,
} from '../lib/mvpVerifier'
import { useWalletModal } from '@solana/wallet-adapter-react-ui'

// ---------------------------------------------------------------------------
// MVP Flow — Intent 5: ONE flow end-to-end
//
// Licensed surveyor attests ONE parcel → issues W3C VC (SD-JWT) →
// owner+tenant dual-sign residency proof → tenant presents to mock verifier.
//
// All on-chain writes go through the real terra_registry program via IDL.
// Off-chain: SD-JWT VC construction, dual-sign residency proof, mock verifier
// signature check. Zero budget: SOL fees only, devnet RPC, free tiers.
// ---------------------------------------------------------------------------

type FlowStep = 'surveyor' | 'vc' | 'residency' | 'verify' | 'complete'

type Parcel = {
  id: string
  name: string
  owner: string
  geometry: string
}

type ResidencyProof = {
  ownerPubkey: string
  tenantPubkey: string
  parcelId: string
  ownerSignature: string
  tenantSignature: string
  timestamp: number
}

type VC = {
  vcType: string
  issuer: string
  credentialSubject: string
  observation: string
  issuanceDate: number
  expirationDate: number
  sdJwtDigest: string
  selectiveDisclosure: boolean
  issuerSignature?: string
}

const DEMO_PARCELS: Parcel[] = [
  {
    id: 'e2e-verify-parcel',
    name: 'Soa Pilot Parcel — Yaoundé, Cameroon',
    owner: '11111111111111111111111111111111',
    geometry: 'POLYGON((11.499 3.847, 11.505 3.847, 11.505 3.849, 11.499 3.849, 11.499 3.847))',
  },
]

function formatPubkey(pk: string): string {
  if (!pk) return ''
  return `${pk.slice(0, 4)}…${pk.slice(-4)}`
}

function formatTimestamp(ts: number): string {
  return new Date(ts * 1000).toLocaleString()
}


export default function MvpFlowPage() {
  const navigate = useNavigate()
  const { publicKey, connected, anchorWallet, send, signMessage } = useWallet()
  const { setVisible: setWalletModalVisible } = useWalletModal()
  const [step, setStep] = useState<FlowStep>('surveyor')
  const [selectedParcel, setSelectedParcel] = useState<Parcel | null>(DEMO_PARCELS[0])
  const [vc, setVc] = useState<VC | null>(null)
  const [residencyProof, setResidencyProof] = useState<ResidencyProof | null>(null)
  const [verificationResult, setVerificationResult] = useState<'idle' | 'valid' | 'invalid' | 'checking'>('idle')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [ownerSigned, setOwnerSigned] = useState(false)
  const [tenantSigned, setTenantSigned] = useState(false)
  const [tenantKey, setTenantKey] = useState('')

  const handleConnect = useCallback(() => {
    setWalletModalVisible(true)
  }, [setWalletModalVisible])

  // Step 1: Surveyor attests parcel → issues W3C VC (SD-JWT)
  // Builds the full on-chain account chain: parcel → claim → observation →
  // verification_attestation. Each PDA is derived with the exact seeds the
  // program expects, and every account is created before it is referenced.
  const handleIssueVC = useCallback(async () => {
    if (!anchorWallet || !send || !selectedParcel || !publicKey) return
    setBusy(true)
    setError(null)
    try {
      const programId = new PublicKey(TERRA_PROGRAM_ID)
      const encoder = new TextEncoder()

      // ── PDAs (must match program seed layout) ─────────────────────────
      const parcelIdBytes = await digest([`terra:mvp:parcel:${selectedParcel.id}`])
      const [parcelPk] = PublicKey.findProgramAddressSync(
        [encoder.encode('parcel'), parcelIdBytes],
        programId,
      )
      const [ownershipPk] = PublicKey.findProgramAddressSync(
        [encoder.encode('ownership'), parcelPk.toBuffer()],
        programId,
      )

      const claimIdBytes = await digest([`terra:mvp:claim:${selectedParcel.id}`])
      const [claimPk] = PublicKey.findProgramAddressSync(
        [encoder.encode('claim'), parcelPk.toBuffer(), claimIdBytes],
        programId,
      )

      const surveyorPk = publicKey
      const [observationPk] = PublicKey.findProgramAddressSync(
        [encoder.encode('observation'), claimPk.toBuffer(), surveyorPk.toBuffer()],
        programId,
      )

      const [attestationPk] = PublicKey.findProgramAddressSync(
        [encoder.encode('verification_attestation'), claimPk.toBuffer(), surveyorPk.toBuffer()],
        programId,
      )

      // ── Dates ─────────────────────────────────────────────────────────
      const issuanceDate = Math.floor(Date.now() / 1000)
      const expirationDate = issuanceDate + 365 * 24 * 60 * 60 // 1 year
      const vcType = 'ResidencyAttestation'

      const vcDigest = await sdJwtDigest(
        claimPk,
        surveyorPk,
        observationPk,
        claimPk, // credential subject = parcel
        vcType,
        issuanceDate,
        expirationDate,
      )

      const coder = new BorshCoder(idl as never)

      // 1) register_parcel — creates parcel + ownership PDAs
      const geometryHash = await digest([`terra:mvp:geometry:${selectedParcel.id}`])
      const regData = coder.instruction.encode('register_parcel', {
        id: Array.from(parcelIdBytes),
        name: selectedParcel.name,
        geometry_hash: Array.from(geometryHash),
      })
      const regIx = {
        programId,
        keys: [
          { pubkey: parcelPk, isSigner: false, isWritable: true },
          { pubkey: surveyorPk, isSigner: true, isWritable: false },
          { pubkey: ownershipPk, isSigner: false, isWritable: true },
          { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ],
        data: regData,
      }

      // 2) create_claim — creates claim PDA (requires parcel PDA)
      const statementHash = await digest([`terra:mvp:statement:${selectedParcel.id}`])
      const claimData = coder.instruction.encode('create_claim', {
        claim_id: Array.from(claimIdBytes),
        claim_type: 1, // CLAIM_TYPE.BOUNDARY
        statement_hash: Array.from(statementHash),
        parcel_type: 0,
        region: [0, 0],
      })
      const claimIx = {
        programId,
        keys: [
          { pubkey: claimPk, isSigner: false, isWritable: true },
          { pubkey: parcelPk, isSigner: false, isWritable: false },
          { pubkey: surveyorPk, isSigner: true, isWritable: false },
          { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ],
        data: claimData,
      }

      // 3) submit_observation — creates observation PDA (requires claim PDA)
      const findingsHash = await digest([`terra:mvp:findings:${selectedParcel.id}`])
      const sigHash = await digest([`terra:mvp:sig:${selectedParcel.id}`])
      const obsData = coder.instruction.encode('submit_observation', {
        location: [-6_200_000_000, 106_800_000_000], // demo coordinates
        method: 0,
        findings_hash: Array.from(findingsHash),
        confidence: 90,
        signature_hash: Array.from(sigHash),
      })
      const obsIx = {
        programId,
        keys: [
          { pubkey: observationPk, isSigner: false, isWritable: true },
          { pubkey: claimPk, isSigner: false, isWritable: false },
          { pubkey: surveyorPk, isSigner: true, isWritable: false },
          { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ],
        data: obsData,
      }

      // 4) submit_verification_attestation — creates attestation PDA
      const attData = coder.instruction.encode('submit_verification_attestation', {
        vc_type: vcType,
        credential_subject: claimPk,
        issuance_date: new BN(issuanceDate),
        expiration_date: new BN(expirationDate),
        sd_jwt_digest: Array.from(vcDigest),
        selective_disclosure: false,
      })
      const attIx = {
        programId,
        keys: [
          { pubkey: attestationPk, isSigner: false, isWritable: true },
          { pubkey: claimPk, isSigner: false, isWritable: false },
          { pubkey: observationPk, isSigner: false, isWritable: false },
          { pubkey: surveyorPk, isSigner: true, isWritable: false },
          { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ],
        data: attData,
      }

      const tx = new Transaction().add(regIx, claimIx, obsIx, attIx)
      await send(tx)

      let issuerSignature = ''
      if (signMessage) {
        try {
          const msg = canonicalVcMessage({
            vcType,
            issuer: surveyorPk.toBase58(),
            credentialSubject: claimPk.toBase58(),
            observation: observationPk.toBase58(),
            issuanceDate,
            expirationDate,
            sdJwtDigest: bytesToHex(vcDigest),
          })
          const sig = await signMessage(msg)
          issuerSignature = bytesToHex(sig)
        } catch {
          // On-chain tx signature serves as programmatic anchor
        }
      }

      setVc({
        vcType,
        issuer: surveyorPk.toBase58(),
        credentialSubject: claimPk.toBase58(),
        observation: observationPk.toBase58(),
        issuanceDate,
        expirationDate,
        sdJwtDigest: bytesToHex(vcDigest),
        selectiveDisclosure: false,
        issuerSignature,
      })
      setStep('vc')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'VC issuance failed')
    } finally {
      setBusy(false)
    }
  }, [anchorWallet, send, signMessage, publicKey, selectedParcel])

  // Step 2: Owner+tenant dual-sign residency proof. Both parties sign the
  // exact same canonical message (RFC-018 §4, Step 2).
  const handleOwnerSign = useCallback(async () => {
    if (!signMessage || !publicKey) return
    try {
      const tenant = tenantKey.trim() || publicKey.toBase58()
      new PublicKey(tenant) // validate before signing
      const timestamp = Math.floor(Date.now() / 1000)
      const message = residencyMessage({
        ownerPubkey: publicKey.toBase58(),
        tenantPubkey: tenant,
        parcelId: selectedParcel!.id,
        timestamp,
      })
      const signature = await signMessage(message)
      setOwnerSigned(true)
      setResidencyProof({
        ownerPubkey: publicKey.toBase58(),
        tenantPubkey: tenant,
        parcelId: selectedParcel!.id,
        ownerSignature: bytesToHex(signature),
        tenantSignature: '',
        timestamp,
      })
      setTenantSigned(false)
      setError(null)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Owner sign failed')
    }
  }, [signMessage, publicKey, selectedParcel, tenantKey])

  const handleTenantSign = useCallback(async () => {
    if (!signMessage || !publicKey || !residencyProof) return
    try {
      if (publicKey.toBase58() !== residencyProof.tenantPubkey) {
        throw new Error(
          `Connect the tenant wallet ${formatPubkey(residencyProof.tenantPubkey)} to sign`,
        )
      }
      const message = residencyMessage(residencyProof)
      const signature = await signMessage(message)
      setTenantSigned(true)
      setResidencyProof((prev) =>
        prev ? { ...prev, tenantSignature: bytesToHex(signature) } : prev,
      )
      setError(null)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Tenant sign failed')
    }
  }, [signMessage, publicKey, residencyProof])

  // Step 4: Standalone verifier — cryptographic VC + residency verification
  const handleVerify = useCallback(async () => {
    if (!vc || !residencyProof || !selectedParcel) return
    setVerificationResult('checking')
    setBusy(true)
    try {
      const bundle: VerificationBundle = {
        parcelId: selectedParcel.id,
        vc,
        residencyProof,
      }
      localStorage.setItem('terra_mvp_presentation_bundle', JSON.stringify(bundle))

      const programId = new PublicKey(TERRA_PROGRAM_ID)
      const report = await verifyCompletePresentation(bundle, connection, programId)
      setVerificationResult(report.overallValid ? 'valid' : 'invalid')
      if (!report.overallValid) {
        const failures = [
          !report.issuerSignature.valid && 'Invalid issuer signature',
          !report.sdJwtDisclosure.valid && 'SD-JWT digest mismatch',
          !report.residencyDualSign.valid && 'Residency dual-signature failure',
          !report.temporalValidity.valid && 'Temporal validity or freshness window failure',
          report.onChainAnchorAndQuorum && !report.onChainAnchorAndQuorum.valid && report.onChainAnchorAndQuorum.details,
        ].filter(Boolean)
        setError(failures.join('; ') || 'Verification failed')
      } else {
        setError(null)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Verification failed')
      setVerificationResult('invalid')
    } finally {
      setBusy(false)
    }
  }, [vc, residencyProof, selectedParcel])

  const restart = () => {
    setStep('surveyor')
    setVc(null)
    setResidencyProof(null)
    setVerificationResult('idle')
    setOwnerSigned(false)
    setTenantSigned(false)
    setError(null)
  }

  const stepLabels: Record<FlowStep, string> = {
    surveyor: '1 · Surveyor Attests',
    vc: '2 · W3C VC Issued',
    residency: '3 · Dual-Sign Residency',
    verify: '4 · Standalone Verifier',
    complete: '5 · Complete',
  }

  const stepDescriptions: Record<FlowStep, string> = {
    surveyor: 'Select a parcel and have the licensed surveyor issue a W3C Verifiable Credential (SD-JWT) anchored on-chain.',
    vc: 'The VC is anchored on-chain with its SD-JWT digest. Selective disclosure is available for privacy-preserving verification.',
    residency: 'Owner and tenant each sign the residency proof with their wallet keys. Both signatures are required.',
    verify: 'The standalone verifier validates VC issuer signature, SD-JWT selective disclosure, dual residency signatures, and on-chain quorum.',
    complete: 'MVP flow complete — one parcel, one VC, one dual-signed residency proof, one verified presentation.',
  }

  return (
    <div className="st-page mvp-page">
      <header className="mvp-head">
        <div className="min-w-0">
          <h1 className="mvp-title">
            <Activity size={18} /> Terra MVP Flow
          </h1>
          <p className="text-xs text-muted">
            Licensed surveyor → W3C VC → dual-signed residency → mock verifier
          </p>
        </div>
        <div className="mvp-status">
          <span
            className={`mvp-badge ${connected ? 'mvp-badge-ok' : 'mvp-badge-warn'}`}
          >
            {connected ? 'Wallet connected' : 'Wallet not connected'}
          </span>
        </div>
      </header>

      {!connected && (
        <div className="mvp-connect-banner">
          <Wallet size={14} /> Connect a wallet to start
          <button className="btn btn-primary ml-2" onClick={handleConnect}>
            Connect Wallet
          </button>
        </div>
      )}

      {/* Stepper */}
      <div className="mvp-stepper">
        {(['surveyor', 'vc', 'residency', 'verify', 'complete'] as FlowStep[]).map(
          (s, i) => {
            const active = step === s
            const done =
              ['surveyor', 'vc', 'residency', 'verify'].indexOf(s) <
              ['surveyor', 'vc', 'residency', 'verify'].indexOf(step)
            return (
              <div key={s} className="mvp-step">
                <div
                  className={`mvp-step-marker ${
                    done
                      ? 'mvp-step-done'
                      : active
                      ? 'mvp-step-active'
                      : 'mvp-step-pending'
                  }`}
                >
                  {i + 1}
                </div>
                <div className="mvp-step-label">{stepLabels[s]}</div>
                {i < 4 && <div className="mvp-step-connector" />}
              </div>
            )
          },
        )}
      </div>

      <div className="mvp-content">
        {/* Step 1: Surveyor selects parcel */}
        {step === 'surveyor' && (
          <div className="mvp-card">
            <div className="mvp-card-h">
              <MapPin size={16} /> {stepLabels.surveyor}
            </div>
            <p className="text-sm text-muted">{stepDescriptions.surveyor}</p>
            <div className="mvp-parcel-select">
              <label className="text-xs text-muted">Select parcel</label>
              {DEMO_PARCELS.map((p) => (
                <button
                  key={p.id}
                  className={`mvp-parcel-chip ${selectedParcel?.id === p.id ? 'mvp-parcel-chip-selected' : ''}`}
                  onClick={() => setSelectedParcel(p)}
                >
                  <div className="text-xs font-mono">{p.id}</div>
                  <div className="text-sm">{p.name}</div>
                  <div className="text-[10px] text-muted">{formatPubkey(p.owner)}</div>
                </button>
              ))}
            </div>
            {selectedParcel && (
              <div className="mvp-parcel-preview">
                <div className="mvp-preview-row">
                  <span className="text-xs text-muted">Parcel ID</span>
                  <span className="text-xs font-mono">{selectedParcel.id}</span>
                </div>
                <div className="mvp-preview-row">
                  <span className="text-xs text-muted">Geometry</span>
                  <span className="text-xs font-mono truncate">{selectedParcel.geometry}</span>
                </div>
                <div className="mvp-preview-row">
                  <span className="text-xs text-muted">Owner</span>
                  <span className="text-xs font-mono">{formatPubkey(selectedParcel.owner)}</span>
                </div>
              </div>
            )}
            <div className="mvp-card-actions">
              {selectedParcel ? (
                <button
                  className="btn btn-primary"
                  onClick={handleIssueVC}
                  disabled={busy || !connected}
                >
                  {busy ? 'Issuing…' : 'Attest Parcel'}
                </button>
              ) : (
                <button className="btn btn-secondary" disabled>
                  Select parcel first
                </button>
              )}
              <button className="btn btn-ghost ml-2" onClick={restart}>
                Restart
              </button>
            </div>
            {error && <div className="mvp-error">{error}</div>}
          </div>
        )}

        {/* Step 2: W3C VC issued */}
        {step === 'vc' && (
          <div className="mvp-card">
            <div className="mvp-card-h">
              <FileText size={16} /> {stepLabels.vc}
            </div>
            <p className="text-sm text-muted">{stepDescriptions.vc}</p>
            <div className="mvp-vc-card">
              <div className="mvp-vc-header">
                <span className="text-xs font-mono">{vc?.vcType}</span>
                <span className="text-xs text-muted">
                  Signed by: {formatPubkey(vc?.issuer ?? '')}
                </span>
              </div>
              <div className="mvp-vc-body">
                <div className="mvp-preview-row">
                  <span className="text-xs text-muted">Subject</span>
                  <span className="text-xs font-mono">
                    {formatPubkey(vc?.credentialSubject ?? '')}
                  </span>
                </div>
                <div className="mvp-preview-row">
                  <span className="text-xs text-muted">Issued</span>
                  <span className="text-xs font-mono">
                    {formatTimestamp(vc?.issuanceDate ?? 0)}
                  </span>
                </div>
                <div className="mvp-preview-row">
                  <span className="text-xs text-muted">Expires</span>
                  <span className="text-xs font-mono">
                    {formatTimestamp(vc?.expirationDate ?? 0)}
                  </span>
                </div>
                <div className="mvp-preview-row">
                  <span className="text-xs text-muted">SD-JWT Digest</span>
                  <span className="text-[10px] font-mono truncate" title={vc?.sdJwtDigest}>
                    {vc?.sdJwtDigest?.slice(0, 32)}…
                  </span>
                </div>
                <div className="mvp-preview-row">
                  <span className="text-xs text-muted">Selective Disclosure</span>
                  <span className="text-xs">
                    {vc?.selectiveDisclosure ? 'Enabled' : 'Disabled'}
                  </span>
                </div>
              </div>
              <div className="mvp-vc-footer">
                <CheckCircle2 size={14} className="text-green-600" />
                <span className="text-sm font-medium">VC anchored on-chain</span>
              </div>
            </div>
            <div className="mvp-card-actions">
              <button
                className="btn btn-primary"
                onClick={() => setStep('residency')}
                disabled={busy}
              >
                {busy ? 'Issuing…' : 'Proceed to Residency'}
              </button>
              <button className="btn btn-ghost ml-2" onClick={restart}>
                Restart
              </button>
            </div>
            {error && <div className="mvp-error">{error}</div>}
          </div>
        )}

        {/* Step 3: Owner+tenant dual-sign residency proof */}
        {step === 'residency' && (
          <div className="mvp-card">
            <div className="mvp-card-h">
              <UserCheck size={16} /> {stepLabels.residency}
            </div>
            <p className="text-sm text-muted">{stepDescriptions.residency}</p>
            <div className="mb-3">
              <label className="text-xs text-muted" style={{ display: 'block', marginBottom: 4 }}>
                Tenant wallet (base58)
              </label>
              <input
                value={tenantKey}
                onChange={(e) => setTenantKey(e.target.value)}
                placeholder={publicKey?.toBase58() ?? 'Tenant pubkey'}
                disabled={ownerSigned || busy}
                style={{
                  width: '100%',
                  padding: '8px 10px',
                  borderRadius: 8,
                  border: '1px solid var(--border)',
                  background: 'var(--surface)',
                  color: 'var(--ink)',
                  fontFamily: 'var(--font-mono)',
                  fontSize: 12,
                }}
              />
            </div>
            <div className="mvp-dual-sign">
              <div className="mvp-sign-role">
                <UserCheck size={16} className="text-blue-600" />
                <div>
                  <div className="text-sm font-medium">Owner</div>
                  <div className="text-xs text-muted">
                    {formatPubkey(residencyProof?.ownerPubkey ?? publicKey?.toBase58() ?? '')}
                  </div>
                </div>
                <button
                  className={`btn ${ownerSigned ? 'btn-success' : 'btn-outline'} w-full mb-2`}
                  onClick={handleOwnerSign}
                  disabled={busy || !connected || !publicKey}
                >
                  {ownerSigned ? 'Owner Signed ✓' : 'Sign as Owner'}
                </button>
              </div>
              <div className="mvp-sign-arrow">
                <ArrowRight size={14} />
              </div>
              <div className="mvp-sign-role">
                <UserX size={16} className="text-purple-600" />
                <div>
                  <div className="text-sm font-medium">Tenant</div>
                  <div className="text-xs text-muted">
                    {formatPubkey(residencyProof?.tenantPubkey ?? '')}                  </div>
                </div>
                <button
                  className={`btn ${tenantSigned ? 'btn-success' : 'btn-outline'} w-full mb-2`}
                  onClick={handleTenantSign}
                  disabled={busy || !connected || !publicKey || !ownerSigned}
                >
                  {tenantSigned ? 'Tenant Signed ✓' : 'Sign as Tenant'}
                </button>
              </div>
            </div>
            {residencyProof && (
              <div className="mvp-proof-preview">
                <div className="mvp-preview-row">
                  <span className="text-xs text-muted">Parcel</span>
                  <span className="text-xs font-mono">{residencyProof.parcelId}</span>
                </div>
                <div className="mvp-preview-row">
                  <span className="text-xs text-muted">Owner Signature</span>
                  <span className="text-[10px] font-mono" title={residencyProof.ownerSignature}>
                    {residencyProof.ownerSignature.slice(0, 24)}…
                  </span>
                </div>
                <div className="mvp-preview-row">
                  <span className="text-xs text-muted">Tenant Signature</span>
                  <span className="text-[10px] font-mono" title={residencyProof.tenantSignature}>
                    {residencyProof.tenantSignature.slice(0, 24)}…
                  </span>
                </div>
              </div>
            )}
            <div className="mvp-card-actions">
              <button
                className="btn btn-primary"
                onClick={() => setStep('verify')}
                disabled={busy || !(ownerSigned && tenantSigned)}
              >
                {busy ? 'Signing…' : 'Proceed to Verify'}
              </button>
              <button className="btn btn-ghost ml-2" onClick={restart}>
                Restart
              </button>
            </div>
            {error && <div className="mvp-error">{error}</div>}
          </div>
        )}

        {/* Step 4: Mock verifier */}
        {step === 'verify' && (
          <div className="mvp-card">
            <div className="mvp-card-h">
              <ShieldCheck size={16} /> {stepLabels.verify}
            </div>
            <p className="text-sm text-muted">{stepDescriptions.verify}</p>
            <div className="mvp-verify-panel">
              <div className="mvp-verify-check">
                <div className="mvp-verify-check-icon">
                  {verificationResult === 'checking' ? (
                    <Clock size={14} className="animate-spin" />
                  ) : verificationResult === 'valid' ? (
                    <CheckCircle2 size={14} className="text-green-600" />
                  ) : (
                    <Fingerprint size={14} className="text-red-600" />
                  )}
                </div>
                <div>
                  <div className="text-sm font-medium">
                    {verificationResult === 'checking'
                      ? 'Verifying…'
                      : verificationResult === 'valid'
                        ? 'Verification Passed'
                        : 'Verification Failed'}
                  </div>
                  <div className="text-xs text-muted">
                    {verificationResult === 'valid'
                      ? 'VC digest matches on-chain anchor; both residency signatures valid.'
                      : verificationResult === 'invalid'
                        ? 'VC digest or residency signatures do not match.'
                        : 'Verifying credentials against on-chain anchor…'}
                  </div>
                </div>
              </div>
              <div className="mvp-verify-details">
                <div className="mvp-preview-row">
                  <span className="text-xs text-muted">VC Type</span>
                  <span className="text-xs font-mono">{vc?.vcType}</span>
                </div>
                <div className="mvp-preview-row">
                  <span className="text-xs text-muted">Issuer</span>
                  <span className="text-xs font-mono truncate">
                    {formatPubkey(vc?.issuer ?? '')}
                  </span>
                </div>
                <div className="mvp-preview-row">
                  <span className="text-xs text-muted">Owner Signed</span>
                  <span className="text-xs">
                    {residencyProof?.ownerSignature ? 'Yes' : 'No'}
                  </span>
                </div>
                <div className="mvp-preview-row">
                  <span className="text-xs text-muted">Tenant Signed</span>
                  <span className="text-xs">
                    {residencyProof?.tenantSignature ? 'Yes' : 'No'}
                  </span>
                </div>
              </div>
            </div>
            <div className="mvp-card-actions flex flex-wrap gap-2 items-center">
              {verificationResult === 'valid' ? (
                <button
                  className="btn btn-primary"
                  onClick={() => setStep('complete')}
                  disabled={busy}
                >
                  Complete Flow
                </button>
              ) : (
                <button
                  className="btn btn-primary"
                  onClick={handleVerify}
                  disabled={busy}
                >
                  {busy ? 'Verifying…' : 'Run Standalone Verification'}
                </button>
              )}
              {vc && residencyProof && selectedParcel && (
                <button
                  className="btn btn-secondary flex items-center gap-1.5"
                  onClick={() => {
                    const bundle: VerificationBundle = {
                      parcelId: selectedParcel.id,
                      vc,
                      residencyProof,
                    }
                    localStorage.setItem('terra_mvp_presentation_bundle', JSON.stringify(bundle))
                    navigate('/verify', { state: { bundle } })
                  }}
                >
                  <ExternalLink size={14} /> Open in Standalone Verifier (/verify)
                </button>
              )}
              <button className="btn btn-ghost ml-2" onClick={restart}>
                Restart
              </button>
            </div>
            {error && <div className="mvp-error">{error}</div>}
          </div>
        )}

        {/* Step 5: Complete */}
        {step === 'complete' && (
          <div className="mvp-card mvp-card-complete">
            <div className="mvp-card-h">
              <ShieldCheck size={16} className="text-green-600" />
              {stepLabels.complete}
            </div>
            <div className="mvp-complete-content">
              <div className="mvp-complete-icon">✓</div>
              <h3 className="mvp-complete-title">Flow Complete</h3>
              <p className="text-sm text-muted">
                One parcel, one VC, one dual-signed residency proof, one verified presentation.
              </p>
              <div className="mvp-complete-summary">
                <div className="mvp-summary-row">
                  <span className="text-xs text-muted">Parcel</span>
                  <span className="text-xs font-mono">{selectedParcel?.id}</span>
                </div>
                <div className="mvp-summary-row">
                  <span className="text-xs text-muted">Surveyor</span>
                  <span className="text-xs font-mono truncate">
                    {formatPubkey(publicKey?.toBase58() ?? '')}
                  </span>
                </div>
                <div className="mvp-summary-row">
                  <span className="text-xs text-muted">VC Type</span>
                  <span className="text-xs font-mono">{vc?.vcType}</span>
                </div>
                <div className="mvp-summary-row">
                  <span className="text-xs text-muted">Residency Proof</span>
                  <span className="text-xs">Dual-signed</span>
                </div>
                <div className="mvp-summary-row">
                  <span className="text-xs text-muted">Verification</span>
                  <span className="text-xs text-green-600">Passed</span>
                </div>
              </div>
            </div>
            <div className="mvp-card-actions">
              <button className="btn btn-primary" onClick={restart}>
                Run Again
              </button>
            </div>
          </div>
        )}
      </div>

      <footer className="mvp-footer">
        <p className="text-[10px] text-muted">
          Terra MVP — SOL fees only · devnet · no token · free tiers
        </p>
      </footer>
    </div>
  )
}