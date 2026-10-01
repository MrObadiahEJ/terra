import { useState } from 'react'
import { Lock } from 'lucide-react'
import { Broadcast, ClockBar, DemoBanner, Field, VerdictCard } from './shared'
import { SIM_START, DAY, err, pushGate, type Gate, type Verdict } from './helpers'

const MIN_LAMPORTS = 100_000_000 // escrow.rs MIN_ESCROW_AMOUNT = 0.1 SOL
const MAX_LAMPORTS = 1_000_000_000_000 // 1M SOL
const CANCEL_WINDOW = 7 * DAY
const SETTLE_WINDOW = 3 * DAY

const STATUS_NAMES = ['CREATED', 'DEPOSITED', 'ACCEPTED', 'SETTLED', 'CANCELLED', 'DISPUTED'] as const
const STATUS_STYLE = ['lab-badge-info', 'lab-badge-info', 'lab-badge-warn', 'lab-badge-ok', 'lab-badge-mut', 'lab-badge-err']

type Role = 'buyer' | 'seller' | 'other'
type Action = 'create_escrow' | 'deposit_escrow' | 'accept_escrow' | 'settle_escrow' | 'cancel_escrow' | 'mutual_cancel_escrow'

interface Call {
  ix: Action
  ok: boolean
  summary: string
  verdict: Verdict
}

export default function EscrowPanel() {
  const [status, setStatus] = useState<number | null>(null)
  const [amountSol, setAmountSol] = useState(2)
  const [depositSol, setDepositSol] = useState(2)
  const [deposited, setDeposited] = useState(0) // lamports
  const [createdAt, setCreatedAt] = useState(SIM_START)
  const [settleDeadline, setSettleDeadline] = useState(0)
  const [now, setNow] = useState(SIM_START)
  const [role, setRole] = useState<Role>('buyer')
  const [buyerDesignated, setBuyerDesignated] = useState(true)
  const [last, setLast] = useState<Call | null>(null)

  const lamports = Math.round(amountSol * 1e9)
  const cancelDeadline = createdAt + CANCEL_WINDOW

  const run = (ix: Action) => {
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
      case 'create_escrow': {
        pass('amount ≥ MIN_ESCROW_AMOUNT', `${lamports.toLocaleString()} lamports (min 0.1 SOL)`)
        if (lamports < MIN_LAMPORTS) fail('EscrowAmountTooLow', 'below 0.1 SOL floor')
        else if (lamports > MAX_LAMPORTS) fail('EscrowAmountTooHigh', 'above 1M SOL ceiling')
        if (!buyerDesignated) fail('EmptySuccessor', 'buyer pubkey = default (successor unset)')
        else pass('buyer ≠ Pubkey::default()', 'designated buyer present')
        if (status !== null) fail('ParcelAlreadyInEscrow', `parcel already has escrow status = ${STATUS_NAMES[status]}`)
        else pass('parcel free', 'no escrow record for this parcel')
        if (ok) {
          setStatus(0)
          setCreatedAt(now)
          setDeposited(0)
          summary = `EscrowCreated — ${amountSol} SOL, buyer window opens (${new Date(now * 1000).toISOString().slice(0, 10)} + 7d)`
        }
        break
      }
      case 'deposit_escrow': {
        if (status !== 0) fail('InvalidEscrowStatus', `deposit requires CREATED(0), status = ${status === null ? 'none' : STATUS_NAMES[status]}`)
        else pass('status == CREATED', 'CREATED')
        if (role !== 'buyer') fail('NotDesignatedBuyer', `signer role = ${role}, only the designated buyer deposits`)
        else pass('signer == buyer', 'buyer signs')
        const amt = Math.round(depositSol * 1e9)
        if (amt <= 0) fail('InvalidEscrowAmount', 'deposit_amount must be > 0')
        else pass('deposit_amount > 0', `${amt.toLocaleString()} lamports`)
        if (deposited > 0) fail('DepositExists', `vault already holds ${deposited.toLocaleString()} lamports`)
        else pass('vault empty', 'no deposit recorded')
        if (deposited + amt > lamports) fail('DepositExceedsAmount', `${(deposited + amt).toLocaleString()} > ${lamports.toLocaleString()}`)
        else pass('sum ≤ escrow.amount', 'within escrow amount')
        if (ok) {
          setDeposited(deposited + amt)
          setStatus(1)
          summary = `EscrowDeposited — ${depositSol} SOL into vault (total ${((deposited + amt) / 1e9).toFixed(3)} SOL)`
        }
        break
      }
      case 'accept_escrow': {
        if (status !== 1) fail('InvalidEscrowStatus', `accept requires DEPOSITED(1), status = ${status === null ? 'none' : STATUS_NAMES[status]}`)
        else pass('status == DEPOSITED', 'DEPOSITED')
        if (role !== 'seller') fail('NotDesignatedSeller', `signer role = ${role}, only the designated seller accepts`)
        else pass('signer == seller', 'seller signs')
        if (deposited < lamports) fail('InsufficientDeposit', `deposited ${deposited.toLocaleString()} < amount ${lamports.toLocaleString()}`)
        else pass('deposit fully funded', `${deposited.toLocaleString()} lamports`)
        if (ok) {
          setStatus(2)
          setSettleDeadline(now + SETTLE_WINDOW)
          summary = `EscrowAccepted — seller accepts; settle unlocked at ${new Date((now + SETTLE_WINDOW) * 1000).toISOString().slice(0, 16).replace('T', ' ')}Z`
        }
        break
      }
      case 'settle_escrow': {
        if (status !== 2) fail('InvalidEscrowStatus', `settle requires ACCEPTED(2), status = ${status === null ? 'none' : STATUS_NAMES[status]}`)
        else pass('status == ACCEPTED', 'ACCEPTED')
        if (role !== 'buyer') fail('NotDesignatedBuyer', `signer role = ${role}, settle is buyer-only`)
        else pass('signer == buyer', 'buyer signs')
        if (now < settleDeadline) fail('SettlementNotYetEffective', `settle_deadline = ${new Date(settleDeadline * 1000).toISOString().slice(0, 16).replace('T', ' ')}Z is in the future`)
        else pass('now ≥ settle_deadline', 'settlement window elapsed')
        if (deposited < lamports) fail('InsufficientDeposit', 'settle requires the full deposit in the vault')
        else pass('deposit fully funded', 'full amount held')
        if (ok) {
          setStatus(3)
          summary = `EscrowSettled — ${amountSol} SOL released to seller; parcel ownership confirmed`
        }
        break
      }
      case 'cancel_escrow': {
        if (status === 0) {
          pass('status == CREATED', 'CREATED — seller-side cancel path')
          if (role !== 'seller') fail('NotDesignatedSeller', `signer role = ${role}; seller cancels before any deposit`)
          else pass('signer == seller', 'seller signs')
          if (deposited !== 0) fail('DepositExists', 'vault must be empty — use mutual_cancel_escrow instead')
          else pass('deposit_amount == 0', 'no deposit to refund')
        } else if (status === 1) {
          pass('status == DEPOSITED', 'DEPOSITED — buyer grace-period path')
          if (role !== 'buyer') fail('NotDesignatedBuyer', `signer role = ${role}; buyer cancels within grace period`)
          else pass('signer == buyer', 'buyer signs')
          if (now >= cancelDeadline) fail('CancelWindowExpired', `cancel_deadline = ${new Date(cancelDeadline * 1000).toISOString().slice(0, 16).replace('T', ' ')}Z passed`)
          else pass('now < cancel_deadline', 'within 7-day cancel window')
        } else {
          fail('InvalidEscrowStatus', `cancel only from CREATED/DEPOSITED, status = ${status === null ? 'none' : STATUS_NAMES[status]}`)
        }
        if (ok) {
          setStatus(4)
          summary = `EscrowCancelled — deposit ${(deposited / 1e9).toFixed(3)} SOL refunded to buyer`
        }
        break
      }
      case 'mutual_cancel_escrow': {
        if (status !== 1 && status !== 2) {
          fail('InvalidEscrowStatus', `mutual cancel only from DEPOSITED/ACCEPTED, status = ${status === null ? 'none' : STATUS_NAMES[status]}`)
        } else {
          pass('status ∈ {DEPOSITED, ACCEPTED}', STATUS_NAMES[status ?? 0])
          pass('buyer + seller both sign', 'two-party consent')
          if (deposited === 0) pass('vault empty', 'nothing to refund')
          else {
            pass('refund path', `${(deposited / 1e9).toFixed(3)} SOL back to buyer`)
          }
          setStatus(4)
          summary = `MutualCancelEscrow — both parties agreed; ${(deposited / 1e9).toFixed(3)} SOL refunded`
        }
        break
      }
    }
    setLast({ ix, ok, summary, verdict: { ok, headline: summary, gates } })
  }

  const nextIx: Action =
    status === null ? 'create_escrow' : status === 0 ? 'deposit_escrow' : status === 1 ? 'accept_escrow' : status === 2 ? 'settle_escrow' : 'cancel_escrow'

  const stepState = (i: number): 'done' | 'active' | 'pending' | 'failed' => {
    if (status === null) return 'pending'
    if (status === 4) return i === 0 ? 'done' : i === 1 && deposited > 0 ? 'done' : 'pending'
    if (i < status) return 'done'
    if (i === status) return 'active'
    return 'pending'
  }

  return (
    <div className="lab-body">
      <DemoBanner source="escrow.rs" />

      <div className="lab-card mt-3 space-y-2">
        <div className="flex items-center justify-between flex-wrap gap-2">
          <h3 className="text-[13px] font-semibold flex items-center gap-1.5">
            <Lock size={14} /> Escrow state machine
          </h3>
          <span className={`lab-badge ${status === null ? 'lab-badge-mut' : STATUS_STYLE[status]}`}>
            {status === null ? 'NO ESCROW' : STATUS_NAMES[status]}
          </span>
        </div>
        <div className="pg-steps">
          {['CREATED', 'DEPOSITED', 'ACCEPTED', 'SETTLED'].map((s, i) => (
            <span key={s} className={`pg-step pg-step-${stepState(i)}`}>
              {s}
            </span>
          ))}
          {status === 4 && <span className="pg-step pg-step-failed">CANCELLED</span>}
        </div>
        <div className="pg-fields">
          <Field label="escrow.amount (SOL)">
            <input
              className="select-input"
              type="number"
              step="0.1"
              value={amountSol}
              onChange={(e) => setAmountSol(Number(e.target.value) || 0)}
            />
          </Field>
          <Field label="deposit_amount (SOL)">
            <input
              className="select-input"
              type="number"
              step="0.1"
              value={depositSol}
              onChange={(e) => setDepositSol(Number(e.target.value) || 0)}
            />
          </Field>
          <Field label="signer role">
            <select className="select-input" value={role} onChange={(e) => setRole(e.target.value as Role)}>
              <option value="buyer">designated buyer</option>
              <option value="seller">designated seller</option>
              <option value="other">unrelated signer</option>
            </select>
          </Field>
          <Field label="buyer designated">
            <select
              className="select-input"
              value={buyerDesignated ? 'yes' : 'no'}
              onChange={(e) => setBuyerDesignated(e.target.value === 'yes')}
            >
              <option value="yes">buyer pubkey set</option>
              <option value="no">buyer = Pubkey::default()</option>
            </select>
          </Field>
        </div>
        <div className="pg-actions">
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('create_escrow')}>
            create_escrow
          </button>
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('deposit_escrow')}>
            deposit_escrow
          </button>
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('accept_escrow')}>
            accept_escrow
          </button>
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('settle_escrow')}>
            settle_escrow
          </button>
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('cancel_escrow')}>
            cancel_escrow
          </button>
          <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => run('mutual_cancel_escrow')}>
            mutual_cancel_escrow
          </button>
          <button
            className="btn btn-ghost px-2 py-1 text-[11px]"
            onClick={() => {
              setStatus(null)
              setDeposited(0)
              setLast(null)
              setCreatedAt(SIM_START)
            }}
          >
            reset escrow
          </button>
        </div>
        <p className="text-[10px] text-muted">
          Windows: buyer grace cancel = 7d from creation (deadline{' '}
          <span className="font-mono">{new Date(cancelDeadline * 1000).toISOString().slice(0, 16).replace('T', ' ')}Z</span>
          ), settle unlocked 3d after accept (deadline{' '}
          <span className="font-mono">
            {settleDeadline ? `${new Date(settleDeadline * 1000).toISOString().slice(0, 16).replace('T', ' ')}Z` : '—'}
          </span>
          ). Amount bounds: 0.1 SOL ≤ x ≤ 1,000,000 SOL.
        </p>
      </div>

      <div className="mt-3">{last && <VerdictCard title={`${last.ix} — guard trace`} verdict={last.verdict} />}</div>

      <Broadcast
        ix={last?.ix ?? nextIx}
        ok={last?.ok ?? false}
        summary={last?.summary ?? 'create_escrow not run yet — state has no escrow record'}
        note="Re-sends the last call to the demo chain (failed calls land failed with the exact TerraError code)."
      />

      <ClockBar now={now} onAdvance={(s) => setNow(now + s)} onReset={() => setNow(SIM_START)} />
    </div>
  )
}
