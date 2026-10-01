import { useState } from 'react'
import { Gavel } from 'lucide-react'
import { Broadcast, ClockBar, DemoBanner, Field, VerdictCard } from './shared'
import { SIM_START, DAY, err, pushGate, type Gate, type Verdict } from './helpers'

const DISPUTE_EXPIRY = 90 * DAY // dispute.rs DISPUTE_EXPIRY_SECS
const MIN_DISPUTE_VALIDATORS = 2

const D_STATUS = ['FILED', 'FROZEN', 'ADJUDICATED', 'EXECUTED', 'CANCELLED'] as const
const ALL_VALIDATORS = ['V-1', 'V-2', 'V-3', 'V-4', 'V-5']

type Role = 'holder' | 'validator' | 'admin'
type DAction = 'file_dispute' | 'freeze_parcel' | 'adjudicate_dispute' | 'execute_judgment' | 'cancel_dispute'

interface Call {
  ix: DAction
  ok: boolean
  summary: string
  verdict: Verdict
}

export default function DisputePanel() {
  const [parcel, setParcel] = useState<'REGISTERED' | 'DISPUTED' | 'FROZEN'>('REGISTERED')
  const [owner, setOwner] = useState('DemoHolder11111111111111111111111111111')
  const [dispute, setDispute] = useState<{
    status: number
    filedAt: number
    required: number
    outcome: number
  } | null>(null)
  const [role, setRole] = useState<Role>('holder')
  const [required, setRequired] = useState(MIN_DISPUTE_VALIDATORS)
  const [oneUnregistered, setOneUnregistered] = useState(false)
  const [signers, setSigners] = useState<string[]>(['V-1', 'V-2'])
  const [duplicateSigner, setDuplicateSigner] = useState(false)
  const [outcome, setOutcome] = useState(0)
  const [now, setNow] = useState(SIM_START)
  const [last, setLast] = useState<Call | null>(null)

  const registered = oneUnregistered ? ALL_VALIDATORS.slice(0, 4) : ALL_VALIDATORS
  const expiryAt = dispute ? dispute.filedAt + DISPUTE_EXPIRY : 0

  const run = (ix: DAction) => {
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
      case 'file_dispute': {
        if (role !== 'holder' && role !== 'validator') fail('NotAuthorized', `signer role = ${role}; holder or validator may file`)
        else pass('is_holder || is_validator', `${role} signs`)
        if (required < MIN_DISPUTE_VALIDATORS) fail('InvalidThreshold', `required = ${required} < MIN_DISPUTE_VALIDATORS = ${MIN_DISPUTE_VALIDATORS} (anti-grief)`)
        else pass('required ≥ 2 co-signatures', `required = ${required}`)
        const unregistered = ALL_VALIDATORS.find((v) => !registered.includes(v))
        if (unregistered) fail('NotValidator', `${unregistered} is not in registry.validators`)
        else pass('all declared validators registered', `${ALL_VALIDATORS.length} keys in registry.validators`)
        if (registered.length === 0) fail('NoValidators', 'registry has no validators to co-sign')
        else pass('validators.length > 0', `${registered.length} validators`)
        if (dispute !== null && dispute.status !== 4) fail('InvalidStatus', `parcel already has dispute status = ${D_STATUS[dispute.status]}`)
        else pass('no active dispute', 'dispute PDA free')
        if (ok) {
          setDispute({ status: 0, filedAt: now, required, outcome: 0 })
          setParcel('DISPUTED')
          summary = `DisputeFiled — case_hash over parcel + ${required} validator co-signatures; parcel → DISPUTED, expires ${new Date((now + DISPUTE_EXPIRY) * 1000).toISOString().slice(0, 10)}`
        }
        break
      }
      case 'freeze_parcel': {
        if (!dispute || dispute.status !== 0) fail('InvalidDisputeStatus', `freeze requires FILED(0), status = ${dispute ? D_STATUS[dispute.status] : 'no dispute'}`)
        else pass('dispute.status == FILED', 'FILED')
        if (dispute && now >= expiryAt) fail('DisputeExpired', `filed_at + 90d = ${new Date(expiryAt * 1000).toISOString().slice(0, 10)} passed`)
        else if (dispute) pass('now < filed_at + 90d', `expires ${new Date(expiryAt * 1000).toISOString().slice(0, 10)}`)
        if (parcel !== 'DISPUTED') fail('InvalidStatus', `parcel.status = ${parcel}, freeze expects DISPUTED`)
        else pass('parcel.status == DISPUTED', 'DISPUTED')
        const unique = new Set(signers)
        if (duplicateSigner) fail('DuplicateValidator', 'remaining_accounts contains the same validator key twice')
        else pass('remaining_accounts deduplicated', `${unique.size} unique signer keys`)
        if (unique.size < (dispute?.required ?? required)) fail('InsufficientValidatorSigners', `${unique.size} present < required ${dispute?.required ?? required}`)
        else pass('present ≥ required', `${unique.size} ≥ ${dispute?.required ?? required}`)
        if (ok && dispute) {
          setDispute({ ...dispute, status: 1 })
          setParcel('FROZEN')
          summary = `ParcelFrozen — ${unique.size} validators present; dispute + parcel → FROZEN`
        }
        break
      }
      case 'adjudicate_dispute': {
        if (role !== 'admin' && role !== 'validator') fail('NotAuthorized', `signer role = ${role}; admin/validator adjudicates`)
        else pass('is_admin || is_validator', `${role} signs`)
        if (!dispute || dispute.status !== 1) fail('InvalidDisputeStatus', `adjudicate requires FROZEN(1), status = ${dispute ? D_STATUS[dispute.status] : 'no dispute'}`)
        else pass('dispute.status == FROZEN', 'FROZEN')
        if (dispute && now >= expiryAt) fail('DisputeExpired', '90-day adjudication window passed')
        else if (dispute) pass('now < filed_at + 90d', 'within window')
        if (outcome !== 0 && outcome !== 1) fail('InvalidDisputeOutcome', `outcome = ${outcome}; only OWNER_WINS(0)/OWNER_LOSES(1)`)
        else pass('outcome ∈ {OWNER_WINS, OWNER_LOSES}', outcome === 0 ? 'OWNER_WINS(0)' : 'OWNER_LOSES(1)')
        if (outcome === 1 && owner === '') fail('EmptyNewOwner', 'successor owner key empty')
        else pass('new_owner set', outcome === 0 ? 'owner retained' : 'successor ready')
        if (ok && dispute) {
          setDispute({ ...dispute, status: 2, outcome })
          summary = `DisputeAdjudicated — outcome = ${outcome === 0 ? 'OWNER_WINS(0)' : 'OWNER_LOSES(1)'}`
        }
        break
      }
      case 'execute_judgment': {
        if (!dispute || dispute.status !== 2) fail('InvalidDisputeStatus', `execute requires ADJUDICATED(2), status = ${dispute ? D_STATUS[dispute.status] : 'no dispute'}`)
        else pass('dispute.status == ADJUDICATED', 'ADJUDICATED')
        if (dispute && now >= expiryAt) fail('DisputeExpired', 'execution after 90-day expiry rejected')
        else if (dispute) pass('now < filed_at + 90d', 'within window')
        if (dispute && dispute.outcome === 1) pass('ownership transfer', `owner ${owner} → DemoSuccessor222222222222222222222222222`)
        else pass('ownership retained', 'OWNER_WINS — owner keeps the parcel')
        if (ok && dispute) {
          setDispute({ ...dispute, status: 3 })
          setParcel('REGISTERED')
          if (dispute.outcome === 1) setOwner('DemoSuccessor222222222222222222222222222')
          summary = dispute.outcome === 1
            ? 'JudgmentExecuted — parcel frozen lifted, ownership transferred to successor'
            : 'JudgmentExecuted — parcel unfrozen, owner confirmed'
        }
        break
      }
      case 'cancel_dispute': {
        if (!dispute || dispute.status !== 0) fail('InvalidDisputeStatus', `cancel only while FILED(0) — pre-freeze per M-3; status = ${dispute ? D_STATUS[dispute.status] : 'no dispute'}`)
        else pass('dispute.status == FILED', 'FILED — still cancellable')
        if (role !== 'holder' && role !== 'admin') fail('NotAuthorized', `signer role = ${role}; holder or admin cancels`)
        else pass('holder/admin signs', `${role} signs`)
        if (ok && dispute) {
          setDispute({ ...dispute, status: 4 })
          setParcel('REGISTERED')
          summary = 'DisputeCancelled — parcel returned to REGISTERED before freeze'
        }
        break
      }
    }
    setLast({ ix, ok, summary, verdict: { ok, headline: summary, gates } })
  }

  const steps = [
    { label: 'FILED', state: dispute === null ? 'pending' : dispute.status === 4 ? 'failed' : 'done' as const },
    { label: 'FROZEN', state: dispute && dispute.status >= 1 && dispute.status < 4 ? 'done' : dispute === null || dispute.status === 4 ? 'pending' : 'active' as const },
    { label: 'ADJUDICATED', state: dispute && dispute.status >= 2 && dispute.status < 4 ? 'done' : 'pending' as const },
    { label: 'EXECUTED', state: dispute?.status === 3 ? 'done' : 'pending' as const },
  ]

  return (
    <div className="lab-body">
      <DemoBanner source="dispute.rs" />

      <div className="lab-card mt-3 space-y-2">
        <div className="flex items-center justify-between flex-wrap gap-2">
          <h3 className="text-[13px] font-semibold flex items-center gap-1.5">
            <Gavel size={14} /> Dispute state machine
          </h3>
          <div className="flex gap-1 items-center flex-wrap">
            <span className={`lab-badge ${parcel === 'REGISTERED' ? 'lab-badge-ok' : parcel === 'DISPUTED' ? 'lab-badge-warn' : 'lab-badge-err'}`}>
              parcel {parcel}
            </span>
            <span className={`lab-badge ${dispute === null ? 'lab-badge-mut' : dispute.status === 4 ? 'lab-badge-err' : dispute.status === 3 ? 'lab-badge-ok' : 'lab-badge-info'}`}>
              {dispute === null ? 'no dispute' : D_STATUS[dispute.status]}
            </span>
          </div>
        </div>
        <div className="pg-steps">
          {steps.map((s) => (
            <span key={s.label} className={`pg-step pg-step-${s.state}`}>
              {s.label}
            </span>
          ))}
        </div>
        <div className="pg-fields">
          <Field label="signer role">
            <select className="select-input" value={role} onChange={(e) => setRole(e.target.value as Role)}>
              <option value="holder">parcel holder</option>
              <option value="validator">validator</option>
              <option value="admin">admin</option>
            </select>
          </Field>
          <Field label={`required co-signatures (min ${MIN_DISPUTE_VALIDATORS})`}>
            <select className="select-input" value={required} onChange={(e) => setRequired(Number(e.target.value))}>
              {[1, 2, 3, 4].map((n) => (
                <option key={n} value={n}>{n}</option>
              ))}
            </select>
          </Field>
          <Field label="registry membership">
            <select
              className="select-input"
              value={oneUnregistered ? 'bad' : 'ok'}
              onChange={(e) => setOneUnregistered(e.target.value === 'bad')}
            >
              <option value="ok">all 5 validators registered</option>
              <option value="bad">V-5 not in registry.validators</option>
            </select>
          </Field>
          <Field label="freeze signers (remaining_accounts)">
            <select
              className="select-input"
              value={signers.join(',')}
              onChange={(e) => setSigners(e.target.value ? e.target.value.split(',') : [])}
            >
              <option value="V-1,V-2">V-1, V-2</option>
              <option value="V-1,V-2,V-3">V-1, V-2, V-3</option>
              <option value="V-1">V-1 only</option>
              <option value="">none</option>
            </select>
          </Field>
          <Field label="duplicate signer key">
            <select
              className="select-input"
              value={duplicateSigner ? 'yes' : 'no'}
              onChange={(e) => setDuplicateSigner(e.target.value === 'yes')}
            >
              <option value="no">deduplicated</option>
              <option value="yes">same key twice</option>
            </select>
          </Field>
          <Field label="adjudication outcome">
            <select className="select-input" value={outcome} onChange={(e) => setOutcome(Number(e.target.value))}>
              <option value={0}>OWNER_WINS (0)</option>
              <option value={1}>OWNER_LOSES (1)</option>
              <option value={2}>invalid (2)</option>
            </select>
          </Field>
        </div>
        <div className="pg-actions">
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('file_dispute')}>
            file_dispute
          </button>
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('freeze_parcel')}>
            freeze_parcel
          </button>
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('adjudicate_dispute')}>
            adjudicate_dispute
          </button>
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('execute_judgment')}>
            execute_judgment
          </button>
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('cancel_dispute')}>
            cancel_dispute
          </button>
          <button
            className="btn btn-ghost px-2 py-1 text-[11px]"
            onClick={() => {
              setDispute(null)
              setParcel('REGISTERED')
              setLast(null)
              setOwner('DemoHolder11111111111111111111111111111')
              setNow(SIM_START)
            }}
          >
            reset
          </button>
        </div>
        <p className="text-[10px] text-muted">
          Expiry: filed_at + 90d ={' '}
          <span className="font-mono">
            {expiryAt ? `${new Date(expiryAt * 1000).toISOString().slice(0, 10)}` : '—'}
          </span>{' '}
          · M-3: cancel only while FILED (before freeze) · owner = canonical Rights PDA, never Parcel.owner.
        </p>
      </div>

      <div className="mt-3">{last && <VerdictCard title={`${last.ix} — guard trace`} verdict={last.verdict} />}</div>

      <Broadcast
        ix={last?.ix ?? 'file_dispute'}
        ok={last?.ok ?? false}
        summary={last?.summary ?? 'file_dispute not run yet — no dispute PDA'}
        note="Re-sends the last call; freeze needs present ≥ required unique signer keys."
      />

      <ClockBar
        now={now}
        onAdvance={(s) => setNow(now + s)}
        onReset={() => setNow(SIM_START)}
        extra={
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => setNow(now + 90 * DAY)}>
            +90d
          </button>
        }
      />
    </div>
  )
}
