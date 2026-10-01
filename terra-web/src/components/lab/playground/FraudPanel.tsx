import { useState } from 'react'
import { ShieldAlert } from 'lucide-react'
import { Broadcast, ClockBar, DemoBanner, Field, VerdictCard } from './shared'
import { SIM_START, DAY, HOUR, err, pushGate, type Gate, type Verdict } from './helpers'

const R_STATUS = ['OPEN', 'UNDER_REVIEW', 'UPHELD', 'DISMISSED', 'APPEALED'] as const
const APPEAL_DELAY = 24 * HOUR // fraud_governance.rs RESTRICTION_APPEAL_DELAY_SECS
const REHAB = 30 * DAY // RESTRICTION_REHAB_SECS
const REASONS = ['FALSIFIED_OBSERVATION', 'COLLUSION', 'IDENTITY_MISUSE', 'EVIDENCE_TAMPERING', 'OTHER']
const COMMITTEE = ['C-1', 'C-2', 'C-3', 'C-4', 'C-5']

type FAction =
  | 'submit_fraud_report'
  | 'open_fraud_review'
  | 'cast_fraud_vote'
  | 'finalize_fraud_review'
  | 'file_fraud_appeal'
  | 'rehabilitate_restriction'

interface Call {
  ix: FAction
  ok: boolean
  summary: string
  verdict: Verdict
}

export default function FraudPanel() {
  const [report, setReport] = useState<{
    status: number
    reason: number
    reporter: string
    accused: string
    createdAt: number
  } | null>(null)
  const [restriction, setRestriction] = useState<{ status: number; appealOpensAt: number; rehabAt: number } | null>(null)
  const [votes, setVotes] = useState<Record<string, boolean>>({})
  const [voteChoice, setVoteChoice] = useState(true)
  const [voter, setVoter] = useState('C-1')
  const [reason, setReason] = useState(0)
  const [reporter, setReporter] = useState('DemoReporter111111111111111111111111111')
  const [accused, setAccused] = useState('DemoAccused1111111111111111111111111111')
  const [wallet, setWallet] = useState('DemoAccused1111111111111111111111111111')
  const [oneUnregistered, setOneUnregistered] = useState(false)
  const [now, setNow] = useState(SIM_START)
  const [last, setLast] = useState<Call | null>(null)

  const pool = oneUnregistered ? COMMITTEE.slice(0, 4) : COMMITTEE
  const votedCount = Object.keys(votes).length
  const allVoted = pool.every((c) => votes[c] !== undefined)
  const upheldVotes = Object.values(votes).filter(Boolean).length
  const decisionUpheld = upheldVotes * 2 > pool.length

  const run = (ix: FAction) => {
    const gates: Gate[] = []
    let ok = true
    let summary = ''
    const fail = (name: string, detail: string) => {
      if (ok) {
        ok = false
        summary = `Rejected — ${err(name)} — ${detail}`
      }
      pushGate(gates, name, false, detail)
    }
    const pass = (label: string, detail: string) => pushGate(gates, label, true, detail)

    switch (ix) {
      case 'submit_fraud_report': {
        if (reporter === accused) fail('SelfFraudReport', 'reporter == accused — self-reporting rejected')
        else pass('reporter ≠ accused', 'independent reporter')
        if (reason > 4) fail('InvalidFraudReason', `reason_code = ${reason} > fraud_reason::MAX = 4`)
        else pass('reason_code ≤ MAX', `${reason} = ${REASONS[reason]}`)
        if (report !== null && report.status < 2 && report.status !== 3) fail('InvalidStatus', `open report already exists (status = ${R_STATUS[report.status]})`)
        else pass('no open report', 'fraud report PDA free')
        if (ok) {
          setReport({ status: 0, reason, reporter, accused, createdAt: now })
          setVotes({})
          setRestriction(null)
          summary = `FraudReportSubmitted — reason = ${REASONS[reason]}, accused = ${accused.slice(0, 12)}… (accused is NOT auto-punished)`
        }
        break
      }
      case 'open_fraud_review': {
        if (!report || report.status !== 0) fail('InvalidFraudStatus', `open requires OPEN(0), status = ${report ? R_STATUS[report.status] : 'no report'}`)
        else pass('report.status == OPEN', 'OPEN')
        const bad = COMMITTEE.find((c) => !pool.includes(c))
        if (bad) fail('NotValidator', `${bad} not in registry.validators — review pool filtered`)
        else pass('committee ∈ registry.validators', `${pool.length} validators in pool`)
        if (ok && report) {
          setReport({ ...report, status: 1 })
          summary = `FraudReviewOpened — ${pool.length}-validator committee, threshold = majority (${Math.floor(pool.length / 2) + 1})`
        }
        break
      }
      case 'cast_fraud_vote': {
        if (!report || report.status !== 1) fail('InvalidFraudStatus', `vote requires UNDER_REVIEW(1), status = ${report ? R_STATUS[report.status] : 'no report'}`)
        else pass('report.status == UNDER_REVIEW', 'UNDER_REVIEW')
        if (votes[voter] !== undefined) fail('DuplicateCommitteeVote', `${voter} already voted (${votes[voter] ? 'uphold' : 'dismiss'})`)
        else pass('first vote from this validator', `${voter} fresh`)
        if (ok) {
          setVotes({ ...votes, [voter]: voteChoice })
          summary = `FraudVoteCast — ${voter} ${voteChoice ? 'UPHOLD' : 'DISMISS'} (${votedCount + 1}/${pool.length} cast)`
        }
        break
      }
      case 'finalize_fraud_review': {
        if (!report || report.status !== 1) fail('InvalidFraudStatus', `finalize requires UNDER_REVIEW(1), status = ${report ? R_STATUS[report.status] : 'no report'}`)
        else pass('report.status == UNDER_REVIEW', 'UNDER_REVIEW')
        if (!allVoted) fail('ReviewNotFinalizable', `${votedCount}/${pool.length} votes cast`)
        else pass('all committee votes in', `${pool.length}/${pool.length}`)
        if (report && report.status === 2) fail('ReviewAlreadyDecided', 'review already decided (UPHELD)')
        else if (report && report.status === 3) fail('ReviewAlreadyDecided', 'review already decided (DISMISSED)')
        else pass('decision pending', 'no prior decision')
        if (ok && report) {
          if (decisionUpheld) {
            setReport({ ...report, status: 2 })
            setRestriction({ status: 0, appealOpensAt: now + APPEAL_DELAY, rehabAt: now + REHAB })
            summary = `FraudReviewFinalized — UPHELD, restriction ACTIVE on ${accused.slice(0, 12)}…; appeal opens +24h, rehab eligible +30d`
          } else {
            setReport({ ...report, status: 3 })
            setRestriction(null)
            summary = 'FraudReviewFinalized — DISMISSED, no restriction applied'
          }
        }
        break
      }
      case 'file_fraud_appeal': {
        if (!restriction || restriction.status !== 0) fail('NotRestrictedWallet', 'no ACTIVE restriction to appeal')
        else pass('restriction.status == ACTIVE', 'ACTIVE')
        if (wallet !== report?.accused) fail('NotRestrictedWallet', `signer ${wallet.slice(0, 12)}… ≠ restricted wallet ${report?.accused.slice(0, 12)}…`)
        else pass('signer == restriction.wallet', 'restricted wallet signs')
        if (restriction && now < restriction.appealOpensAt) fail('AppealWindowClosed', `appeal opens ${new Date(restriction.appealOpensAt * 1000).toISOString().slice(0, 16).replace('T', ' ')}Z (+24h)`)
        else if (restriction) pass('now ≥ appeal_opens_at', '24h appeal delay elapsed')
        if (ok && report && restriction) {
          setReport({ ...report, status: 4 })
          summary = `FraudAppealFiled — report → APPEALED, appeal review queued for the committee`
        }
        break
      }
      case 'rehabilitate_restriction': {
        if (!restriction || restriction.status !== 0) fail('RestrictionNotActive', `restriction.status = ${restriction ? (restriction.status === 1 ? 'LIFTED' : 'none') : 'none'}`)
        else pass('restriction.status == ACTIVE', 'ACTIVE')
        if (restriction && now < restriction.rehabAt) fail('RehabTooEarly', `rehab_eligible_at = ${new Date(restriction.rehabAt * 1000).toISOString().slice(0, 16).replace('T', ' ')}Z (+30d clean standing)`)
        else if (restriction) pass('now ≥ rehab_eligible_at', '30 days clean standing')
        if (wallet !== report?.accused) fail('NotRestrictedWallet', 'only the restricted wallet lifts its own restriction')
        else pass('signer == restriction.wallet', 'self-regulating lift')
        if (report && report.status === 4) fail('InvalidFraudStatus', 'report is APPEALED — open appeal blocks rehab')
        else if (report) pass('no open appeal', `report.status = ${R_STATUS[report.status]}`)
        if (ok && restriction && report) {
          setRestriction({ ...restriction, status: 1 })
          setReport({ ...report, status: 2 })
          summary = 'RestrictionLifted — rehabilitated after 30d clean standing (self-regulating, no admin)'
        }
        break
      }
    }
    setLast({ ix, ok, summary, verdict: { ok, headline: summary, gates } })
  }

  const stepState = (i: number): string => {
    if (report === null) return 'pending'
    if (i < report.status) return 'done'
    if (i === report.status) return report.status >= 2 ? 'done' : 'active'
    return 'pending'
  }

  return (
    <div className="lab-body">
      <DemoBanner source="fraud_governance.rs" />

      <div className="lab-card mt-3 space-y-2">
        <div className="flex items-center justify-between flex-wrap gap-2">
          <h3 className="text-[13px] font-semibold flex items-center gap-1.5">
            <ShieldAlert size={14} /> Fraud report → review → restriction → appeal → rehab
          </h3>
          <div className="flex gap-1 items-center flex-wrap">
            <span className={`lab-badge ${report === null ? 'lab-badge-mut' : report.status >= 2 ? (report.status === 3 ? 'lab-badge-mut' : 'lab-badge-ok') : 'lab-badge-warn'}`}>
              report {report === null ? '—' : R_STATUS[report.status]}
            </span>
            <span className={`lab-badge ${restriction === null ? 'lab-badge-mut' : restriction.status === 0 ? 'lab-badge-err' : 'lab-badge-ok'}`}>
              restriction {restriction === null ? '—' : restriction.status === 0 ? 'ACTIVE' : 'LIFTED'}
            </span>
          </div>
        </div>
        <div className="pg-steps">
          {['OPEN', 'UNDER_REVIEW', 'DECIDED', 'APPEAL/REHAB'].map((s, i) => (
            <span key={s} className={`pg-step pg-step-${stepState(i)}`}>
              {s}
            </span>
          ))}
        </div>
        <div className="pg-fields">
          <Field label="reporter">
            <input className="select-input" value={reporter} onChange={(e) => setReporter(e.target.value)} />
          </Field>
          <Field label="accused">
            <input className="select-input" value={accused} onChange={(e) => setAccused(e.target.value)} />
          </Field>
          <Field label="reason_code">
            <select className="select-input" value={reason} onChange={(e) => setReason(Number(e.target.value))}>
              {REASONS.map((r, i) => (
                <option key={r} value={i}>{`${i} — ${r}`}</option>
              ))}
              <option value={99}>99 — invalid (&gt; MAX)</option>
            </select>
          </Field>
          <Field label="voter (committee)">
            <select className="select-input" value={voter} onChange={(e) => setVoter(e.target.value)}>
              {COMMITTEE.map((c) => (
                <option key={c} value={c}>{c}</option>
              ))}
            </select>
          </Field>
          <Field label="vote choice">
            <select className="select-input" value={voteChoice ? 'up' : 'down'} onChange={(e) => setVoteChoice(e.target.value === 'up')}>
              <option value="up">uphold</option>
              <option value="down">dismiss</option>
            </select>
          </Field>
          <Field label="review pool membership">
            <select
              className="select-input"
              value={oneUnregistered ? 'bad' : 'ok'}
              onChange={(e) => setOneUnregistered(e.target.value === 'bad')}
            >
              <option value="ok">all 5 registered</option>
              <option value="bad">C-5 unregistered</option>
            </select>
          </Field>
          <Field label="appeal signer wallet">
            <select className="select-input" value={wallet} onChange={(e) => setWallet(e.target.value)}>
              <option value={accused}>restricted wallet (accused)</option>
              <option value="DemoOther111111111111111111111111111111">unrelated wallet</option>
            </select>
          </Field>
        </div>
        <div className="pg-actions">
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('submit_fraud_report')}>
            submit_fraud_report
          </button>
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('open_fraud_review')}>
            open_fraud_review
          </button>
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('cast_fraud_vote')}>
            cast_fraud_vote
          </button>
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('finalize_fraud_review')}>
            finalize_fraud_review
          </button>
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('file_fraud_appeal')}>
            file_fraud_appeal
          </button>
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('rehabilitate_restriction')}>
            rehabilitate_restriction
          </button>
          <button
            className="btn btn-ghost px-2 py-1 text-[11px]"
            onClick={() => {
              setReport(null)
              setRestriction(null)
              setVotes({})
              setLast(null)
              setNow(SIM_START)
            }}
          >
            reset
          </button>
        </div>
        <p className="text-[10px] text-muted">
          Votes: <span className="font-mono">{votedCount}/{pool.length}</span> cast, decision = simple majority
          ({Math.floor(pool.length / 2) + 1} to uphold) · appeal delay 24h, rehab window 30d from restriction.
        </p>
      </div>

      <div className="mt-3">{last && <VerdictCard title={`${last.ix} — guard trace`} verdict={last.verdict} />}</div>

      <Broadcast
        ix={last?.ix ?? 'submit_fraud_report'}
        ok={last?.ok ?? false}
        summary={last?.summary ?? 'submit_fraud_report not run yet — no report PDA'}
        note="Reports never auto-punish: restriction only appears after a full committee review."
      />

      <ClockBar
        now={now}
        onAdvance={(s) => setNow(now + s)}
        onReset={() => setNow(SIM_START)}
        extra={
          <>
            <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => setNow(now + DAY)}>
              +1d (appeal)
            </button>
            <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => setNow(now + 30 * DAY)}>
              +30d (rehab)
            </button>
          </>
        }
      />
    </div>
  )
}
