import { type ReactNode } from 'react'
import { Radio } from 'lucide-react'
import { fmtSim, tx, type Verdict } from './helpers'

export function VerdictCard({ title, verdict }: { title: string; verdict: Verdict }) {
  return (
    <div className="lab-card">
      <div className="flex items-center justify-between mb-2 gap-2">
        <h3 className="text-[13px] font-semibold">{title}</h3>
        <span className={`lab-badge ${verdict.ok ? 'lab-badge-ok' : 'lab-badge-err'}`}>
          {verdict.ok ? 'PASS' : 'FAIL'}
        </span>
      </div>
      <p className={`text-[12px] font-medium mb-2 ${verdict.ok ? 'text-emerald-700' : 'text-red-700'}`}>
        {verdict.headline}
      </p>
      <ol className="lab-gates">
        {verdict.gates.map((g, i) => (
          <li key={i} className={`gate-${g.state}`}>
            <span className="gate-mark">{g.state === 'pass' ? '✓' : g.state === 'fail' ? '✗' : '–'}</span>
            <span className="font-mono text-[11px]">{g.label}</span>
            <span className="text-[11px] text-muted break-all">{g.detail}</span>
          </li>
        ))}
      </ol>
    </div>
  )
}

export function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="text-[11px] text-muted">
      {label}
      {children}
    </label>
  )
}

/** Sign & send card that emits a demo tx with a `Name (NNNN)` summary. */
export function Broadcast({
  ix,
  ok,
  summary,
  note,
}: {
  ix: string
  ok: boolean
  summary: string
  note?: string
}) {
  return (
    <div className="lab-card mt-3 flex items-center justify-between gap-2 flex-wrap">
      <div>
        <h3 className="text-[13px] font-semibold flex items-center gap-1.5">
          <Radio size={14} /> Broadcast <span className="font-mono">{ix}</span>
        </h3>
        <p className="text-[11px] text-muted mt-1">
          {note ??
            (ok
              ? 'Current state passes every guard — transaction confirms on the demo chain.'
              : 'Current state fails a guard — transaction lands failed with the exact error code.')}
        </p>
      </div>
      <button className="btn btn-primary" onClick={() => tx(ix, ok, summary)} title={summary}>
        <Radio size={13} /> Sign &amp; send
      </button>
    </div>
  )
}

export function DemoBanner({ source }: { source: string }) {
  return (
    <div className="pg-banner">
      <span className="lab-badge lab-badge-warn">DEMO</span>
      <span className="text-[11px]">
        Offline state machine mirroring <span className="font-mono">{source}</span> — same guard order, same{' '}
        <span className="font-mono">TerraError</span> codes read live from the IDL. No wallet, no signature,
        no chain writes.
      </span>
    </div>
  )
}

export function ClockBar({
  now,
  onAdvance,
  onReset,
  extra,
}: {
  now: number
  onAdvance: (secs: number) => void
  onReset: () => void
  extra?: ReactNode
}) {
  return (
    <div className="pg-clock">
      <span className="text-[11px] text-muted">
        simulated clock: <span className="font-mono">{fmtSim(now)}</span>
      </span>
      <div className="flex gap-1 flex-wrap">
        {extra}
        <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => onAdvance(21_600)}>
          +6h
        </button>
        <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => onAdvance(86_400)}>
          +1d
        </button>
        <button className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => onAdvance(259_200)}>
          +3d
        </button>
        <button className="btn btn-ghost px-2 py-1 text-[11px]" onClick={onReset}>
          reset
        </button>
      </div>
    </div>
  )
}
