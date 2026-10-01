import { useEffect } from 'react'
import { NavLink } from 'react-router-dom'
import { WalletMultiButton, WalletDisconnectButton } from '@solana/wallet-adapter-react-ui'
import { useWallet } from '../../lib/wallet'
import { useAppStore } from '../../store/appStore'

export default function Navbar() {
  const { publicKey, walletName } = useWallet()
  const refreshParcels = useAppStore((s) => s.refreshParcels)

  useEffect(() => {
    if (publicKey) refreshParcels()
  }, [publicKey, refreshParcels])

  return (
    <header className="flex flex-wrap items-center gap-3 px-4 min-h-14 border-b bg-surface shrink-0">
      <div className="flex items-center gap-2 font-semibold tracking-tight">
        <span className="w-2.5 h-2.5 rounded-full bg-emerald-500" />
        Terra
      </div>
      <nav className="flex gap-1 ml-2 text-sm">
        <NavLink
          to="/"
          end
          className={({ isActive }) => (isActive ? 'nav-link active' : 'nav-link')}
        >
          Globe
        </NavLink>
        <NavLink
          to="/lab"
          className={({ isActive }) => (isActive ? 'nav-link active' : 'nav-link')}
        >
          Lab
        </NavLink>
        <NavLink
          to="/transactions"
          className={({ isActive }) => (isActive ? 'nav-link active' : 'nav-link')}
        >
          Transactions
        </NavLink>
        <NavLink
          to="/status"
          className={({ isActive }) => (isActive ? 'nav-link active' : 'nav-link')}
        >
          Status
        </NavLink>
      </nav>

      <div className="ml-auto flex items-center gap-2">
        {walletName && (
          <span className="text-[11px] text-muted">{walletName}</span>
        )}
        {publicKey ? (
          <>
            <WalletDisconnectButton className="btn btn-ghost" />
          </>
        ) : (
          <WalletMultiButton className="btn btn-primary" />
        )}
      </div>
    </header>
  )
}
