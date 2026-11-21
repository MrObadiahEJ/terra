import { useCallback, useEffect, useState } from 'react'
import { useLocation, useNavigate } from 'react-router-dom'
import {
  AlertCircle,
  ArrowLeft,
  CheckCircle2,
  Clock,
  Copy,
  FileCheck,
  FileText,
  Fingerprint,
  RefreshCw,
  ShieldAlert,
  ShieldCheck,
  UserCheck,
  XCircle,
} from 'lucide-react'
import { PublicKey } from '@solana/web3.js'
import { connection } from '../lib/wallet'
import { TERRA_PROGRAM_ID } from '../lib/constants'
import {
  verifyCompletePresentation,
  type VerificationBundle,
  type VerificationReport,
} from '../lib/mvpVerifier'

const DEFAULT_SAMPLE_BUNDLE: VerificationBundle = {
  parcelId: 'e2e-verify-parcel',
  vc: {
    vcType: 'ResidencyAttestation',
    issuer: 'GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage',
    credentialSubject: '4uQeVj5tqViQh7yWWGStvkEG1Zmhx6uasJtWCJziofM',
    observation: '9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM',
    issuanceDate: 1732190000,
    expirationDate: 1763726000,
    sdJwtDigest: 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
    selectiveDisclosure: false,
    issuerSignature: '',
  },
  residencyProof: {
    ownerPubkey: 'GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage',
    tenantPubkey: '4uQeVj5tqViQh7yWWGStvkEG1Zmhx6uasJtWCJziofM',
    parcelId: 'e2e-verify-parcel',
    ownerSignature: '',
    tenantSignature: '',
    timestamp: Math.floor(Date.now() / 1000),
  },
}

export default function VerifierPage() {
  const location = useLocation()
  const navigate = useNavigate()

  const [rawJson, setRawJson] = useState('')
  const [report, setReport] = useState<VerificationReport | null>(null)
  const [verifying, setVerifying] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [copied, setCopied] = useState(false)
  const [activeTab, setActiveTab] = useState<'editor' | 'report'>('editor')

  // Load bundle from navigation state, localStorage, or query params
  useEffect(() => {
    try {
      if (location.state && (location.state as { bundle?: VerificationBundle }).bundle) {
        const bundle = (location.state as { bundle: VerificationBundle }).bundle
        setRawJson(JSON.stringify(bundle, null, 2))
        runVerification(bundle)
        return
      }

      const stored = localStorage.getItem('terra_mvp_presentation_bundle')
      if (stored) {
        const parsed = JSON.parse(stored)
        setRawJson(JSON.stringify(parsed, null, 2))
        runVerification(parsed)
        return
      }

      const params = new URLSearchParams(location.search)
      const b64 = params.get('bundle')
      if (b64) {
        const decoded = atob(b64)
        const parsed = JSON.parse(decoded)
        setRawJson(JSON.stringify(parsed, null, 2))
        runVerification(parsed)
        return
      }

      // Default sample
      setRawJson(JSON.stringify(DEFAULT_SAMPLE_BUNDLE, null, 2))
    } catch {
      setRawJson(JSON.stringify(DEFAULT_SAMPLE_BUNDLE, null, 2))
    }
  }, [location])

  const runVerification = useCallback(async (bundle: VerificationBundle) => {
    setVerifying(true)
    setError(null)
    try {
      const rep = await verifyCompletePresentation(
        bundle,
        connection,
        new PublicKey(TERRA_PROGRAM_ID),
      )
      setReport(rep)
      setActiveTab('report')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Verification error occurred')
    } finally {
      setVerifying(false)
    }
  }, [])

  const handleVerify = () => {
    try {
      const parsed = JSON.parse(rawJson) as VerificationBundle
      if (!parsed.vc || !parsed.residencyProof) {
        throw new Error('Bundle must include "vc" and "residencyProof" fields.')
      }
      runVerification(parsed)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Invalid JSON input.')
    }
  }

  const handleLoadSample = () => {
    setRawJson(JSON.stringify(DEFAULT_SAMPLE_BUNDLE, null, 2))
    setError(null)
  }

  const handleCopyCertificate = () => {
    if (!report) return
    const cert = {
      title: 'Terra Protocol — Verification Attestation Certificate',
      verifiedAt: new Date().toISOString(),
      overallValid: report.overallValid,
      report,
    }
    navigator.clipboard.writeText(JSON.stringify(cert, null, 2))
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <div className="st-page verifier-page max-w-5xl mx-auto px-4 py-8">
      {/* Header */}
      <header className="flex items-center justify-between mb-8 pb-4 border-b border-white/10">
        <div>
          <button
            onClick={() => navigate('/mvp')}
            className="flex items-center gap-1 text-xs text-muted hover:text-white mb-2 transition"
          >
            <ArrowLeft size={14} /> Back to MVP Flow
          </button>
          <h1 className="text-2xl font-bold flex items-center gap-2">
            <ShieldCheck className="text-emerald-400" size={26} /> Standalone Credential Verifier
          </h1>
          <p className="text-xs text-muted mt-1">
            Zero-knowledge, cryptographic, and on-chain verification of W3C Verifiable Credentials, SD-JWT Disclosures, and Residency Proofs.
          </p>
        </div>

        <div className="flex items-center gap-2">
          {report && (
            <div
              className={`px-3 py-1.5 rounded-full text-xs font-semibold flex items-center gap-1.5 ${
                report.overallValid
                  ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30'
                  : 'bg-amber-500/20 text-amber-400 border border-amber-500/30'
              }`}
            >
              {report.overallValid ? (
                <>
                  <CheckCircle2 size={14} /> PRESENTATION VALID
                </>
              ) : (
                <>
                  <AlertCircle size={14} /> VERIFICATION REJECTED
                </>
              )}
            </div>
          )}
        </div>
      </header>

      {/* Tabs */}
      <div className="flex gap-2 mb-6 border-b border-white/10 pb-2">
        <button
          onClick={() => setActiveTab('editor')}
          className={`px-4 py-2 text-sm rounded-lg transition ${
            activeTab === 'editor'
              ? 'bg-white/10 text-white font-medium'
              : 'text-muted hover:text-white'
          }`}
        >
          Presentation Bundle (JSON)
        </button>
        <button
          onClick={() => setActiveTab('report')}
          disabled={!report}
          className={`px-4 py-2 text-sm rounded-lg transition ${
            activeTab === 'report'
              ? 'bg-white/10 text-white font-medium'
              : !report
              ? 'opacity-40 cursor-not-allowed text-muted'
              : 'text-muted hover:text-white'
          }`}
        >
          Verification Audit Report
        </button>
      </div>

      {error && (
        <div className="p-3 mb-6 bg-rose-500/10 border border-rose-500/30 text-rose-400 rounded-lg text-sm flex items-center gap-2">
          <XCircle size={16} /> {error}
        </div>
      )}

      {/* Tab: Editor */}
      {activeTab === 'editor' && (
        <div className="space-y-4">
          <div className="flex justify-between items-center text-xs text-muted">
            <span>Paste presentation bundle JSON containing VC and ResidencyProof:</span>
            <button
              onClick={handleLoadSample}
              className="text-primary hover:underline flex items-center gap-1"
            >
              <RefreshCw size={12} /> Reset to Sample
            </button>
          </div>

          <textarea
            value={rawJson}
            onChange={(e) => setRawJson(e.target.value)}
            rows={14}
            className="w-full font-mono text-xs p-4 bg-black/40 border border-white/10 rounded-xl focus:border-primary focus:outline-none"
            placeholder="Paste JSON here..."
          />

          <div className="flex justify-end gap-3 pt-2">
            <button
              onClick={handleVerify}
              disabled={verifying}
              className="btn btn-primary flex items-center gap-2 px-6 py-2.5 rounded-lg text-sm font-semibold"
            >
              {verifying ? (
                <>
                  <RefreshCw className="animate-spin" size={16} /> Verifying Checks...
                </>
              ) : (
                <>
                  <ShieldCheck size={16} /> Verify Presentation
                </>
              )}
            </button>
          </div>
        </div>
      )}

      {/* Tab: Verification Report */}
      {activeTab === 'report' && report && (
        <div className="space-y-6">
          {/* Summary Banner */}
          <div
            className={`p-5 rounded-xl border flex items-start justify-between ${
              report.overallValid
                ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-300'
                : 'bg-amber-500/10 border-amber-500/30 text-amber-300'
            }`}
          >
            <div>
              <div className="text-lg font-bold flex items-center gap-2">
                {report.overallValid ? (
                  <>
                    <CheckCircle2 size={20} /> Presentation Verified Authentically
                  </>
                ) : (
                  <>
                    <ShieldAlert size={20} /> Verification Failed One or More Checks
                  </>
                )}
              </div>
              <p className="text-xs text-muted mt-1">
                {report.overallValid
                  ? 'All cryptographic signatures, selective disclosure hashes, temporal freshness, and on-chain quorum anchors are valid.'
                  : 'The presented credentials do not satisfy the required cryptographic or on-chain quorum conditions.'}
              </p>
            </div>

            <button
              onClick={handleCopyCertificate}
              className="btn btn-secondary text-xs flex items-center gap-1.5 px-3 py-1.5 rounded-lg"
            >
              <Copy size={13} /> {copied ? 'Copied!' : 'Copy Audit Certificate'}
            </button>
          </div>

          {/* 5-Step Verification Breakdown */}
          <div className="grid gap-4">
            {/* 1. Issuer Signature */}
            <div className="p-4 bg-white/[0.02] border border-white/10 rounded-xl">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <div
                    className={`w-7 h-7 rounded-full flex items-center justify-center text-xs ${
                      report.issuerSignature.valid
                        ? 'bg-emerald-500/20 text-emerald-400'
                        : 'bg-rose-500/20 text-rose-400'
                    }`}
                  >
                    1
                  </div>
                  <div>
                    <h3 className="text-sm font-semibold flex items-center gap-2">
                      <Fingerprint size={16} /> W3C VC Issuer Signature
                    </h3>
                    <p className="text-xs text-muted">
                      Ed25519 cryptographic signature over canonical VC payload
                    </p>
                  </div>
                </div>
                <span
                  className={`text-xs px-2.5 py-1 rounded-full font-mono ${
                    report.issuerSignature.valid
                      ? 'bg-emerald-500/20 text-emerald-400'
                      : 'bg-rose-500/20 text-rose-400'
                  }`}
                >
                  {report.issuerSignature.valid ? 'PASSED' : 'FAILED'}
                </span>
              </div>
              <div className="mt-3 text-xs font-mono text-muted pl-10">
                {report.issuerSignature.details}
              </div>
            </div>

            {/* 2. SD-JWT Selective Disclosure */}
            <div className="p-4 bg-white/[0.02] border border-white/10 rounded-xl">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <div
                    className={`w-7 h-7 rounded-full flex items-center justify-center text-xs ${
                      report.sdJwtDisclosure.valid
                        ? 'bg-emerald-500/20 text-emerald-400'
                        : 'bg-rose-500/20 text-rose-400'
                    }`}
                  >
                    2
                  </div>
                  <div>
                    <h3 className="text-sm font-semibold flex items-center gap-2">
                      <FileCheck size={16} /> SD-JWT Selective Disclosure Digest
                    </h3>
                    <p className="text-xs text-muted">
                      SHA-256 pre-image matching against claim, validator, observation, and subject
                    </p>
                  </div>
                </div>
                <span
                  className={`text-xs px-2.5 py-1 rounded-full font-mono ${
                    report.sdJwtDisclosure.valid
                      ? 'bg-emerald-500/20 text-emerald-400'
                      : 'bg-rose-500/20 text-rose-400'
                  }`}
                >
                  {report.sdJwtDisclosure.valid ? 'PASSED' : 'FAILED'}
                </span>
              </div>
              <div className="mt-3 text-xs font-mono text-muted pl-10 space-y-1">
                <div>Status: {report.sdJwtDisclosure.details}</div>
                {report.sdJwtDisclosure.digest && (
                  <div className="text-[11px] break-all opacity-80">
                    Recomputed Digest: {report.sdJwtDisclosure.digest}
                  </div>
                )}
              </div>
            </div>

            {/* 3. Dual-Signature Residency */}
            <div className="p-4 bg-white/[0.02] border border-white/10 rounded-xl">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <div
                    className={`w-7 h-7 rounded-full flex items-center justify-center text-xs ${
                      report.residencyDualSign.valid
                        ? 'bg-emerald-500/20 text-emerald-400'
                        : 'bg-rose-500/20 text-rose-400'
                    }`}
                  >
                    3
                  </div>
                  <div>
                    <h3 className="text-sm font-semibold flex items-center gap-2">
                      <UserCheck size={16} /> Owner & Tenant Dual-Signature
                    </h3>
                    <p className="text-xs text-muted">
                      Dual Ed25519 authorization over canonical residency agreement message
                    </p>
                  </div>
                </div>
                <span
                  className={`text-xs px-2.5 py-1 rounded-full font-mono ${
                    report.residencyDualSign.valid
                      ? 'bg-emerald-500/20 text-emerald-400'
                      : 'bg-rose-500/20 text-rose-400'
                  }`}
                >
                  {report.residencyDualSign.valid ? 'PASSED' : 'FAILED'}
                </span>
              </div>
              <div className="mt-3 text-xs font-mono text-muted pl-10 space-y-1">
                <div>{report.residencyDualSign.details}</div>
                <div className="flex gap-4 pt-1 text-[11px]">
                  <span>Owner Key: {report.residencyDualSign.ownerValid ? 'Verified' : 'Invalid'}</span>
                  <span>Tenant Key: {report.residencyDualSign.tenantValid ? 'Verified' : 'Invalid'}</span>
                </div>
              </div>
            </div>

            {/* 4. Temporal Validity */}
            <div className="p-4 bg-white/[0.02] border border-white/10 rounded-xl">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <div
                    className={`w-7 h-7 rounded-full flex items-center justify-center text-xs ${
                      report.temporalValidity.valid
                        ? 'bg-emerald-500/20 text-emerald-400'
                        : 'bg-rose-500/20 text-rose-400'
                    }`}
                  >
                    4
                  </div>
                  <div>
                    <h3 className="text-sm font-semibold flex items-center gap-2">
                      <Clock size={16} /> Temporal Validity & Freshness
                    </h3>
                    <p className="text-xs text-muted">
                      VC expiration window check and proof timestamp skew limit (≤10m)
                    </p>
                  </div>
                </div>
                <span
                  className={`text-xs px-2.5 py-1 rounded-full font-mono ${
                    report.temporalValidity.valid
                      ? 'bg-emerald-500/20 text-emerald-400'
                      : 'bg-rose-500/20 text-rose-400'
                  }`}
                >
                  {report.temporalValidity.valid ? 'PASSED' : 'FAILED'}
                </span>
              </div>
              <div className="mt-3 text-xs font-mono text-muted pl-10">
                {report.temporalValidity.details}
              </div>
            </div>

            {/* 5. On-Chain Anchor & Validator Quorum */}
            {report.onChainAnchorAndQuorum && (
              <div className="p-4 bg-white/[0.02] border border-white/10 rounded-xl">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <div
                      className={`w-7 h-7 rounded-full flex items-center justify-center text-xs ${
                        report.onChainAnchorAndQuorum.valid
                          ? 'bg-emerald-500/20 text-emerald-400'
                          : 'bg-amber-500/20 text-amber-400'
                      }`}
                    >
                      5
                    </div>
                    <div>
                      <h3 className="text-sm font-semibold flex items-center gap-2">
                        <FileText size={16} /> On-Chain Anchor & Validator Quorum
                      </h3>
                      <p className="text-xs text-muted">
                        Solana program state: VerificationAttestation PDA & validator registry status
                      </p>
                    </div>
                  </div>
                  <span
                    className={`text-xs px-2.5 py-1 rounded-full font-mono ${
                      report.onChainAnchorAndQuorum.valid
                        ? 'bg-emerald-500/20 text-emerald-400'
                        : 'bg-amber-500/20 text-amber-400'
                    }`}
                  >
                    {report.onChainAnchorAndQuorum.valid ? 'PASSED' : 'NOT ANCHORED'}
                  </span>
                </div>
                <div className="mt-3 text-xs font-mono text-muted pl-10 space-y-1">
                  <div>{report.onChainAnchorAndQuorum.details}</div>
                  {report.onChainAnchorAndQuorum.attestationPda && (
                    <div className="text-[11px] break-all opacity-80">
                      PDA: {report.onChainAnchorAndQuorum.attestationPda}
                    </div>
                  )}
                  <div className="flex gap-4 pt-1 text-[11px]">
                    <span>Attestation: {report.onChainAnchorAndQuorum.attestationFound ? 'Found' : 'Missing'}</span>
                    <span>Validator Registered: {report.onChainAnchorAndQuorum.validatorRegistered ? 'Yes' : 'No'}</span>
                    <span>Quorum Satisfied: {report.onChainAnchorAndQuorum.quorumAchieved ? 'Yes' : 'Pending'}</span>
                  </div>
                </div>
              </div>
            )}
          </div>

          <div className="flex justify-end pt-4">
            <button
              onClick={() => setActiveTab('editor')}
              className="btn btn-secondary text-xs px-4 py-2"
            >
              Edit Presentation Bundle
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
