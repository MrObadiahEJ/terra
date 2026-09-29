import { useState } from 'react'
import GeometryVault from '../components/lab/GeometryVault'
import VerificationPipeline from '../components/lab/VerificationPipeline'
import CrossBorderGate from '../components/lab/CrossBorderGate'
import { FlaskConical } from 'lucide-react'

const TABS = [
  { id: 'vault', label: 'Geometry Vault', hint: 'RFC-013' },
  { id: 'pipeline', label: 'Verification pipeline', hint: 'claim → quorum → fact' },
  { id: 'crossborder', label: 'Cross-border gate', hint: 'RFC-012 Phase 10' },
] as const

type TabId = (typeof TABS)[number]['id']

const TAB_IDS: TabId[] = ['vault', 'pipeline', 'crossborder']

function initialTab(): TabId {
  const t = new URLSearchParams(window.location.search).get('tab')
  return (TAB_IDS as string[]).includes(t ?? '') ? (t as TabId) : 'vault'
}

export default function LabPage() {
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
      </main>

      <footer className="lab-foot">
        Sources of truth: <span className="font-mono">spatial_asset.rs</span> (RFC-013),{' '}
        <span className="font-mono">verification/*</span>, <span className="font-mono">routing.rs</span> +
        <span className="font-mono">cross_border.rs</span> (RFC-012 Phase 10). Error codes are the exact{' '}
        <span className="font-mono">6000 + variant index</span> values from{' '}
        <span className="font-mono">TerraError</span> (237 variants).
      </footer>
    </div>
  )
}
