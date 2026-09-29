import { useEffect, useRef, useState } from 'react'
import { PublicKey } from '@solana/web3.js'
import { sha256Bytes, sha256Hex } from '../../lib/geo'
import { ClipboardCheck, Gavel, RotateCcw, Search, ShieldQuestion } from 'lucide-react'

// --- program constants (verification/*.rs) ---------------------------------

const CLAIM_TYPES = [
  'PARCEL_EXISTS', 'BOUNDARY', 'OWNERSHIP', 'USAGE', 'LEASE', 'EASEMENT', 'LIEN',
  'MORTGAGE', 'OCCUPANCY', 'TRANSACTION', 'SUBDIVISION', 'AMALGAMATION',
  'PROPERTY_ATTRIBUTE', 'INFRASTRUCTURE',
] as const

const EVIDENCE_TYPES = [
  'PERSON_SUBMISSION', 'PHOTO', 'VIDEO', 'DOCUMENT', 'SURVEY', 'GPS_OBSERVATION',
  'PHYSICAL_OBSERVATION', 'WITNESS_STATEMENT', 'TRANSACTION_RECORD',
  'PROPERTY_RECORD', 'MAP', 'SATELLITE_OBSERVATION', 'SIGNATURE',
] as const

const RESULTS = ['CONFIRMED', 'DISPUTED', 'UNABLE_TO_VERIFY'] as const

const CLAIM_STATUS = ['SUBMITTED', 'UNDER_VERIFICATION', 'VERIFIED', 'REJECTED', 'CHALLENGED'] as const
const STATUS_CLASS = ['warn', 'info', 'ok', 'err', 'warn'] as const

const CHALLENGE_STATUS = ['FILED', 'UNDER_REVIEW', 'UPHELD', 'OVERTURNED'] as const

interface Attestation {
  validator: number
  result: number
  confidence: number
}

interface Claim {
  claimId: string
  statementHash: string
  type: number
  status: number
  required: number
  evidence: number[]
  observationCount: number
  attestations: Attestation[]
}

interface Challenge {
  requiredVotes: number
  status: number
  voters: number[]
  uphold: number
  overturn: number
  deadline: string
}

interface LogEntry {
  id: number
  event: string
  detail: string
  tone: 'info' | 'ok' | 'warn' | 'err'
}

export default function VerificationPipeline() {
  const [wallets, setWallets] = useState<string[]>([])
  useEffect(() => {
    let cancelled = false
    ;(async () => {
      const ws = await Promise.all(
        [1, 2, 3].map(async (i) =>
          new PublicKey(await sha256Bytes(`terra-lab-validator-${i}`)).toBase58(),
        ),
      )
      if (!cancelled) setWallets(ws)
    })()
    return () => {
      cancelled = true
    }
  }, [])

  const [claim, setClaim] = useState<Claim | null>(null)
  const [challenge, setChallenge] = useState<Challenge | null>(null)
  const [log, setLog] = useState<LogEntry[]>([])
  const [err, setErr] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)
  const seq = useRef(0)

  const [claimType, setClaimType] = useState(0)
  const [required, setRequired] = useState(2)
  const [results, setResults] = useState([0, 0, 0])
  const [confs, setConfs] = useState([90, 85, 80])
  const [challengeVotes, setChallengeVotes] = useState(2)

  const push = (event: string, detail: string, tone: LogEntry['tone'] = 'info') =>
    setLog((l) => [{ id: ++seq.current, event, detail, tone }, ...l])

  const reset = () => {
    setClaim(null)
    setChallenge(null)
    setLog([])
    setErr(null)
    setNotice(null)
    seq.current = 0
  }

  const statusOk = (allowed: number[]): string | null => {
    if (!claim) return 'InvalidClaimStatus (6131): no claim'
    if (!allowed.includes(claim.status)) {
      return `InvalidClaimStatus (6131): claim.status = ${CLAIM_STATUS[claim.status]} (needs ${allowed
        .map((s) => CLAIM_STATUS[s])
        .join(' / ')})`
    }
    return null
  }

  const fail = (m: string) => {
    setErr(m)
    setNotice(null)
  }
  const ok = (m: string) => {
    setNotice(m)
    setErr(null)
  }

  // 1) create_claim ---------------------------------------------------------
  const createClaim = async () => {
    const statementHash = await sha256Hex(`statement:${claimType}:demo-parcel`)
    const claimId = await sha256Hex(`claim:${claimType}:demo-parcel`)
    if (claimId === '0'.repeat(64)) return fail('EmptyClaimId (6128)')
    if (claimType < 0 || claimType > 13) return fail('InvalidClaimType (6129)')
    if (statementHash === '0'.repeat(64)) return fail('EmptyStatementHash (6130)')
    if (required < 1) return fail('InvalidRequiredAttestations (6139)')
    setClaim({
      claimId,
      statementHash,
      type: claimType,
      status: 0,
      required,
      evidence: [],
      observationCount: 0,
      attestations: [],
    })
    push(
      'ClaimCreated',
      `claim_id=${claimId.slice(0, 16)}… type=${CLAIM_TYPES[claimType]} required_attestations=${required} (QuorumConfig or default 2)`,
      'ok',
    )
    ok('Claim submitted (status = SUBMITTED)')
    return null
  }

  // 2) add_evidence ---------------------------------------------------------
  const addEvidence = (type: number) => {
    const e = statusOk([0, 1])
    if (e) return fail(e)
    if (type < 0 || type > 12) return fail('InvalidEvidenceType (6133)')
    setClaim((c) => (c ? { ...c, evidence: [...c.evidence, type] } : c))
    push('EvidenceAdded', `evidence_type=${type} (${EVIDENCE_TYPES[type]})`, 'ok')
    setErr(null)
  }

  // 3) submit_observation ---------------------------------------------------
  const submitObservation = () => {
    const e = statusOk([0, 1])
    if (e) return fail(e)
    const first = claim!.observationCount === 0
    const nextCount = claim!.observationCount + 1
    setClaim((c) =>
      c ? { ...c, observationCount: c.observationCount + 1, status: first ? 1 : c.status } : c,
    )
    push(
      'ObservationSubmitted',
      first
        ? `observation_count = ${nextCount} → claim.status = UNDER_VERIFICATION`
        : `observation_count = ${nextCount}`,
      'ok',
    )
    setErr(null)
  }

  // 4) submit_attestation ---------------------------------------------------
  const submitAttestation = (v: number) => {
    const e = statusOk([0, 1])
    if (e) return fail(e)
    const result = results[v]
    const confidence = confs[v]
    if (result > 2) return fail('InvalidAttestationResult (6136)')
    if (confidence < 0 || confidence > 100) return fail('InvalidConfidence (6135): 0..=100 required')
    if (claim!.attestations.some((a) => a.validator === v)) {
      return fail('AlreadyEndorsedRotation (6049): this validator already attested')
    }
    const attestations = [...claim!.attestations, { validator: v, result, confidence }]
    const nextCount = attestations.filter((a) => a.result === 0).length
    setClaim((c) => (c ? { ...c, attestations } : c))
    push(
      'VerificationAttestationSubmitted',
      `validator=${v + 1} result=${RESULTS[result]} confidence=${confidence} → attestation_count=${nextCount}/${claim!.required}` +
        (result === 0 ? '' : ' (only CONFIRMED counts toward quorum)'),
      'info',
    )
    setErr(null)
  }

  // 5) verify_claim ---------------------------------------------------------
  const verifyClaim = () => {
    if (!claim) return fail('InvalidClaimStatus (6131): no claim')
    if (claim.status !== 0 && claim.status !== 1) {
      return fail(`InvalidClaimStatus (6131): claim.status = ${CLAIM_STATUS[claim.status]}`)
    }
    const count = claim.attestations.filter((a) => a.result === 0).length
    if (count < claim.required) {
      return fail(
        `QuorumNotReached (6137): attestation_count = ${count} < required_attestations = ${claim.required}`,
      )
    }
    setClaim((c) => (c ? { ...c, status: 2 } : c))
    push(
      'ClaimVerified',
      `attestation_count=${count}/${claim.required} → claim.status = VERIFIED (version bumped)`,
      'ok',
    )
    ok('Claim verified — a challenger can now file a challenge within 14 days.')
  }

  // 6) file_challenge / vote_challenge -------------------------------------
  const fileChallenge = async () => {
    if (!claim) return fail('InvalidClaimStatus (6131): no claim')
    if (claim.status !== 2) {
      return fail(
        `InvalidClaimStatus (6131): file_challenge requires VERIFIED, got ${CLAIM_STATUS[claim.status]}`,
      )
    }
    if (challengeVotes < 1) return fail('InvalidThreshold (6016): required_votes must be >= 1')
    const hash = await sha256Hex(`challenge:${claim.claimId}`)
    if (hash === '0'.repeat(64)) return fail('EmptyStatementHash (6130)')
    const deadline = new Date(new Date().getTime() + 14 * 24 * 3600 * 1000).toISOString()
    setChallenge({
      requiredVotes: challengeVotes,
      status: 0,
      voters: [],
      uphold: 0,
      overturn: 0,
      deadline,
    })
    setClaim((c) => (c ? { ...c, status: 4 } : c))
    push(
      'ChallengeFiled',
      `required_votes=${challengeVotes} review_deadline=${deadline.slice(0, 19).replace('T', ' ')} UTC → claim.status = CHALLENGED`,
      'warn',
    )
    ok('Challenge filed — validators vote: uphold quorum rejects the claim, overturn quorum keeps it verified.')
  }

  const vote = (v: number, uphold: boolean) => {
    if (!challenge) return fail('InvalidClaimStatus (6131): no challenge')
    if (challenge.status > 1) {
      return fail(`InvalidClaimStatus (6131): challenge is ${CHALLENGE_STATUS[challenge.status]}`)
    }
    if (challenge.voters.includes(v)) {
      return fail('AlreadyEndorsedRotation (6049): validator already voted on this challenge')
    }
    const ch = challenge
    const voters = [...ch.voters, v]
    const upholdCount = ch.uphold + (uphold ? 1 : 0)
    const overturnCount = ch.overturn + (uphold ? 0 : 1)
    let status = ch.status === 0 ? 1 : ch.status
    if (upholdCount >= ch.requiredVotes) status = 2
    else if (overturnCount >= ch.requiredVotes) status = 3

    setChallenge({ ...ch, voters, uphold: upholdCount, overturn: overturnCount, status })
    push(
      'ChallengeVoteRecorded',
      `validator=${v + 1} vote=${uphold ? 'UPHOLD' : 'OVERTURN'} → uphold=${upholdCount} overturn=${overturnCount}`,
      'info',
    )

    if (status === 2 || status === 3) {
      const upheld = status === 2
      push(
        'ChallengeResolved',
        `outcome=${CHALLENGE_STATUS[status]} → claim.status = ${upheld ? 'REJECTED' : 'VERIFIED'}`,
        upheld ? 'err' : 'ok',
      )
      setClaim((c) => (c ? { ...c, status: upheld ? 3 : 2 } : c))
      setNotice(
        upheld
          ? 'Challenge UPHELD — claim rejected by governance.'
          : 'Challenge OVERTURNED — claim stands, verified again.',
      )
      setErr(null)
    }
  }

  const count = claim ? claim.attestations.filter((a) => a.result === 0).length : 0

  return (
    <div className="lab-body">
      <div className="lab-grid">
        {/* left: steps */}
        <div className="lab-col space-y-3">
          <div className="lab-card space-y-2">
            <div className="flex items-center justify-between">
              <h3 className="text-[13px] font-semibold flex items-center gap-1.5">
                <ClipboardCheck size={14} /> 1 · create_claim
              </h3>
              {claim && (
                <button className="btn btn-ghost p-1" onClick={reset} title="Reset experiment">
                  <RotateCcw size={13} />
                </button>
              )}
            </div>
            <div className="grid grid-cols-2 gap-x-3 gap-y-1">
              <label className="text-[11px] text-muted">
                claim_type (u8)
                <select
                  className="select-input"
                  value={claimType}
                  disabled={!!claim}
                  onChange={(e) => setClaimType(Number(e.target.value))}
                >
                  {CLAIM_TYPES.map((t, i) => (
                    <option key={t} value={i}>
                      {i} = {t}
                    </option>
                  ))}
                </select>
              </label>
              <label className="text-[11px] text-muted">
                required_attestations
                <select
                  className="select-input"
                  value={required}
                  disabled={!!claim}
                  onChange={(e) => setRequired(Number(e.target.value))}
                >
                  {[1, 2, 3].map((n) => (
                    <option key={n} value={n}>
                      {n}
                    </option>
                  ))}
                </select>
              </label>
            </div>
            {!claim ? (
              <button className="btn btn-primary w-full justify-center" onClick={createClaim}>
                Create claim
              </button>
            ) : (
              <p className="font-mono text-[10px] break-all text-muted">
                claim_id {claim.claimId.slice(0, 24)}… · statement {claim.statementHash.slice(0, 24)}…
              </p>
            )}
          </div>

          <div className="lab-card space-y-2">
            <h3 className="text-[13px] font-semibold">2 · add_evidence + submit_observation</h3>
            <p className="text-[11px] text-muted">
              Evidence chips attach while the claim is not terminal; the first observation flips the claim to
              UNDER_VERIFICATION.
            </p>
            <div className="flex flex-wrap gap-1">
              {EVIDENCE_TYPES.map((t, i) => (
                <button
                  key={t}
                  className="btn btn-secondary px-2 py-1 text-[10px]"
                  disabled={!claim || claim.status > 1}
                  onClick={() => addEvidence(i)}
                >
                  {t}
                </button>
              ))}
            </div>
            <button
              className="btn btn-secondary w-full justify-center gap-1.5"
              disabled={!claim || claim.status > 1}
              onClick={submitObservation}
            >
              <Search size={13} /> Submit observation ({claim?.observationCount ?? 0})
            </button>
          </div>

          <div className="lab-card space-y-2">
            <h3 className="text-[13px] font-semibold">3 · submit_attestation (per validator)</h3>
            {[0, 1, 2].map((vi) => {
              const done = claim?.attestations.some((a) => a.validator === vi) ?? false
              return (
                <div key={vi} className="lab-attestation">
                  <div className="flex items-center justify-between gap-2 min-w-0">
                    <span className="text-[12px] font-medium">Validator {vi + 1}</span>
                    <span className="font-mono text-[10px] text-muted truncate">
                      {wallets[vi] ?? 'deriving wallet…'}
                    </span>
                  </div>
                  <div className="lab-attestation-controls">
                    <select
                      className="select-input"
                      value={results[vi]}
                      disabled={done || !claim}
                      onChange={(e) =>
                        setResults((r) => r.map((x, i) => (i === vi ? Number(e.target.value) : x)))
                      }
                    >
                      {RESULTS.map((r, i) => (
                        <option key={r} value={i}>
                          {i} = {r}
                        </option>
                      ))}
                    </select>
                    <input
                      className="text-input"
                      type="number"
                      min={0}
                      max={100}
                      value={confs[vi]}
                      disabled={done || !claim}
                      onChange={(e) => setConfs((c) => c.map((x, i) => (i === vi ? Number(e.target.value) : x)))}
                      title="confidence 0-100"
                    />
                    <button
                      className="btn btn-secondary px-2 py-1 shrink-0"
                      disabled={done || !claim}
                      onClick={() => submitAttestation(vi)}
                    >
                      {done ? '✓' : 'Attest'}
                    </button>
                  </div>
                </div>
              )
            })}
            <p className="text-[10px] text-muted">
              digest guard: the program re-derives canonical_attestation_digest(claim, validator, observation,
              result, confidence) — mismatches reject with AttestationDigestMismatch (6158).
            </p>
          </div>

          <div className="lab-card space-y-2">
            <h3 className="text-[13px] font-semibold">4 · verify_claim</h3>
            <div className="lab-quorum" aria-hidden>
              {Array.from({ length: Math.max(required, 1) }).map((_, i) => (
                <span key={i} className={`lab-quorum-cell ${i < count ? 'on' : ''}`} />
              ))}
            </div>
            <p className="text-[11px] text-muted">
              attestation_count <b className="font-mono">{count}</b> / required_attestations{' '}
              <b className="font-mono">{claim?.required ?? '—'}</b> — only CONFIRMED attestations increment the
              counter; anyone may call verify_claim once quorum is met.
            </p>
            <button
              className="btn btn-primary w-full justify-center"
              disabled={!claim}
              onClick={verifyClaim}
            >
              Verify claim (read-then-write quorum check)
            </button>
          </div>

          <div className="lab-card space-y-2">
            <h3 className="text-[13px] font-semibold flex items-center gap-1.5">
              <Gavel size={14} /> 5 · file_challenge + vote_challenge
            </h3>
            {!challenge ? (
              <>
                <label className="text-[11px] text-muted">
                  required_votes (u8, minimum 1)
                  <select
                    className="select-input"
                    value={challengeVotes}
                    onChange={(e) => setChallengeVotes(Number(e.target.value))}
                  >
                    {[1, 2, 3].map((n) => (
                      <option key={n} value={n}>
                        {n}
                      </option>
                    ))}
                  </select>
                </label>
                <button
                  className="btn btn-secondary w-full justify-center"
                  disabled={!claim || claim.status !== 2}
                  onClick={fileChallenge}
                >
                  File challenge (requires VERIFIED claim)
                </button>
              </>
            ) : (
              <>
                <p className="text-[11px] text-muted">
                  Status <b>{CHALLENGE_STATUS[challenge.status]}</b> · uphold {challenge.uphold} / overturn{' '}
                  {challenge.overturn} · needs {challenge.requiredVotes} · deadline{' '}
                  {challenge.deadline.slice(0, 10)}
                </p>
                {[0, 1, 2].map((vi) => {
                  const voted = challenge.voters.includes(vi)
                  const done = challenge.status > 1
                  return (
                    <div key={vi} className="lab-validator">
                      <span className="text-[12px] flex-1">Validator {vi + 1}</span>
                      <button
                        className="btn btn-secondary px-2 py-1"
                        disabled={voted || done}
                        onClick={() => vote(vi, false)}
                      >
                        Overturn
                      </button>
                      <button
                        className="btn btn-secondary px-2 py-1"
                        disabled={voted || done}
                        onClick={() => vote(vi, true)}
                      >
                        Uphold
                      </button>
                    </div>
                  )
                })}
                <p className="text-[10px] text-muted">
                  Per program: uphold quorum → claim REJECTED, overturn quorum → claim VERIFIED. Only
                  registered validators may vote (NotValidator 6028 otherwise), once each (6049), within the
                  14-day review window.
                </p>
              </>
            )}
          </div>
        </div>

        {/* right: state + events */}
        <div className="lab-col space-y-3">
          <div className="lab-card">
            <div className="flex items-center justify-between mb-2">
              <h3 className="text-[13px] font-semibold">Claim state</h3>
              {claim && (
                <span className={`lab-badge lab-badge-${STATUS_CLASS[claim.status]}`}>
                  {CLAIM_STATUS[claim.status]}
                </span>
              )}
            </div>
            {!claim ? (
              <p className="text-[12px] text-muted">No claim yet — start at step 1.</p>
            ) : (
              <dl className="text-[11px] space-y-1">
                <div className="flex justify-between">
                  <dt className="text-muted">claim_type</dt>
                  <dd className="font-mono">{claim.type} = {CLAIM_TYPES[claim.type]}</dd>
                </div>
                <div className="flex justify-between">
                  <dt className="text-muted">evidence_count</dt>
                  <dd className="font-mono">{claim.evidence.length}</dd>
                </div>
                <div className="flex justify-between">
                  <dt className="text-muted">observation_count</dt>
                  <dd className="font-mono">{claim.observationCount}</dd>
                </div>
                <div className="flex justify-between">
                  <dt className="text-muted">attestation_count</dt>
                  <dd className="font-mono">
                    {count} / {claim.required}
                  </dd>
                </div>
                <div className="flex justify-between">
                  <dt className="text-muted">version</dt>
                  <dd className="font-mono">{claim.status >= 2 ? 2 : 1}</dd>
                </div>
              </dl>
            )}
          </div>

          <div className="lab-card">
            <div className="flex items-center justify-between mb-2">
              <h3 className="text-[13px] font-semibold flex items-center gap-1.5">
                <ShieldQuestion size={14} /> Event log
              </h3>
              <span className="text-[10px] text-muted">emitted exactly as on-chain</span>
            </div>
            {log.length === 0 ? (
              <p className="text-[12px] text-muted">Events will appear here.</p>
            ) : (
              <ul className="lab-log">
                {log.map((e) => (
                  <li key={e.id} className={`lab-log-entry tone-${e.tone}`}>
                    <span className="font-mono font-medium">{e.event}</span>
                    <span className="block text-muted break-all">{e.detail}</span>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </div>
      </div>

      {err && <p className="lab-msg err">{err}</p>}
      {notice && <p className="lab-msg ok">{notice}</p>}
    </div>
  )
}
