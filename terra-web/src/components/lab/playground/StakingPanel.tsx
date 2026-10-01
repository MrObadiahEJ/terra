import { useState } from 'react'
import { Coins } from 'lucide-react'
import { Broadcast, ClockBar, DemoBanner, Field, VerdictCard } from './shared'
import { SIM_START, DAY, err, pushGate, type Gate, type Verdict } from './helpers'

const MIN_STAKE = 1 // SOL — staking.rs MIN_STAKE_LAMPORTS = 1e9
const UNBONDING = 7 * DAY // UNBONDING_PERIOD_SECS
const MAX_RATE_BPS = 2000 // create_stake_pool: 0 < rate ≤ 2000

type SStatus = 'NONE' | 'ACTIVE' | 'UNBONDING' | 'WITHDRAWN'
type Role = 'manager' | 'other'
type SAction = 'create_stake_pool' | 'deposit_stake' | 'initiate_unbonding' | 'withdraw_stake' | 'report_equivocation' | 'claim_rewards' | 'slash_validator'

interface Call {
  ix: SAction
  ok: boolean
  summary: string
  verdict: Verdict
}

export default function StakingPanel() {
  const [pool, setPool] = useState(false)
  const [rateBps, setRateBps] = useState(500)
  const [status, setStatus] = useState<SStatus>('NONE')
  const [stake, setStake] = useState(0) // SOL
  const [unbondAt, setUnbondAt] = useState(0)
  const [depositedAt, setDepositedAt] = useState(0)
  const [slashed, setSlashed] = useState(false)
  const [rewardsClaimed, setRewardsClaimed] = useState(false)
  const [depositSol, setDepositSol] = useState(5)
  const [role, setRole] = useState<Role>('manager')
  const [offense, setOffense] = useState(1)
  const [now, setNow] = useState(SIM_START)
  const [last, setLast] = useState<Call | null>(null)

  const withdrawReadyAt = unbondAt + UNBONDING
  const rewards =
    status === 'NONE' || rewardsClaimed ? 0 : (stake * rateBps * Math.min(Math.max((now - depositedAt) / DAY, 0), 30)) / 10_000

  const run = (ix: SAction) => {
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
      case 'create_stake_pool': {
        if (pool) fail('InvalidStatus', 'stake pool already exists for this region')
        else pass('pool PDA free', 'no stake pool yet')
        if (rateBps <= 0 || rateBps > MAX_RATE_BPS) fail('InvalidStatus', `reward_rate_bps = ${rateBps}; must be 1..${MAX_RATE_BPS}`)
        else pass('0 < reward_rate_bps ≤ 2000', `${rateBps} bps`)
        if (role !== 'manager') fail('NotAuthorized', `signer role = ${role}; pool manager creates the pool`)
        else pass('signer == manager', 'manager signs')
        if (ok) {
          setPool(true)
          summary = `StakePoolCreated — reward_rate = ${rateBps} bps, unbonding 7d, min deposit 1 SOL`
        }
        break
      }
      case 'deposit_stake': {
        if (!pool) fail('InvalidStatus', 'create_stake_pool first — no pool to deposit into')
        else pass('pool exists', `rate ${rateBps} bps`)
        if (status === 'ACTIVE') fail('StakeAlreadyActive', 'stake is already ACTIVE — deactivate or withdraw first')
        else pass('no active stake', `status = ${status}`)
        if (status === 'UNBONDING') fail('UnbondingInProgress', 'unbonding in progress — deposit rejected until withdraw')
        else pass('not unbonding', `status = ${status}`)
        if (depositSol < MIN_STAKE) fail('InsufficientStake', `${depositSol} SOL < MIN_STAKE_LAMPORTS = 1 SOL`)
        else pass('amount ≥ 1 SOL', `${depositSol} SOL`)
        if (ok) {
          setStatus('ACTIVE')
          setStake(depositSol)
          setDepositedAt(now)
          setRewardsClaimed(false)
          setSlashed(false)
          summary = `StakeDeposited — ${depositSol} SOL bonded, stake → ACTIVE`
        }
        break
      }
      case 'initiate_unbonding': {
        if (status === 'UNBONDING') fail('UnbondingInProgress', 'unbonding already started')
        else if (status !== 'ACTIVE') fail('InvalidStatus', `initiate_unbonding requires ACTIVE stake, status = ${status}`)
        else {
          pass('stake.status == ACTIVE', 'ACTIVE')
          setStatus('UNBONDING')
          setUnbondAt(now)
          summary = `UnbondingInitiated — withdraw unlocks ${new Date((now + UNBONDING) * 1000).toISOString().slice(0, 16).replace('T', ' ')}Z (7d period)`
        }
        break
      }
      case 'withdraw_stake': {
        if (status === 'NONE' || status === 'WITHDRAWN') fail('InvalidStatus', `nothing to withdraw — status = ${status}`)
        else if (status === 'ACTIVE') fail('UnbondingNotComplete', 'no unbonding started — initiate_unbonding first')
        else pass('unbonding started', `at ${new Date(unbondAt * 1000).toISOString().slice(0, 16).replace('T', ' ')}Z`)
        if (status === 'UNBONDING' && now < withdrawReadyAt) fail('UnbondingNotComplete', `unbonding_starts_at + 7d = ${new Date(withdrawReadyAt * 1000).toISOString().slice(0, 16).replace('T', ' ')}Z is in the future`)
        else if (status === 'UNBONDING') pass('now ≥ unbonding_starts_at + 7d', 'unbonding period elapsed')
        if (ok) {
          setStatus('WITHDRAWN')
          summary = `StakeWithdrawn — ${stake} SOL returned after 7d unbonding`
        }
        break
      }
      case 'report_equivocation': {
        if (offense > 3) fail('InvalidOffenseType', `offense_type = ${offense} > MAX`)
        else pass('offense_type valid', `${offense} ∈ 0..3`)
        if (offense === 0) fail('SelfReportNotAllowed', 'offense_type = 0 maps to self-report — validators cannot report themselves')
        else pass('reporter ≠ offender', 'third-party reporter')
        if (slashed) fail('InvalidStatus', 'validator already slashed for this offense')
        else pass('not yet slashed', 'clean record')
        if (ok) {
          summary = `EquivocationReported — offense_type ${offense}, slashing case queued for admin review`
        }
        break
      }
      case 'claim_rewards': {
        if (status !== 'ACTIVE' && status !== 'UNBONDING') fail('InvalidStatus', `claim requires bonded stake, status = ${status}`)
        else pass('stake bonded', status)
        if (rewards <= 0) fail('InvalidStatus', 'no accrued rewards at the current simulated clock')
        else pass('rewards accrued', `${rewards.toFixed(4)} SOL`)
        if (ok) {
          setRewardsClaimed(true)
          summary = `RewardsClaimed — ${rewards.toFixed(4)} SOL at ${rateBps} bps`
        }
        break
      }
      case 'slash_validator': {
        if (role !== 'manager') fail('NotAuthorized', `signer role = ${role}; admin/manager slashes`)
        else pass('signer == manager', 'manager signs')
        if (status !== 'ACTIVE' && status !== 'UNBONDING') fail('InvalidStatus', `no bonded stake to slash — status = ${status}`)
        else pass('stake bonded', status)
        if (slashed) fail('InvalidStatus', 'validator already slashed')
        else pass('not yet slashed', 'clean record')
        if (ok) {
          setSlashed(true)
          summary = 'ValidatorSlashed — reputation penalty applied, stake confiscated to treasury'
        }
        break
      }
    }
    setLast({ ix, ok, summary, verdict: { ok, headline: summary, gates } })
  }

  const stepState = (s: SStatus, i: number): string => {
    const order: SStatus[] = ['NONE', 'ACTIVE', 'UNBONDING', 'WITHDRAWN']
    const cur = order.indexOf(status)
    if (status === 'WITHDRAWN') return i <= 3 ? (i === 3 ? 'done' : 'done') : 'pending'
    if (i < cur) return 'done'
    if (i === cur) return 'active'
    if (s === 'WITHDRAWN') return 'pending'
    return 'pending'
  }

  return (
    <div className="lab-body">
      <DemoBanner source="staking.rs" />

      <div className="lab-card mt-3 space-y-2">
        <div className="flex items-center justify-between flex-wrap gap-2">
          <h3 className="text-[13px] font-semibold flex items-center gap-1.5">
            <Coins size={14} /> Stake lifecycle: pool → deposit → unbond → withdraw
          </h3>
          <div className="flex gap-1 items-center flex-wrap">
            <span className={`lab-badge ${status === 'ACTIVE' ? 'lab-badge-ok' : status === 'UNBONDING' ? 'lab-badge-warn' : status === 'WITHDRAWN' ? 'lab-badge-mut' : 'lab-badge-mut'}`}>
              stake {status}
            </span>
            <span className={`lab-badge ${slashed ? 'lab-badge-err' : 'lab-badge-ok'}`}>{slashed ? 'SLASHED' : 'clean'}</span>
          </div>
        </div>
        <div className="pg-steps">
          {(['NONE', 'ACTIVE', 'UNBONDING', 'WITHDRAWN'] as const).map((s, i) => (
            <span key={s} className={`pg-step pg-step-${stepState(s, i)}`}>
              {s}
            </span>
          ))}
        </div>
        <div className="pg-fields">
          <Field label={`reward_rate_bps (1..${MAX_RATE_BPS})`}>
            <input
              className="select-input"
              type="number"
              value={rateBps}
              onChange={(e) => setRateBps(Number(e.target.value) || 0)}
            />
          </Field>
          <Field label="deposit amount (SOL, min 1)">
            <input
              className="select-input"
              type="number"
              step="0.5"
              value={depositSol}
              onChange={(e) => setDepositSol(Number(e.target.value) || 0)}
            />
          </Field>
          <Field label="signer role">
            <select className="select-input" value={role} onChange={(e) => setRole(e.target.value as Role)}>
              <option value="manager">pool manager / admin</option>
              <option value="other">unrelated signer</option>
            </select>
          </Field>
          <Field label="equivocation offense_type">
            <select className="select-input" value={offense} onChange={(e) => setOffense(Number(e.target.value))}>
              <option value={1}>1 — double-sign</option>
              <option value={2}>2 — contradictory attestation</option>
              <option value={3}>3 — stale payload replay</option>
              <option value={0}>0 — self-report (invalid)</option>
              <option value={9}>9 — invalid (&gt; MAX)</option>
            </select>
          </Field>
        </div>
        <div className="pg-actions">
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('create_stake_pool')}>
            create_stake_pool
          </button>
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('deposit_stake')}>
            deposit_stake
          </button>
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('initiate_unbonding')}>
            initiate_unbonding
          </button>
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('withdraw_stake')}>
            withdraw_stake
          </button>
          <button
            className="btn btn-secondary px-2 py-1 text-[11px]"
            disabled={rewards <= 0}
            title={rewards <= 0 ? 'no accrued rewards at this clock' : undefined}
            onClick={() => run('claim_rewards')}
          >
            claim_rewards ({rewards.toFixed(3)})
          </button>
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('report_equivocation')}>
            report_equivocation
          </button>
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('slash_validator')}>
            slash_validator
          </button>
          <button
            className="btn btn-ghost px-2 py-1 text-[11px]"
            onClick={() => {
              setPool(false)
              setStatus('NONE')
              setStake(0)
              setUnbondAt(0)
              setSlashed(false)
              setRewardsClaimed(false)
              setLast(null)
              setNow(SIM_START)
            }}
          >
            reset
          </button>
        </div>
        <p className="text-[10px] text-muted">
          bonded: <span className="font-mono">{stake} SOL</span> · unbonding unlocks{' '}
          <span className="font-mono">
            {unbondAt ? `${new Date(withdrawReadyAt * 1000).toISOString().slice(0, 16).replace('T', ' ')}Z` : '—'}
          </span>{' '}
          · accrual simulated at rate × days (capped 30d) — claim_rewards enables only with rewards &gt; 0.
        </p>
      </div>

      <div className="mt-3">{last && <VerdictCard title={`${last.ix} — guard trace`} verdict={last.verdict} />}</div>

      <Broadcast
        ix={last?.ix ?? 'deposit_stake'}
        ok={last?.ok ?? false}
        summary={last?.summary ?? 'deposit_stake not run yet — create a pool first'}
        note="7-day unbonding is enforced on-chain — advance the clock to pass it."
      />

      <ClockBar
        now={now}
        onAdvance={(s) => setNow(now + s)}
        onReset={() => setNow(SIM_START)}
        extra={
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => setNow(now + 7 * DAY)}>
            +7d (unbond)
          </button>
        }
      />
    </div>
  )
}
