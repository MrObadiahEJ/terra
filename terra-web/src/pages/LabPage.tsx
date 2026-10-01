import { useState } from 'react'
import { Link } from 'react-router-dom'
import GeometryVault from '../components/lab/GeometryVault'
import VerificationPipeline from '../components/lab/VerificationPipeline'
import CrossBorderGate from '../components/lab/CrossBorderGate'
import Playground from '../components/lab/Playground'
import { useTxStore } from '../lib/txStore'
import { FlaskConical } from 'lucide-react'

const TABS = [
  { id: 'vault', label: 'Geometry Vault', hint: 'RFC-013 A+B' },
  { id: 'pipeline', label: 'Verification pipeline', hint: 'claim → quorum → fact' },
  { id: 'crossborder', label: 'Cross-border gate', hint: 'RFC-012 Phase 10' },
  { id: 'playground', label: 'Playground', hint: '8 protocol state machines' },
] as const

type TabId = (typeof TABS)[number]['id']

const TAB_IDS: TabId[] = ['vault', 'pipeline', 'crossborder', 'playground']

function initialTab(): TabId {
  const t = new URLSearchParams(window.location.search).get('tab')
  return (TAB_IDS as string[]).includes(t ?? '') ? (t as TabId) : 'vault'
}

export default function LabPage() {
  const txs = useTxStore((s) => s.txs)
  const [tab, setTabState] = useState<TabId>(initialTab)
  const setTab = (t: TabId) => {
    setTabState(t)
    // Shareable deep-link: /lab?tab=pipeline
    const url = new URL(window.location.href)
    url.searchParams.set('tab', t)
    window.history.replaceState(null, '', url)
  }

  return (
    <div className="lab">
      <header className="lab-head">
        <h1 className="lab-title">
          <FlaskConical size={20} /> Terra Lab
        </h1>
        <p className="lab-sub">
          Interactive experiments that mirror the on-chain program 1:1 — same guards, same error codes, same
          event names. Everything runs offline in your browser: no wallet, no API, no devnet.
        </p>
      </header>

      <nav className="lab-tabs" aria-label="Lab experiments">
        {TABS.map((t) => (
          <button
            key={t.id}
            className={`lab-tab ${tab === t.id ? 'active' : ''}`}
            onClick={() => setTab(t.id)}
          >
            {t.label}
            <span className="lab-tab-hint">{t.hint}</span>
          </button>
        ))}
      </nav>

      <main className="lab-main">
        {tab === 'vault' && <GeometryVault />}
        {tab === 'pipeline' && <VerificationPipeline />}
        {tab === 'crossborder' && <CrossBorderGate />}
        {tab === 'playground' && <Playground />}
      </main>

      <footer className="lab-foot">
        <div className="lab-recent">
          <span className="lab-recent-label">Recent chain activity</span>
          {txs.slice(0, 3).map((t) => (
            <span className="lab-recent-item" key={t.sig}>
              <span className={`tx-dot ${t.status}`} style={{ marginTop: 0 }} />
              <span className="font-mono text-[11px]">{t.instruction}</span>
              <span className={`lab-badge ${t.status === 'confirmed' ? 'lab-badge-ok' : 'lab-badge-err'}`}>
                {t.status}
              </span>
            </span>
          ))}
          <Link className="lab-recent-link" to="/transactions">
            View all →
          </Link>
        </div>
        Sources of truth: <span className="font-mono">spatial_asset.rs</span> (RFC-013),{' '}
        <span className="font-mono">verification/*</span>, <span className="font-mono">routing.rs</span> +
        <span className="font-mono">cross_border.rs</span> (RFC-012 Phase 10), and the playground modules{' '}
        <span className="font-mono">escrow.rs</span>, <span className="font-mono">dispute.rs</span>,{' '}
        <span className="font-mono">staking.rs</span>, <span className="font-mono">fraud_governance.rs</span>,{' '}
        <span className="font-mono">device_identity.rs</span>, <span className="font-mono">recovery.rs</span>,{' '}
        <span className="font-mono">zk/*</span>. Error codes are the exact{' '}
        <span className="font-mono">6000 + variant index</span> values from{' '}
        <span className="font-mono">TerraError</span> (241 variants).
      </footer>
    </div>
  )
}
