import { useMemo, useState } from 'react'
import { Activity, ChevronDown, Radio, RotateCcw, Search } from 'lucide-react'
import type { DemoTx } from '../lib/txStore'
import { reportTx, useTxStore } from '../lib/txStore'

const COMPOSE: Array<{ id: string; label: string; fail: { code: string; name: string } | null }> = [
  { id: 'init_spatial_asset', label: 'init_spatial_asset', fail: { code: '6231', name: 'InvalidSpatialDimension' } },
  { id: 'append_geometry_version', label: 'append_geometry_version', fail: { code: '6002', name: 'EmptyGeometryHash' } },
  { id: 'verify_geometry_version', label: 'verify_geometry_version', fail: { code: '6236', name: 'GeometryAlreadyVerified' } },
  { id: 'submit_observation_v2', label: 'submit_observation_v2', fail: { code: '6131', name: 'InvalidClaimStatus' } },
  { id: 'verify_claim', label: 'verify_claim', fail: { code: '6137', name: 'QuorumNotReached' } },
  { id: 'record_cross_border_verification', label: 'record_cross_border_verification', fail: { code: '6227', name: 'SameJurisdiction' } },
  { id: 'transfer_rights', label: 'transfer_rights', fail: { code: null, name: 'BlockhashNotFound' } },
]

const STATUS_FILTERS = [
  { id: 'all', label: 'All' },
  { id: 'confirmed', label: 'Confirmed' },
  { id: 'failed', label: 'Failed' },
] as const

type StatusFilter = (typeof STATUS_FILTERS)[number]['id']
type Phase = 'signing' | 'broadcasting' | 'confirming' | null

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms))

function shortSig(sig: string) {
  return `${sig.slice(0, 4)}…${sig.slice(-4)}`
}

function timeAgo(t: number) {
  const s = Math.max(1, Math.floor((Date.now() - t) / 1000))
  if (s < 60) return `${s}s ago`
  const m = Math.floor(s / 60)
  if (m < 60) return `${m}m ago`
  const h = Math.floor(m / 60)
  if (h < 24) return `${h}h ago`
  return `${Math.floor(h / 24)}d ago`
}

function isLogError(line: string) {
  return /Error|failed|simulation/i.test(line) && !/success/i.test(line)
}

function TxDetail({ tx }: { tx: DemoTx }) {
  return (
    <div className="tx-detail">
      <div className="tx-meta-grid">
        <div>
          <span className="tx-meta-label">Signature</span>
          <span className="font-mono text-[11px] break-all">{tx.sig}</span>
        </div>
        <div>
          <span className="tx-meta-label">Block time</span>
          <span className="font-mono text-[11px]">{new Date(tx.blockTime).toISOString().replace('.000Z', 'Z')}</span>
        </div>
        <div>
          <span className="tx-meta-label">Slot</span>
          <span className="font-mono text-[11px]">{tx.slot}</span>
        </div>
        <div>
          <span className="tx-meta-label">Fee / compute</span>
          <span className="font-mono text-[11px]">
            {tx.feeLamports} lamports · {tx.computeUnits} CU
          </span>
        </div>
      </div>

      {tx.error && (
        <div className="tx-error-card">
          <b>
            {tx.error.name}
            {tx.error.code ? ` (${tx.error.code})` : ''}
          </b>
          <span>{tx.summary}</span>
        </div>
      )}

      <div className="tx-section-label">Accounts</div>
      <div className="tx-accts">
        {tx.accounts.map((a, i) => (
          <div className="tx-acct" key={a.pubkey + i}>
            <span className="font-mono text-[11px] break-all">{a.pubkey}</span>
            <span className={`tx-role role-${a.role}`}>{a.role}</span>
          </div>
        ))}
      </div>

      <div className="tx-section-label">Program logs</div>
      <pre className="tx-logs">
        {tx.logs.map((l, i) => (
          <div key={i} className={isLogError(l) ? 'e' : /success/i.test(l) ? 's' : ''}>
            {l}
          </div>
        ))}
      </pre>
    </div>
  )
}

export default function TransactionsPage() {
  const txs = useTxStore((s) => s.txs)
  const clear = useTxStore((s) => s.clear)
  const reseed = useTxStore((s) => s.reseed)

  const [status, setStatus] = useState<StatusFilter>('all')
  const [query, setQuery] = useState('')
  const [openSig, setOpenSig] = useState<string | null>(null)

  const [composeIx, setComposeIx] = useState(COMPOSE[0].id)
  const [injectFail, setInjectFail] = useState(false)
  const [phase, setPhase] = useState<Phase>(null)

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase()
    return txs.filter((t) => {
      if (status !== 'all' && t.status !== status) return false
      if (!q) return true
      return (
        t.sig.toLowerCase().includes(q) ||
        t.instruction.toLowerCase().includes(q) ||
        t.summary.toLowerCase().includes(q)
      )
    })
  }, [txs, status, query])

  const stats = useMemo(() => {
    const total = txs.length
    const confirmed = txs.filter((t) => t.status === 'confirmed').length
    const fees = txs.reduce((s, t) => s + t.feeLamports, 0)
    const cu = total ? Math.round(txs.reduce((s, t) => s + t.computeUnits, 0) / total) : 0
    return {
      total,
      rate: total ? Math.round((confirmed / total) * 100) : 0,
      fees: (fees / 1e9).toFixed(5),
      cu,
    }
  }, [txs])

  const send = async () => {
    if (phase) return
    const opt = COMPOSE.find((c) => c.id === composeIx)!
    try {
      setPhase('signing')
      await sleep(350)
      setPhase('broadcasting')
      await sleep(450)
      setPhase('confirming')
      await sleep(650)
      if (injectFail && opt.fail) {
        reportTx(
          opt.id,
          false,
          `${opt.fail.name} (${opt.fail.code}): demo failure injected by the composer`,
        )
      } else {
        reportTx(opt.id, true, `${opt.label} — demo broadcast confirmed (recent blockhash)`)
      }
    } finally {
      setPhase(null)
    }
  }

  return (
    <div className="tx">
      <header className="lab-head">
        <h1 className="lab-title">
          <Activity size={20} /> Transactions
        </h1>
        <p className="lab-sub">
          Demo chain explorer — every Lab action, demo registration and broadcast below produces a
          simulated transaction with a real-shaped signature, accounts, program logs and error codes.
          Runs fully offline: no wallet, no RPC, no devnet.
        </p>
      </header>

      <div className="tx-stats">
        <div className="tx-stat">
          <span className="tx-stat-label">Transactions</span>
          <b>{stats.total}</b>
        </div>
        <div className="tx-stat">
          <span className="tx-stat-label">Confirmation rate</span>
          <b>{stats.rate}%</b>
        </div>
        <div className="tx-stat">
          <span className="tx-stat-label">Fees paid</span>
          <b>{stats.fees} SOL</b>
        </div>
        <div className="tx-stat">
          <span className="tx-stat-label">Avg compute</span>
          <b>{stats.cu} CU</b>
        </div>
      </div>

      <section className="lab-card tx-composer">
        <div className="flex items-center gap-1.5 mb-2">
          <Radio size={14} />
          <h3 className="text-[13px] font-semibold">Broadcast a demo transaction</h3>
        </div>
        <div className="tx-composer-row">
          <label className="text-[11px] text-muted">
            Instruction
            <select
              className="select-input"
              value={composeIx}
              onChange={(e) => setComposeIx(e.target.value)}
              disabled={!!phase}
            >
              {COMPOSE.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.label}
                </option>
              ))}
            </select>
          </label>
          <label className="tx-fail-toggle">
            <input
              type="checkbox"
              checked={injectFail}
              onChange={(e) => setInjectFail(e.target.checked)}
              disabled={!!phase}
            />
            Inject failure
            <span className="font-mono text-[10px] text-muted">
              {(() => {
                const f = COMPOSE.find((c) => c.id === composeIx)?.fail
                return f ? `${f.name} (${f.code})` : 'runtime error'
              })()}
            </span>
          </label>
          <button className="btn btn-primary" onClick={send} disabled={!!phase}>
            {phase ? `${phase}…` : 'Sign & send'}
          </button>
        </div>
        <div className="tx-stepper" aria-hidden>
          {(['signing', 'broadcasting', 'confirming'] as const).map((p, i) => (
            <span key={p} className={`tx-step ${phase === p ? 'on' : ''} ${phase ? '' : 'idle'}`}>
              {i > 0 && <span className="tx-step-arrow">→</span>}
              {p}
            </span>
          ))}
        </div>
      </section>

      <div className="tx-toolbar">
        <div className="tx-chips">
          {STATUS_FILTERS.map((f) => (
            <button
              key={f.id}
              className={`tx-chip ${status === f.id ? 'active' : ''}`}
              onClick={() => setStatus(f.id)}
            >
              {f.label}
            </button>
          ))}
        </div>
        <label className="tx-search">
          <Search size={13} />
          <input
            className="text-input"
            placeholder="Search signature, instruction, error…"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
        </label>
        <button
          className="btn btn-secondary px-2 py-1 text-[11px] gap-1"
          onClick={() => (txs.length ? clear() : reseed())}
        >
          <RotateCcw size={12} /> {txs.length ? 'Clear feed' : 'Restore demo feed'}
        </button>
      </div>

      <div className="tx-list">
        {filtered.length === 0 && (
          <div className="tx-empty">
            {txs.length === 0
              ? 'Feed cleared — restore it or broadcast a transaction above. Lab actions also repopulate it.'
              : 'No transactions match this filter.'}
          </div>
        )}
        {filtered.map((tx) => {
          const isOpen = openSig === tx.sig
          return (
            <div key={tx.sig} className="tx-row-wrap">
              <button
                className="tx-row"
                onClick={() => setOpenSig(isOpen ? null : tx.sig)}
                aria-expanded={isOpen}
              >
                <span className={`tx-dot ${tx.status}`} />
                <span className="tx-main">
                  <span className="tx-row-top">
                    <span className="tx-ix">{tx.instruction}</span>
                    <span className={`lab-badge ${tx.status === 'confirmed' ? 'lab-badge-ok' : 'lab-badge-err'}`}>
                      {tx.status}
                    </span>
                    {tx.error && (
                      <span className="tx-errcode">
                        {tx.error.name}
                        {tx.error.code ? ` (${tx.error.code})` : ''}
                      </span>
                    )}
                  </span>
                  <span className="tx-sum">{tx.summary}</span>
                  <span className="tx-meta">
                    <span className="font-mono">{shortSig(tx.sig)}</span> · slot {tx.slot} ·{' '}
                    {tx.feeLamports} lamports · {timeAgo(tx.blockTime)}
                  </span>
                </span>
                <ChevronDown size={14} className={`tx-caret ${isOpen ? 'open' : ''}`} />
              </button>
              {isOpen && <TxDetail tx={tx} />}
            </div>
          )
        })}
      </div>

      <footer className="lab-foot">
        Simulated feed for demos and offline environments. Error codes are the exact{' '}
        <span className="font-mono">6000 + variant index</span> values from{' '}
        <span className="font-mono">TerraError</span>; program logs mirror Anchor's on-chain format.
      </footer>
    </div>
  )
}
