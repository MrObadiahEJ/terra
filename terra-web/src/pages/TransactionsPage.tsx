import { useEffect, useMemo, useState } from 'react'
import { Link } from 'react-router-dom'
import { Connection } from '@solana/web3.js'
import { Activity, ChevronDown, Radio, RotateCcw, Search } from 'lucide-react'
import type { DemoTx } from '../lib/txStore'
import { reportTx, useTxStore } from '../lib/txStore'
import { useLabVault, GEOMETRY_SOURCES, SPATIAL_DIMENSIONS, MAX_GEOMETRY_VERSIONS } from '../lib/labStore'
import { TERRA_RPC_URL } from '../lib/constants'
import { api } from '../lib/api'
import { useWallet } from '../lib/wallet'
import IsoLandSkeleton from '../components/lab/IsoLandSkeleton'

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
type Probe = 'checking' | 'online' | 'offline'

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms))

function withTimeout<T>(p: Promise<T>, ms: number): Promise<T> {
  return Promise.race([p, new Promise<T>((_, rej) => setTimeout(() => rej(new Error('timeout')), ms))])
}

function shortSig(sig: string) {
  return sig.length > 12 ? `${sig.slice(0, 4)}…${sig.slice(-4)}` : sig
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

function fmtIso(iso: string | null): string {
  if (!iso) return '—'
  return iso.replace('T', ' ').slice(0, 19) + ' UTC'
}

function durationBetween(a: string, b: string): string {
  const ms = new Date(b).getTime() - new Date(a).getTime()
  if (!isFinite(ms) || ms < 0) return ''
  const s = Math.round(ms / 1000)
  if (s < 60) return `+${s}s`
  const m = Math.floor(s / 60)
  if (m < 60) return `+${m}m ${s % 60}s`
  return `+${Math.floor(m / 60)}h ${m % 60}m`
}

function isLogError(line: string) {
  return /Error|failed|simulation/i.test(line) && !/success/i.test(line)
}

/** Best-effort devnet confirmation for wallet-signed txs (fails soft offline). */
function LiveStatus({ sig }: { sig: string }) {
  const [st, setSt] = useState<string>('checking…')
  useEffect(() => {
    let cancel = false
    ;(async () => {
      try {
        const conn = new Connection(TERRA_RPC_URL, 'confirmed')
        const r = await withTimeout(
          conn.getSignatureStatuses([sig], { searchTransactionHistory: true }),
          5000,
        )
        if (cancel) return
        const s = r.value[0]
        if (!s) setSt('not found on devnet')
        else if (s.err) setSt('failed on devnet')
        else setSt(`devnet ${s.confirmationStatus ?? 'confirmed'}`)
      } catch {
        if (!cancel) setSt('devnet unreachable')
      }
    })()
    return () => {
      cancel = true
    }
  }, [sig])
  return <span className="tx-live-status">devnet status: {st}</span>
}

/** Realtime pills: off-chain validator API + devnet RPC + wallet identity. */
function StatusPills() {
  const { publicKey } = useWallet()
  const [apiSt, setApiSt] = useState<Probe>('checking')
  const [apiCount, setApiCount] = useState<number | null>(null)
  const [rpcSt, setRpcSt] = useState<Probe>('checking')
  const [slot, setSlot] = useState<number | null>(null)

  useEffect(() => {
    let cancel = false
    const probe = async () => {
      try {
        const p = await withTimeout(api.listParcels(), 5000)
        if (cancel) return
        setApiSt('online')
        setApiCount(p.length)
      } catch {
        if (!cancel) setApiSt('offline')
      }
      try {
        const conn = new Connection(TERRA_RPC_URL, 'confirmed')
        const s = await withTimeout(conn.getSlot('confirmed'), 5000)
        if (cancel) return
        setRpcSt('online')
        setSlot(s)
      } catch {
        if (!cancel) setRpcSt('offline')
      }
    }
    probe()
    const t = setInterval(probe, 20000)
    return () => {
      cancel = true
      clearInterval(t)
    }
  }, [])

  return (
    <div className="tx-pills">
      <span className={`tx-pill probe-${apiSt}`} title="Off-chain validator/API feed">
        <span className="tx-pill-dot" />
        {apiSt === 'checking'
          ? 'Checking validator API…'
          : apiSt === 'online'
            ? `Validator API live · ${apiCount ?? 0} parcels`
            : 'API offline — demo mode'}
      </span>
      <span className={`tx-pill probe-${rpcSt}`} title="Solana devnet RPC">
        <span className="tx-pill-dot" />
        {rpcSt === 'checking'
          ? 'Checking devnet…'
          : rpcSt === 'online'
            ? `Devnet slot ${slot}`
            : 'RPC unreachable — demo mode'}
      </span>
      <span className={`tx-pill ${publicKey ? 'probe-online' : ''}`} title="Terra wallet">
        <span className="tx-pill-dot" />
        {publicKey ? `Terra wallet ${shortSig(publicKey.toBase58())}` : 'No wallet — demo signer'}
      </span>
    </div>
  )
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
        <div>
          <span className="tx-meta-label">Source</span>
          <span className="font-mono text-[11px]">
            {tx.source === 'wallet' ? 'wallet · signed' : 'demo · simulated'}
          </span>
        </div>
      </div>

      {tx.source === 'wallet' && (
        <div className="tx-live">
          <span className="lab-badge lab-badge-info">LIVE · devnet</span>
          <a
            className="tx-live-link"
            href={`https://explorer.solana.com/tx/${tx.sig}?cluster=devnet`}
            target="_blank"
            rel="noreferrer"
          >
            View on Solana Explorer ↗
          </a>
          <LiveStatus sig={tx.sig} />
        </div>
      )}

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

function VersionField({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  return (
    <div className="txv-field">
      <span className="txv-field-label">{label}</span>
      <span className={mono ? 'font-mono text-[11px] break-all' : 'text-[12px]'}>{value}</span>
    </div>
  )
}

/** Land version history: every anchored geometry version through time, each
 *  with its state (anchored → verified) and a WebGL-free 3D skeleton. */
function LandVersionsView() {
  const versions = useLabVault((s) => s.versions)
  const latest = versions.length ? versions[versions.length - 1].version : -1
  const verified = versions.filter((v) => v.verified).length

  if (versions.length === 0) {
    return (
      <div className="lab-card txv-empty">
        <h3 className="text-[13px] font-semibold mb-1">No land versions yet</h3>
        <p className="text-[12px] text-muted mb-3">
          Anchor geometry in the Lab — each version then appears here with its full state history
          (anchored → verified) and a WebGL-free 3D skeleton of that exact snapshot.
        </p>
        <Link className="btn btn-primary" to="/lab?tab=vault">
          Open Geometry Vault →
        </Link>
      </div>
    )
  }

  return (
    <div>
      <div className="txv-summary">
        <span>
          <b className="font-mono">{versions.length}</b> anchored ·{' '}
          <b className="font-mono">{verified}</b> verified · cap {MAX_GEOMETRY_VERSIONS}
        </span>
        <Link className="btn btn-secondary px-2 py-1 text-[11px]" to="/lab?tab=vault">
          Anchor new version →
        </Link>
      </div>

      <div className="txv-list">
        {versions.map((v) => {
          const src = GEOMETRY_SOURCES[v.source]
          const dim = SPATIAL_DIMENSIONS[v.dimension]
          const who = (s: string) => (s.length > 12 ? shortSig(s) : s)
          return (
            <div className="txv-item" key={v.version}>
              <div className="txv-rail" aria-hidden>
                <span className={`txv-node ${v.verified ? 'ok' : 'pending'}`} />
                {v.version !== latest && <span className="txv-line" />}
              </div>
              <div className="txv-card lab-card">
                <div className="txv-card-head">
                  <span className="txv-ver">v{v.version}</span>
                  <span className={`lab-badge ${v.verified ? 'lab-badge-ok' : 'lab-badge-warn'}`}>
                    {v.verified ? 'VERIFIED' : 'ANCHORED'}
                  </span>
                  <span className="txv-pos">
                    {v.version === latest ? 'latest' : `superseded by v${v.version + 1}+`}
                  </span>
                  <span className="flex-1" />
                  <span className="txv-time">{fmtIso(v.submittedAt)}</span>
                </div>
                <div className="txv-body">
                  <div className="txv-fields">
                    <VersionField
                      label="verified at"
                      value={
                        v.verified && v.verifiedAt
                          ? `${fmtIso(v.verifiedAt)} (${durationBetween(v.submittedAt, v.verifiedAt)})`
                          : 'pending'
                      }
                    />
                    <VersionField label="source" value={`${src?.id ?? v.source} — ${src?.label ?? '?'}`} />
                    <VersionField label="dimension" value={dim?.label ?? String(v.dimension)} />
                    <VersionField label="submitted by" value={who(v.submittedBy)} />
                    <VersionField label="verified by" value={v.verifiedBy ? who(v.verifiedBy) : '—'} />
                    <VersionField label="storage ref" value={v.storageReference} />
                    <VersionField label="digest" value={`${v.geometryHash.slice(0, 24)}…`} mono />
                  </div>
                  <div className="txv-iso" title="3D skeleton of this exact version">
                    <IsoLandSkeleton ring={v.ring} compact />
                  </div>
                </div>
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}

export default function TransactionsPage() {
  const txs = useTxStore((s) => s.txs)
  const clear = useTxStore((s) => s.clear)
  const reseed = useTxStore((s) => s.reseed)
  const { publicKey } = useWallet()

  const [view, setView] = useState<'feed' | 'versions'>(() => {
    const t = new URLSearchParams(window.location.search).get('view')
    return t === 'versions' ? 'versions' : 'feed'
  })
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
    const signer = publicKey?.toBase58()
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
          { signer },
        )
      } else {
        reportTx(opt.id, true, `${opt.label} — demo broadcast confirmed (recent blockhash)`, { signer })
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
          Chain explorer for the Terra demo network — Lab actions, broadcasts and wallet-signed
          transactions with real-shaped signatures, accounts, program logs and error codes. Wallet
          transactions are verified against devnet; everything else runs fully offline.
        </p>
        <StatusPills />
      </header>

      <nav className="lab-tabs" aria-label="Transactions views">
        <button
          className={`lab-tab ${view === 'feed' ? 'active' : ''}`}
          onClick={() => setView('feed')}
        >
          Transaction feed
          <span className="lab-tab-hint">demo chain · live wallet txs</span>
        </button>
        <button
          className={`lab-tab ${view === 'versions' ? 'active' : ''}`}
          onClick={() => setView('versions')}
        >
          Land versions
          <span className="lab-tab-hint">geometry through time</span>
        </button>
      </nav>

      {view === 'feed' && (
        <>
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
                        <span
                          className={`lab-badge ${tx.status === 'confirmed' ? 'lab-badge-ok' : 'lab-badge-err'}`}
                        >
                          {tx.status}
                        </span>
                        {tx.source === 'wallet' && <span className="lab-badge lab-badge-info">LIVE</span>}
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
        </>
      )}

      {view === 'versions' && <LandVersionsView />}

      <footer className="lab-foot">
        Demo feed for offline environments; wallet transactions are checked against{' '}
        <span className="font-mono">devnet</span>. Error codes are the exact{' '}
        <span className="font-mono">6000 + variant index</span> values from{' '}
        <span className="font-mono">TerraError</span>; program logs mirror Anchor's on-chain format.
      </footer>
    </div>
  )
}
