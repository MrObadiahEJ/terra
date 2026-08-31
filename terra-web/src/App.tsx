import { useEffect } from 'react'
import Navbar from './components/layout/Navbar'
import GlobePage from './pages/GlobePage'
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
    <div className="flex flex-col h-screen bg-bg text-ink">
      <Navbar />
      <GlobePage />
    </div>
  )
}

export default App
