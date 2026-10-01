import { useEffect, useState } from 'react'
import { BrowserRouter, Navigate, Route, Routes, useLocation } from 'react-router-dom'
import Navbar from './components/layout/Navbar'
import DemoPage from './pages/DemoPage'
import GlobePage from './pages/GlobePage'
import LabPage from './pages/LabPage'
import LandDetailPage from './pages/LandDetailPage'
import NetworkPage from './pages/NetworkPage'
import PortfolioPage from './pages/PortfolioPage'
import StatusPage from './pages/StatusPage'
import TransactionsPage from './pages/TransactionsPage'
import Toasts from './components/Toasts'
import { useWallet } from './lib/wallet'
import { useAppStore } from './store/appStore'

const TITLES: Record<string, string> = {
  '/': 'Terra — Geospatial trust platform',
  '/lab': 'Terra Lab — Interactive demos',
  '/transactions': 'Terra — Transactions',
  '/portfolio': 'Identity & portfolio — Terra',
  '/status': 'Protocol status — Terra',
  '/network': 'Validator network — Terra',
  '/demo': 'Demo scenarios — Terra',
}

function DocumentTitle() {
  const { pathname } = useLocation()
  useEffect(() => {
    document.title =
      TITLES[pathname] ??
      (pathname.startsWith('/lab/land') ? 'Land — Terra' : 'Terra')
  }, [pathname])
  return null
}

// Initialize on-chain parcel store whenever wallet/pubkey changes.
function useSyncParcels() {
  const publicKey = useWallet().publicKey
  const refresh = useAppStore((s) => s.refreshParcels)
  useEffect(() => {
    if (publicKey) refresh()
  }, [publicKey, refresh])
}
function App() {
  useSyncParcels()
  const [theme, setTheme] = useState<'light' | 'dark'>(() =>
    localStorage.getItem('terra-theme') === 'dark' ? 'dark' : 'light',
  )

  useEffect(() => {
    document.documentElement.dataset.theme = theme
    localStorage.setItem('terra-theme', theme)
  }, [theme])

  return (
    <BrowserRouter>
      <div className="flex flex-col h-screen bg-bg text-ink">
        <DocumentTitle />
        <Navbar
          theme={theme}
          onToggleTheme={() => setTheme((current) => current === 'dark' ? 'light' : 'dark')}
        />
        <Routes>
          <Route path="/" element={<GlobePage />} />
          <Route path="/lab/land" element={<LandDetailPage />} />
          <Route path="/lab" element={<LabPage />} />
          <Route path="/demo" element={<DemoPage />} />
          <Route path="/network" element={<NetworkPage />} />
          <Route path="/portfolio" element={<PortfolioPage />} />
          <Route path="/status" element={<StatusPage />} />
          <Route path="/transactions" element={<TransactionsPage />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
        <Toasts />
      </div>
    </BrowserRouter>
  )
}

export default App
