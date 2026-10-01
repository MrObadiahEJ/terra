import { useEffect } from 'react'
import { NavLink } from 'react-router-dom'
import { WalletMultiButton, WalletDisconnectButton } from '@solana/wallet-adapter-react-ui'
import {
  Activity,
  ArrowUpRight,
  FlaskConical,
  Globe2,
  Moon,
  Network,
  BriefcaseBusiness,
  ShieldCheck,
  Sun,
} from 'lucide-react'
import { useWallet } from '../../lib/wallet'
import { useAppStore } from '../../store/appStore'

interface Props {
  theme: 'light' | 'dark'
  onToggleTheme: () => void
}

const navigation = [
  { to: '/', label: 'Explore', icon: Globe2, end: true },
  { to: '/transactions', label: 'Activity', icon: Activity },
  { to: '/demo', label: 'Demo', icon: ShieldCheck },
  { to: '/network', label: 'Network', icon: Network },
  { to: '/portfolio', label: 'Portfolio', icon: BriefcaseBusiness },
  { to: '/lab', label: 'Studio', icon: FlaskConical },
  { to: '/status', label: 'Status', icon: ShieldCheck },
]

export default function Navbar({ theme, onToggleTheme }: Props) {
  const { publicKey, walletName } = useWallet()
  const refreshParcels = useAppStore((s) => s.refreshParcels)

  useEffect(() => {
    if (publicKey) refreshParcels()
  }, [publicKey, refreshParcels])

  return (
    <header className="app-header">
      <NavLink to="/" className="brand-lockup" aria-label="Terra home">
        <span className="brand-mark"><Globe2 size={19} strokeWidth={1.8} /></span>
        <span className="brand-name">terra<span className="brand-period">.</span></span>
      </NavLink>

      <nav className="primary-nav" aria-label="Main navigation">
        {navigation.map(({ to, label, icon: Icon, end }) => (
          <NavLink
            key={to}
            to={to}
            end={end}
            className={({ isActive }) => `primary-nav-link${isActive ? ' active' : ''}`}
          >
            <Icon size={15} strokeWidth={1.8} />
            {label}
          </NavLink>
        ))}
      </nav>

      <div className="header-actions">
        <span className="network-indicator"><span /> Solana devnet</span>
        <button
          className="theme-toggle"
          onClick={onToggleTheme}
          aria-label={`Switch to ${theme === 'dark' ? 'light' : 'dark'} theme`}
          title={`Switch to ${theme === 'dark' ? 'light' : 'dark'} theme`}
        >
          {theme === 'dark' ? <Sun size={16} /> : <Moon size={16} />}
        </button>
        {publicKey ? (
          <div className="wallet-connected">
            <span className="wallet-connected-icon"><ShieldCheck size={15} /></span>
            <span className="wallet-connected-copy">
              <span className="wallet-connected-name">{walletName || 'Wallet connected'}</span>
              <span className="wallet-connected-address">
                {publicKey.toBase58().slice(0, 4)}…{publicKey.toBase58().slice(-4)}
              </span>
            </span>
            <WalletDisconnectButton className="wallet-disconnect" />
          </div>
        ) : (
          <WalletMultiButton className="wallet-connect">
            Connect wallet <ArrowUpRight size={14} />
          </WalletMultiButton>
        )}
      </div>
    </header>
  )
}
