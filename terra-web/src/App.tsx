import { useEffect } from 'react'
import { BrowserRouter, Route, Routes, useLocation } from 'react-router-dom'
import Navbar from './components/layout/Navbar'
import GlobePage from './pages/GlobePage'
import ProgressPage from './pages/ProgressPage'
import LabPage from './pages/LabPage'
import TransactionsPage from './pages/TransactionsPage'
import Toasts from './components/Toasts'
import { useWallet } from './lib/wallet'
import { useAppStore } from './store/appStore'

const TITLES: Record<string, string> = {
  '/': 'Terra — Geospatial trust platform',
  '/progress': 'Terra — Progress',
  '/lab': 'Terra Lab — Interactive demos',
  '/transactions': 'Terra — Transactions',
}

function DocumentTitle() {
  const { pathname } = useLocation()
  useEffect(() => {
    document.title = TITLES[pathname] ?? 'Terra'
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

  return (
    <BrowserRouter>
      <div className="flex flex-col h-screen bg-bg text-ink">
        <DocumentTitle />
        <Navbar />
        <Routes>
          <Route path="/" element={<GlobePage />} />
          <Route path="/progress" element={<ProgressPage />} />
          <Route path="/lab" element={<LabPage />} />
          <Route path="/transactions" element={<TransactionsPage />} />
        </Routes>
        <Toasts />
      </div>
    </BrowserRouter>
  )
}

export default App
