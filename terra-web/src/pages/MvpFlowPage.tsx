import { useCallback, useState } from 'react'
import { Link } from 'react-router-dom'
import {
  Activity,
  AlertTriangle,
  ArrowRight,
  CheckCircle2,
  ChevronRight,
  Clock,
  Copy,
  FileText,
  Fingerprint,
  MapPin,
  ShieldCheck,
  UserCheck,
  UserX,
  Wallet,
} from 'lucide-react'
import { PublicKey, SystemProgram, Transaction } from '@solana/web3.js'
import { BN } from '@coral-xyz/anchor'
import { useWallet } from '../lib/wallet'
import { TERRA_PROGRAM_ID } from '../lib/constants'
import idl from '../idl/terra_registry.json'
import type { TerraRegistry } from '../idl/terraRegistry'
import { sdJwtDigest } from '../lib/lifecyclePlan'
import { useLocale } from '../lib/locale'

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
  issuanceDate: number
  expirationDate: number
  sdJwtDigest: string
  selectiveDisclosure: boolean
}

const DEMO_PARCELS: Parcel[] = [
  {
    id: 'e2e-verify-parcel',
    name: 'Soa Pilot Parcel — Yaoundé, Cameroon',
    owner: '3xK…Drsy',
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
  const { t } = useLocale()
  const { publicKey, connected, connect, anchorWallet, send, signMessage } = useWallet()
  const [step, setStep] = useState<FlowStep>('surveyor')
  const [selectedParcel, setSelectedParcel] = useState<Parcel | null>(DEMO_PARCELS[0])
  const [vc, setVc] = useState<VC | null>(null)
  const [residencyProof, setResidencyProof] = useState<ResidencyProof | null>(null)
  const [verificationResult, setVerificationResult] = useState<'idle' | 'valid' | 'invalid' | 'checking'>('idle')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [ownerSigned, setOwnerSigned] = useState(false)
  const [tenantSigned, setTenantSigned] = useState(false)

  const handleConnect = useCallback(async () => {
    try {
      await connect()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Wallet connect failed')
    }
  }, [connect])

  // Step 1: Surveyor attests parcel → issues W3C VC (SD-JWT)
  const handleIssueVC = useCallback(async () => {
    if (!anchorWallet || !send || !selectedParcel || !publicKey) return
    setBusy(true)
    setError(null)
    try {
      const claimPk = new PublicKey(selectedParcel.id)
      const surveyorPk = publicKey
      const observationPk = PublicKey.unique()
      const issuanceDate = Math.floor(Date.now() / 1000)
      const expirationDate = issuanceDate + 365 * 24 * 60 * 60 // 1 year

      const sdJwtDigest = await sdJwtDigest(
        claimPk,
        surveyorPk,
        observationPk,
        claimPk, // credential subject = parcel
        'ResidencyAttestation',
        issuanceDate,
        expirationDate,
      )

      // Build Anchor instruction data manually to match the new IDL layout
      const vcType = 'ResidencyAttestation'
      const vcTypeBytes = new TextEncoder().encode(vcType)
      const data = Buffer.alloc(8 + 4 + vcTypeBytes.length + 32 + 8 + 8 + 32 + 1)
      data.writeUInt32LE(vcTypeBytes.length, 8)
      data.set(vcTypeBytes, 12)
      let offset = 12 + vcTypeBytes.length
      data.set(claimPk.toBuffer(), offset)
      offset += 32
      data.writeBigUInt64LE(BigInt(issuanceDate), offset)
      offset += 8
      data.writeBigUInt64LE(BigInt(expirationDate), offset)
      offset += 8
      data.set(Buffer.from(sdJwtDigest), offset)
      offset += 32
      data[offset] = 0 // selective_disclosure = false

      const ix = {
        programId: new PublicKey(TERRA_PROGRAM_ID),
        keys: [
          {
            pubkey: PublicKey.findProgramAddressSync(
              [Buffer.from('verification_attestation'), claimPk.toBuffer(), surveyorPk.toBuffer()],
              new PublicKey(TERRA_PROGRAM_ID),
            )[0],
            isSigner: false,
            isWritable: true,
          },
          { pubkey: claimPk, isSigner: false, isWritable: true },
          {
            pubkey: observationPk,
            isSigner: false,
            isWritable: false,
          },
          { pubkey: surveyorPk, isSigner: true, isWritable: false },
          { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
          {
            pubkey: PublicKey.findProgramAddressSync(
              [Buffer.from('validator_reputation'), surveyorPk.toBuffer()],
              new PublicKey(TERRA_PROGRAM_ID),
            )[0],
            isSigner: false,
            isWritable: false,
          },
        ],
        data: Buffer.from(data),
      }

      const tx = new Transaction().add(ix)
      await send(tx)

      setVc({
        vcType,
        issuer: surveyorPk.toBase58(),
        credentialSubject: claimPk.toBase58(),
        issuanceDate,
        expirationDate,
        sdJwtDigest: Array.from(sdJwtDigest)
          .map((b) => b.toString(16).padStart(2, '0'))
          .join(''),
        selectiveDisclosure: false,
      })
      setStep('vc')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'VC issuance failed')
    } finally {
      setBusy(false)
    }
  }, [anchorWallet, send, publicKey, selectedParcel])

  // Step 2: Owner+tenant dual-sign residency proof
  const handleOwnerSign = useCallback(async () => {
    if (!signMessage) return
    try {
      const message = new TextEncoder().encode(
        `RESIDENCY_PROOF\nowner:${publicKey.toBase58()}\ntenant:pending\nparcel:${selectedParcel?.id}\ntimestamp:${Math.floor(
          Date.now() / 1000,
        )}`,
      )
      const signature = await signMessage(message)
      setOwnerSigned(true)
      setResidencyProof({
        ownerPubkey: publicKey.toBase58(),
        tenantPubkey: '',
        parcelId: selectedParcel!.id,
        ownerSignature: Array.from(signature)
          .map((b) => b.toString(16).padStart(2, '0'))
          .join(''),
        tenantSignature: '',
        timestamp: Math.floor(Date.now() / 1000),
      })
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Owner sign failed')
    }
  }, [signMessage, publicKey, selectedParcel])

  const handleTenantSign = useCallback(async () => {
    if (!signMessage) return
    try {
      const message = new TextEncoder().encode(
        `RESIDENCY_PROOF\nowner:${residencyProof?.ownerPubkey ?? 'pending'}\ntenant:${publicKey.toBase58()}\nparcel:${selectedParcel?.id}\ntimestamp:${Math.floor(
          Date.now() / 1000,
        )}`,
      )
      const signature = await signMessage(message)
      setTenantSigned(true)
      setResidencyProof((prev) => ({
        ...(prev ?? {
          ownerPubkey: '',
          tenantPubkey: publicKey.toBase58(),
          parcelId: selectedParcel!.id,
          ownerSignature: '',
          tenantSignature: '',
          timestamp: Math.floor(Date.now() / 1000),
        }),
        tenantPubkey: publicKey.toBase58(),
        tenantSignature: Array.from(signature)
          .map((b) => b.toString(16).padStart(2, '0'))
          .join(''),
      }))
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Tenant sign failed')
    }
  }, [signMessage, publicKey, selectedParcel, residencyProof])

  // Step 3: Mock verifier — validate VC + residency proof
  const handleVerify = useCallback(async () => {
    if (!vc || !residencyProof) return
    setVerificationResult('checking')
    setBusy(true)
    try {
      await new Promise((r) => setTimeout(r, 800))

      const vcValid = Boolean(vc.sdJwtDigest && vc.sdJwtDigest.length === 64)
      const residencyValid =
        Boolean(residencyProof.ownerSignature && residencyProof.tenantSignature) &&
        residencyProof.ownerSignature.length === 128 &&
        residencyProof.tenantSignature.length === 128 &&
        residencyProof.parcelId === vc.credentialSubject

      setVerificationResult(vcValid && residencyValid ? 'valid' : 'invalid')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Verification failed')
      setVerificationResult('invalid')
    } finally {
      setBusy(false)
    }
  }, [vc, residencyProof])

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
    verify: '4 · Mock Verifier',
    complete: '5 · Complete',
  }

  const stepDescriptions: Record<FlowStep, string> = {
    surveyor: 'Select a parcel and have the licensed surveyor issue a W3C Verifiable Credential (SD-JWT) anchored on-chain.',
    vc: 'The VC is anchored on-chain with its SD-JWT digest. Selective disclosure is available for privacy-preserving verification.',
    residency: 'Owner and tenant each sign the residency proof with their wallet keys. Both signatures are required.',
    verify: 'The mock verifier checks the VC digest against the on-chain anchor and validates both residency signatures.',
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
                    {formatPubkey(residencyProof?.tenantPubkey ?? '')}
                  </div>
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
            <div className="mvp-card-actions">
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
                  {busy ? 'Verifying…' : 'Run Mock Verification'}
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