import { useEffect } from 'react'
import { NavLink } from 'react-router-dom'
import { WalletMultiButton, WalletDisconnectButton } from '@solana/wallet-adapter-react-ui'
import {
  Activity,
  ArrowUpRight,
  Compass,
  FlaskConical,
  Languages,
  Menu,
  Moon,
  Network,
  BriefcaseBusiness,
  ShieldCheck,
  Sun,
} from 'lucide-react'
import TerraLogo from '../brand/TerraLogo'
import { useWallet } from '../../lib/wallet'
import { useAppStore } from '../../store/appStore'
import { isLocale, useLocale, type TranslationKey } from '../../lib/locale'

interface Props {
  theme: 'light' | 'dark'
  onToggleTheme: () => void
}

const navigation: { to: string; label: TranslationKey; icon: typeof Compass; end?: boolean }[] = [
  { to: '/welcome', label: 'home', icon: Compass, end: true },
  { to: '/atlas', label: 'explore', icon: Compass },
  { to: '/transactions', label: 'activity', icon: Activity },
  { to: '/demo', label: 'demo', icon: ShieldCheck },
  { to: '/network', label: 'network', icon: Network },
  { to: '/portfolio', label: 'portfolio', icon: BriefcaseBusiness },
  { to: '/lab', label: 'studio', icon: FlaskConical },
  { to: '/status', label: 'status', icon: ShieldCheck },
]

export default function Navbar({ theme, onToggleTheme }: Props) {
  const { publicKey, walletName } = useWallet()
  const refreshParcels = useAppStore((s) => s.refreshParcels)
  const { locale, setLocale, t } = useLocale()

  useEffect(() => {
    if (publicKey) refreshParcels()
  }, [publicKey, refreshParcels])

  return (
    <header className="app-header">
      <NavLink to="/welcome" className="brand-lockup" aria-label="Terra home">
        <span className="brand-mark"><TerraLogo size={34} /></span>
        <span className="brand-name">terra<span className="brand-period">.</span></span>
      </NavLink>

      <nav className="primary-nav" aria-label={t('menu')}>
        {navigation.map(({ to, label, icon: Icon, end }) => (
          <NavLink
            key={to}
            to={to}
            end={end}
            className={({ isActive }) => `primary-nav-link${isActive ? ' active' : ''}`}
          >
            <Icon size={15} strokeWidth={1.8} />
            {t(label)}
          </NavLink>
        ))}
      </nav>

      <details className="compact-nav">
        <summary aria-label={t('menu')} title={t('menu')}><Menu size={17} /><span>{t('menu')}</span></summary>
        <nav className="compact-nav-panel" aria-label={t('menu')}>
          {navigation.map(({ to, label, icon: Icon, end }) => (
            <NavLink
              key={to}
              to={to}
              end={end}
              className={({ isActive }) => `compact-nav-link${isActive ? ' active' : ''}`}
              onClick={(event) => event.currentTarget.closest('details')?.removeAttribute('open')}
            >
              <Icon size={15} /> {t(label)}
            </NavLink>
          ))}
        </nav>
      </details>

      <div className="header-actions">
        <span className="network-indicator"><span /> Solana devnet</span>
        <label className="locale-select">
          <Languages size={14} aria-hidden="true" />
          <span className="sr-only">{t('language')}</span>
          <select
            aria-label={t('language')}
            value={locale}
            onChange={(event) => {
              const nextLocale = event.currentTarget.value
              if (isLocale(nextLocale)) setLocale(nextLocale)
            }}
          >
            <option value="en">EN</option>
            <option value="fr">FR</option>
            <option value="ru">RU</option>
            <option value="zh">中文</option>
          </select>
        </label>
        <button
          className="theme-toggle"
          onClick={onToggleTheme}
          aria-label={theme === 'dark' ? t('themeLight') : t('themeDark')}
          title={theme === 'dark' ? t('themeLight') : t('themeDark')}
        >
          {theme === 'dark' ? <Sun size={16} /> : <Moon size={16} />}
        </button>
        {publicKey ? (
          <div className="wallet-connected">
            <span className="wallet-connected-icon"><ShieldCheck size={15} /></span>
            <span className="wallet-connected-copy">
              <span className="wallet-connected-name">{walletName || t('walletConnected')}</span>
              <span className="wallet-connected-address">
                {publicKey.toBase58().slice(0, 4)}…{publicKey.toBase58().slice(-4)}
              </span>
            </span>
            <WalletDisconnectButton className="wallet-disconnect" />
          </div>
        ) : (
          <WalletMultiButton className="wallet-connect">
            {t('connectWallet')} <ArrowUpRight size={14} />
          </WalletMultiButton>
        )}
      </div>
    </header>
  )
}
