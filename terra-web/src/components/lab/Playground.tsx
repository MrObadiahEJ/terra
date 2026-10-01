import { useState } from 'react'
import { Gamepad2 } from 'lucide-react'
import RoutingPanel from './playground/RoutingPanel'
import EscrowPanel from './playground/EscrowPanel'
import DisputePanel from './playground/DisputePanel'
import FraudPanel from './playground/FraudPanel'
import StakingPanel from './playground/StakingPanel'
import DevicePanel from './playground/DevicePanel'
import RecoveryPanel from './playground/RecoveryPanel'
import ZkPanel from './playground/ZkPanel'

const SUBS = [
  { id: 'routing', label: 'Routing', hint: 'routing.rs' },
  { id: 'escrow', label: 'Escrow', hint: 'escrow.rs' },
  { id: 'disputes', label: 'Disputes', hint: 'dispute.rs' },
  { id: 'fraud', label: 'Fraud', hint: 'fraud_governance.rs' },
  { id: 'staking', label: 'Staking', hint: 'staking.rs' },
  { id: 'devices', label: 'Devices', hint: 'device_identity.rs' },
  { id: 'recovery', label: 'Recovery', hint: 'recovery.rs' },
  { id: 'zk', label: 'ZK proofs', hint: 'zk/mod.rs' },
] as const

type SubId = (typeof SUBS)[number]['id']
const SUB_IDS: SubId[] = SUBS.map((s) => s.id)

function initialSub(): SubId {
  const pt = new URLSearchParams(window.location.search).get('pt')
  return (SUB_IDS as string[]).includes(pt ?? '') ? (pt as SubId) : 'routing'
}

export default function Playground() {
  const [sub, setSubState] = useState<SubId>(initialSub)
  const setSub = (id: SubId) => {
    setSubState(id)
    // Deep link: /lab?tab=playground&pt=escrow
    const url = new URL(window.location.href)
    url.searchParams.set('pt', id)
    window.history.replaceState(null, '', url)
  }

  return (
    <div className="lab-body">
      <div className="lab-card mb-3">
        <div className="flex items-center gap-1.5 flex-wrap">
          <Gamepad2 size={15} />
          <h3 className="text-[13px] font-semibold">Protocol playground — 8 state machines</h3>
          <span className="lab-badge lab-badge-warn">DEMO</span>
        </div>
        <p className="text-[11px] text-muted mt-1">
          Each sub-tab mirrors one program module: the same state transitions, the same guard order, and the
          exact <span className="font-mono">TerraError</span> names+codes read live from{' '}
          <span className="font-mono">TerraError = 6000 + variant index</span> in the IDL. Actions emit demo
          transactions to <a href="/transactions">/transactions</a> — wrong inputs land failed with the code you
          saw in the trace, never a silently fixed value. Time-based guards (7d unbonding, 90d disputes, 48h
          injection timelock, +24h/+30d fraud windows) use the per-panel simulated clock.
        </p>
      </div>

      <nav className="pg-tabs" aria-label="Playground modules">
        {SUBS.map((s) => (
          <button
            key={s.id}
            className={`pg-tab ${sub === s.id ? 'active' : ''}`}
            onClick={() => setSub(s.id)}
          >
            {s.label}
            <span className="pg-tab-hint">{s.hint}</span>
          </button>
        ))}
      </nav>

      <div className="mt-3">
        {sub === 'routing' && <RoutingPanel />}
        {sub === 'escrow' && <EscrowPanel />}
        {sub === 'disputes' && <DisputePanel />}
        {sub === 'fraud' && <FraudPanel />}
        {sub === 'staking' && <StakingPanel />}
        {sub === 'devices' && <DevicePanel />}
        {sub === 'recovery' && <RecoveryPanel />}
        {sub === 'zk' && <ZkPanel />}
      </div>
    </div>
  )
}
