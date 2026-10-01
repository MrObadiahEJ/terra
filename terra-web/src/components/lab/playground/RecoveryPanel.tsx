import { useState } from 'react'
import { HeartPulse, Timer } from 'lucide-react'
import { Broadcast, ClockBar, DemoBanner, Field, VerdictCard } from './shared'
import { SIM_START, HOUR, err, pushGate, tx, type Gate, type Verdict } from './helpers'

const TIMELOCK = 48 * HOUR // recovery.rs EMERGENCY_INJECTION_TIMELOCK_SECS
const FLIP_THRESHOLD = 4 // validator_registry.rs CONSENSUS_FLIP_THRESHOLD

const INITIAL = [
  { id: 'V-alpha', active: true, lastBeat: SIM_START },
  { id: 'V-bravo', active: true, lastBeat: SIM_START },
  { id: 'V-charlie', active: true, lastBeat: SIM_START },
  { id: 'V-delta', active: true, lastBeat: SIM_START },
  { id: 'V-echo', active: true, lastBeat: SIM_START },
]

type RAction = 'heartbeat' | 'set_validator_active' | 'check_quorum_reachable' | 'queue_emergency_injection' | 'execute_emergency_injection'

interface Call {
  ix: string
  ok: boolean
  summary: string
  verdict: Verdict
}

export default function RecoveryPanel() {
  const [validators, setValidators] = useState(INITIAL)
  const [selected, setSelected] = useState('V-alpha')
  const [role, setRole] = useState<'admin' | 'other'>('admin')
  const [candidate, setCandidate] = useState('V-foxtrot-NEW')
  const [injection, setInjection] = useState<{ candidate: string; executeAfter: number; executed: boolean } | null>(null)
  const [quorum, setQuorum] = useState<boolean | null>(null)
  const [now, setNow] = useState(SIM_START)
  const [last, setLast] = useState<Call | null>(null)

  const activeIds = validators.filter((v) => v.active).map((v) => v.id)
  const activeCount = activeIds.length

  const toggle = (id: string) => {
    const v = validators.find((x) => x.id === id)
    if (!v) return
    setValidators(validators.map((x) => (x.id === id ? { ...x, active: !x.active } : x)))
    tx('set_validator_active', true, `SetValidatorActive — ${id} ${v.active ? 'removed from' : 'added to'} registry.validators (n → ${v.active ? activeCount - 1 : activeCount + 1})`)
    setQuorum(null)
  }

  const heartbeat = (id: string) => {
    setValidators(validators.map((x) => (x.id === id ? { ...x, lastBeat: now } : x)))
    tx('heartbeat', true, `Heartbeat — ${id} presence refreshed at ${new Date(now * 1000).toISOString().slice(0, 16).replace('T', ' ')}Z`)
  }

  const run = (ix: RAction) => {
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
      case 'heartbeat': {
        const v = validators.find((x) => x.id === selected)
        if (!v) fail('InvalidStatus', 'validator not found')
        else if (!v.active) fail('NotValidator', `${selected} is not in registry.validators`)
        else {
          pass('validator ∈ registry.validators', selected)
          heartbeat(selected)
          summary = `Heartbeat — ${selected} presence refreshed`
        }
        break
      }
      case 'set_validator_active': {
        const v = validators.find((x) => x.id === selected)
        if (!v) {
          fail('InvalidStatus', 'validator not found')
        } else if (role !== 'admin') {
          pass('set_validator_active guarded by registry.admin', 'requirement checked')
          fail('NotAuthorized', `signer role = ${role}`)
        } else {
          pass('signer == registry.admin', 'admin signs')
          toggle(selected)
          summary = `SetValidatorActive — ${selected} ${v.active ? 'deactivated' : 'reactivated'}; active n = ${v.active ? activeCount - 1 : activeCount + 1}`
        }
        break
      }
      case 'check_quorum_reachable': {
        const reachable = activeCount >= FLIP_THRESHOLD && activeCount > 0
        pushGate(gates, 'active_count = registry.validators.len()', true, `${activeCount} active`)
        pushGate(
          gates,
          `active_count ≥ CONSENSUS_FLIP_THRESHOLD (${FLIP_THRESHOLD})`,
          reachable,
          reachable ? `${activeCount} ≥ ${FLIP_THRESHOLD}` : `quorum unreachable — ${activeCount} < ${FLIP_THRESHOLD}; off-chain service sees QuorumReachabilityChecked and reacts`,
        )
        setQuorum(reachable)
        summary = reachable
          ? `QuorumReachabilityChecked — reachable=true (active=${activeCount}, threshold=${FLIP_THRESHOLD})`
          : `QuorumReachabilityChecked — reachable=false (active=${activeCount} < ${FLIP_THRESHOLD}) — informational event, guards remain open`
        break
      }
      case 'queue_emergency_injection': {
        if (role !== 'admin') fail('NotAuthorized', `signer role = ${role}; queue_emergency_injection is admin-only`)
        else pass('signer == registry.admin', 'admin signs')
        if (activeIds.includes(candidate)) fail('AlreadyEndorsedRotation', `${candidate} is already in registry.validators`)
        else pass('candidate not in registry', `${candidate} is new`)
        if (injection && !injection.executed) pass('queue overwrite allowed', `prior queue at +48h replaced (execute_after = now + 48h)`)
        if (ok) {
          setInjection({ candidate, executeAfter: now + TIMELOCK, executed: false })
          summary = `EmergencyInjectionQueued — candidate ${candidate}, execute_after = ${new Date((now + TIMELOCK) * 1000).toISOString().slice(0, 16).replace('T', ' ')}Z (+48h timelock)`
        }
        break
      }
      case 'execute_emergency_injection': {
        if (!injection) fail('InvalidStatus', 'no queued emergency_injection record — queue_emergency_injection first')
        else if (injection.executed) fail('AlreadyEndorsedRotation', 'injection.executed = true — already applied')
        else pass('injection queued, not executed', `${injection.candidate} at +48h`)
        if (injection && now < injection.executeAfter) fail('EmergencyTimelockNotElapsed', `execute_after = ${new Date(injection.executeAfter * 1000).toISOString().slice(0, 16).replace('T', ' ')}Z, now = ${new Date(now * 1000).toISOString().slice(0, 16).replace('T', ' ')}Z`)
        else if (injection) pass('now ≥ execute_after', '48h timelock elapsed')
        if (injection && activeIds.includes(injection.candidate)) fail('AlreadyEndorsedRotation', `${injection.candidate} already in registry.validators`)
        else if (injection) pass('candidate not already a member', injection.candidate)
        if (ok && injection) {
          setValidators([...validators, { id: injection.candidate, active: true, lastBeat: now }])
          setInjection({ ...injection, executed: true })
          summary = `EmergencyInjectionExecuted — ${injection.candidate} pushed into registry.validators (n → ${activeCount + 1}), registry.version++`
        }
        break
      }
    }
    setLast({ ix, ok, summary, verdict: { ok, headline: summary, gates } })
  }

  return (
    <div className="lab-body">
      <DemoBanner source="recovery.rs + validator_registry.rs" />

      <div className="lab-card mt-3 space-y-2">
        <div className="flex items-center justify-between flex-wrap gap-2">
          <h3 className="text-[13px] font-semibold flex items-center gap-1.5">
            <HeartPulse size={14} /> Registry health → emergency injection
          </h3>
          <div className="flex gap-1 items-center flex-wrap">
            <span className={`lab-badge ${activeCount >= FLIP_THRESHOLD ? 'lab-badge-ok' : 'lab-badge-err'}`}>
              active {activeCount}/{validators.length} (threshold {FLIP_THRESHOLD})
            </span>
            <span className={`lab-badge ${injection === null ? 'lab-badge-mut' : injection.executed ? 'lab-badge-ok' : 'lab-badge-warn'}`}>
              {injection === null ? 'no injection' : injection.executed ? 'injection EXECUTED' : 'injection QUEUED'}
            </span>
            {quorum !== null && (
              <span className={`lab-badge ${quorum ? 'lab-badge-ok' : 'lab-badge-err'}`}>
                quorum {quorum ? 'reachable' : 'UNREACHABLE'}
              </span>
            )}
          </div>
        </div>
        <table className="pg-table">
          <thead>
            <tr>
              <th>validator</th>
              <th>registry member</th>
              <th>last heartbeat</th>
              <th>actions</th>
            </tr>
          </thead>
          <tbody>
            {validators.map((v) => (
              <tr key={v.id}>
                <td className="font-mono text-[11px]">{v.id}</td>
                <td className="text-[11px]">
                  <span className={`lab-badge ${v.active ? 'lab-badge-ok' : 'lab-badge-err'}`}>
                    {v.active ? 'ACTIVE' : 'removed'}
                  </span>
                </td>
                <td className="font-mono text-[11px]">{new Date(v.lastBeat * 1000).toISOString().slice(0, 16).replace('T', ' ')}Z</td>
                <td>
                  <div className="flex gap-1">
                    <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => heartbeat(v.id)}>
                      heartbeat
                    </button>
                    <button className="btn btn-ghost px-2 py-1 text-[11px]" onClick={() => toggle(v.id)}>
                      {v.active ? 'deactivate' : 'activate'}
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        <div className="pg-fields">
          <Field label="selected validator">
            <select className="select-input" value={selected} onChange={(e) => setSelected(e.target.value)}>
              {validators.map((v) => (
                <option key={v.id} value={v.id}>{v.id}</option>
              ))}
            </select>
          </Field>
          <Field label="signer role">
            <select className="select-input" value={role} onChange={(e) => setRole(e.target.value as 'admin' | 'other')}>
              <option value="admin">registry admin</option>
              <option value="other">unrelated signer</option>
            </select>
          </Field>
          <Field label="injection candidate">
            <input className="select-input" value={candidate} onChange={(e) => setCandidate(e.target.value)} />
          </Field>
        </div>
        <div className="pg-actions">
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('heartbeat')}>
            heartbeat (selected)
          </button>
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('set_validator_active')}>
            set_validator_active (selected)
          </button>
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('check_quorum_reachable')}>
            check_quorum_reachable
          </button>
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('queue_emergency_injection')}>
            queue_emergency_injection
          </button>
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('execute_emergency_injection')}>
            execute_emergency_injection
          </button>
          <button
            className="btn btn-ghost px-2 py-1 text-[11px]"
            onClick={() => {
              setValidators(INITIAL)
              setInjection(null)
              setQuorum(null)
              setLast(null)
              setNow(SIM_START)
            }}
          >
            reset
          </button>
        </div>
        <p className="text-[10px] text-muted">
          Recovery drill: deactivate ≥ 2 validators → check_quorum_reachable (reports reachable=false) →
          queue injection (admin, +48h timelock) → advance clock 48h → execute → membership restored.
          <Timer size={10} className="inline mx-1" />
          Emergency injection is deliberately slow: execute_after = queued_at + 48h.
        </p>
      </div>

      <div className="mt-3">{last && <VerdictCard title={`${last.ix} — guard trace`} verdict={last.verdict} />}</div>

      <Broadcast
        ix={last?.ix ?? 'check_quorum_reachable'}
        ok={last?.ok ?? false}
        summary={last?.summary ?? 'check_quorum_reachable not run — registry health unknown'}
        note="Quorum check is informational (always emits); injection calls carry real guards."
      />

      <ClockBar
        now={now}
        onAdvance={(s) => setNow(now + s)}
        onReset={() => setNow(SIM_START)}
        extra={
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => setNow(now + 48 * HOUR)}>
            +48h (timelock)
          </button>
        }
      />
    </div>
  )
}
