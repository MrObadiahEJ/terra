import { lazy, Suspense, useEffect, useState } from 'react'
import { BrowserRouter, Navigate, Route, Routes, useLocation } from 'react-router-dom'
import Navbar from './components/layout/Navbar'
import WelcomePage from './pages/WelcomePage'
import Toasts from './components/Toasts'
import { useWallet } from './lib/wallet'
import { useAppStore } from './store/appStore'
import { LocaleProvider, useLocale, type TranslationKey } from './lib/locale'

const GlobePage = lazy(() => import('./pages/GlobePage'))
const DemoPage = lazy(() => import('./pages/DemoPage'))
const LabPage = lazy(() => import('./pages/LabPage'))
const LandDetailPage = lazy(() => import('./pages/LandDetailPage'))
const NetworkPage = lazy(() => import('./pages/NetworkPage'))
const PortfolioPage = lazy(() => import('./pages/PortfolioPage'))
const StatusPage = lazy(() => import('./pages/StatusPage'))
const TransactionsPage = lazy(() => import('./pages/TransactionsPage'))
const ProtocolConsolePage = lazy(() => import('./pages/ProtocolConsolePage'))

const TITLES: Record<string, TranslationKey> = {
  '/': 'titleHome',
  '/welcome': 'titleWelcome',
  '/atlas': 'titleAtlas',
  '/lab': 'titleLab',
  '/transactions': 'titleTransactions',
  '/portfolio': 'titlePortfolio',
  '/status': 'titleStatus',
  '/network': 'titleNetwork',
  '/demo': 'titleDemo',
  '/console': 'titleConsole',
}

function DocumentTitle() {
  const { pathname } = useLocation()
  const { t } = useLocale()
  useEffect(() => {
    document.title =
      (pathname in TITLES ? t(TITLES[pathname]) : null) ??
      (pathname.startsWith('/lab/land') ? t('titleLand') : t('titleDefault'))
  }, [pathname, t])
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
function RouteLoading() {
  const { t } = useLocale()
  return (
    <div className="route-loading" role="status" aria-label={t('loadingEllipsis')}>
      <span />
    </div>
  )
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
    <LocaleProvider>
      <BrowserRouter>
        <div className="flex flex-col h-screen bg-bg text-ink">
          <DocumentTitle />
          <Navbar
            theme={theme}
            onToggleTheme={() => setTheme((current) => current === 'dark' ? 'light' : 'dark')}
          />
          <Suspense fallback={<RouteLoading />}>
            <Routes>
              <Route path="/" element={<Navigate to="/welcome" replace />} />
              <Route path="/welcome" element={<WelcomePage />} />
              <Route path="/atlas" element={<GlobePage />} />
              <Route path="/lab/land" element={<LandDetailPage />} />
              <Route path="/lab" element={<LabPage />} />
              <Route path="/demo" element={<DemoPage />} />
              <Route path="/network" element={<NetworkPage />} />
              <Route path="/portfolio" element={<PortfolioPage />} />
              <Route path="/status" element={<StatusPage />} />
              <Route path="/transactions" element={<TransactionsPage />} />
              <Route path="/console" element={<ProtocolConsolePage />} />
              <Route path="*" element={<Navigate to="/welcome" replace />} />
            </Routes>
          </Suspense>
          <Toasts />
        </div>
      </BrowserRouter>
    </LocaleProvider>
  )
}

export default App
