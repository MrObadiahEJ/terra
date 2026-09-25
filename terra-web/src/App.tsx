import { useEffect } from 'react'
import { BrowserRouter, Route, Routes } from 'react-router-dom'
import Navbar from './components/layout/Navbar'
import GlobePage from './pages/GlobePage'
import ProgressPage from './pages/ProgressPage'
import { useWallet } from './lib/wallet'
import { useAppStore } from './store/appStore'

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
        <Navbar />
        <Routes>
          <Route path="/" element={<GlobePage />} />
          <Route path="/progress" element={<ProgressPage />} />
        </Routes>
      </div>
    </BrowserRouter>
  )
}

export default App
