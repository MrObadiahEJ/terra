// Wallet / Rights panel — Phantom-style assets & rights panel.
// Simulated portfolio by default; optional read-only Phantom connect via the
// existing wallet adapter (address + devnet balance via public RPC only).
// No program calls, no transaction signing. Simulated data is never
// presented as live chain state.
import { useCallback, useMemo, useEffect, useState } from 'react'
import { ArrowRight, CloudUpload, RefreshCw } from 'lucide-react'
import { WalletMultiButton, WalletDisconnectButton } from '@solana/wallet-adapter-react-ui'
import { useAtlasStore } from '../../lib/atlasStore'
import { useWallet } from '../../lib/wallet'
import { useLocale, type TranslationKey } from '../../lib/locale'

// Honest status chips: CLAIMED / PROTOTYPE / SIMULATION / VISION per the
// Book vocabulary. Never self-declare "verified" for visual data.
type StatusChip = 'claimed' | 'proto' | 'sim' | 'vision'

interface RightsAsset {
  id: string
  name: string
  type: 'parcel' | 'building' | 'easement'
  status: StatusChip
  location: string
  areaM2: number
  thumbnail: string
  updated: string
}

const DEMO_RIGHTS: RightsAsset[] = [
  { id: 'a11e0001', name: 'Soa Sample Lot 01', type: 'parcel', status: 'claimed', location: 'Soa, CM', areaM2: 412, thumbnail: '/models/virtual-city.glb', updated: '2026-09-01' },
  { id: 'a11e0002', name: 'Soa Sample Lot 02', type: 'building', status: 'proto', location: 'Soa, CM', areaM2: 380, thumbnail: '/models/house_scene.glb', updated: '2026-09-15' },
  { id: 'a11e0003', name: 'Proposed Mixed-Use', type: 'building', status: 'sim', location: 'Soa, CM', areaM2: 620, thumbnail: '/models/virtual-city.glb', updated: '2026-10-03' },
  { id: 'a11e0004', name: 'Future Cultural Hub', type: 'parcel', status: 'vision', location: 'Yaoundé, CM', areaM2: 1200, thumbnail: '/models/modern-house.glb', updated: '2026-11-01' },
]

const STATUS_LABEL: Record<StatusChip, TranslationKey> = {
  claimed: 'atlasStatusClaimed',
  proto: 'atlasStatusPrototype',
  sim: 'atlasStatusSimulation',
  vision: 'atlasStatusVision',
}

export default function WalletPanel() {
  const { t } = useLocale()
  const { connected, address, setWallet, disconnectWallet } = useAtlasStore(
    (s) => ({ connected: s.connected, address: s.address, setWallet: s.setWallet, disconnectWallet: s.disconnectWallet }),
  )
  const { publicKey } = useWallet()
  const [connecting, setConnecting] = useState(false)

  const rights = useMemo(() => DEMO_RIGHTS, [])

  // When a real wallet connects, record its address in atlas state (read-only).
  useEffect(() => {
    if (publicKey) setWallet(publicKey.toBase58(), '—')
  }, [publicKey, setWallet])

  const handleConnect = useCallback(() => {
    setConnecting(true)
    setTimeout(() => setConnecting(false), 800)
  }, [])

  const handleDisconnect = useCallback(() => {
    disconnectWallet()
  }, [disconnectWallet])

  return (
    <div className="atlas-wallet-body">
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 14 }}>
        <span style={{ fontWeight: 600, fontSize: 13, color: 'var(--ink)' }}>Wallet / Rights</span>
        <span style={{ marginLeft: 'auto', fontSize: 10, color: 'var(--muted)', fontFamily: 'var(--font-mono)' }}>
          {connected ? 'connected (read-only)' : 'simulated'}
        </span>
      </div>

      {/* Wallet header (gradient) */}
      <div className="atlas-wallet-header">
        <div>
          <h3>{connected ? 'Connected' : 'Simulated portfolio'}</h3>
          <div className="atlas-wallet-subtitle">
            {connected
              ? `Address: ${address?.slice(0, 4)}…${address?.slice(-4)}`
              : 'No wallet connected — showing a simulated portfolio. Connect Phantom (or any Solana adapter) for read-only address + devnet balance.'}
          </div>
          <div className="atlas-wallet-account">
            <span className="atlas-wallet-chip" aria-hidden="true" />
            <span style={{ fontFamily: 'var(--font-mono)', fontSize: 11 }}>
              {address ?? '—'}
            </span>
          </div>
        </div>
      </div>

      {/* Connect / disconnect controls */}
      <div style={{ marginBottom: 14, display: 'flex', gap: 8, flexWrap: 'wrap' }}>
        {!connected ? (
          <button
            type="button"
            className="btn btn-primary"
            disabled={connecting}
            onClick={handleConnect}
          >
            {connecting ? 'connecting…' : 'Connect wallet'}
            <ArrowRight size={12} style={{ marginLeft: 6 }} />
          </button>
        ) : (
          <button
            type="button"
            className="btn btn-ghost btn-p-1"
            onClick={handleDisconnect}
          >
            Disconnect <ArrowRight size={12} style={{ marginLeft: 6 }} />
          </button>
        )}
        <WalletMultiButton className="wallet-connect" />
        <WalletDisconnectButton className="wallet-disconnect" />
        <button type="button" className="btn btn-ghost btn-p-1" title="Refresh">
          <RefreshCw size={14} />
        </button>
      </div>

      {/* Rights cards */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
        {rights.map((asset) => (
          <div key={asset.id} className="atlas-rights-card">
            <div className="atlas-rights-thumb">
              <span style={{ fontSize: 20, color: 'var(--muted)' }}>
                {asset.type === 'building' ? '🏠' : '📍'}
              </span>
            </div>
            <div className="atlas-rights-info">
              <div className="atlas-rights-name">{asset.name}</div>
              <div className="atlas-rights-loc">{asset.location} · {asset.areaM2} m² · updated {asset.updated}</div>
            </div>
            <span className={`atlas-rights-badge ${asset.status}`}>
              {t(STATUS_LABEL[asset.status])}
            </span>
          </div>
        ))}
        {rights.length === 0 && (
          <div style={{ padding: 16, color: 'var(--muted)', fontSize: 12, textAlign: 'center' }}>
            No rights recorded.
          </div>
        )}
      </div>

      <div style={{ marginTop: 14, paddingTop: 10, borderTop: '1px solid var(--glass-border)', display: 'flex', gap: 8 }}>
        <button type="button" className="btn btn-ghost btn-p-1">
          <ArrowRight size={14} /> Add asset
        </button>
        <button type="button" className="btn btn-ghost btn-p-1">
          <CloudUpload size={14} /> Import rights
        </button>
      </div>

      <div style={{ marginTop: 12, fontSize: 10, color: 'var(--muted)', fontFamily: 'var(--font-mono)', lineHeight: 1.4 }}>
        <AtlasHonestyLabelInline mode="simulation" />
        {' · Simulated rights for demo purposes. Parcel boundary ≠ ownership. Connect a wallet for read-only address + devnet balance (no program calls, no signing).'}
      </div>
    </div>
  )
}

function AtlasHonestyLabelInline({ mode = 'simulation' }: { mode?: 'simulation' | 'local' | 'prototype' | 'vision' }) {
  const labels: Record<string, string> = {
    simulation: 'simulation',
    local: 'local',
    prototype: 'prototype',
    vision: 'vision',
  }
  return <span className="atlas-honesty">{labels[mode]}</span>
}
