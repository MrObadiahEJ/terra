import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import * as Cesium from 'cesium'
import '@solana/wallet-adapter-react-ui/styles.css'
import './index.css'
import App from './App.tsx'
import { TerraWalletProvider } from './lib/wallet'

// Tell Cesium where its static assets (Workers, Assets, ThirdParty, Widgets)
// live when served from a non-root base path (e.g. GitHub Pages /terra/).
// The vite cesiumStaticPlugin copies them into public/cesium/ at build time.
// Without this, Cesium defaults to /cesium/ and its workers 404 on GitHub Pages,
// triggering _onRenderError / showErrorPanel.
;(Cesium.buildModuleUrl as { setBaseUrl?: (url: string) => void }).setBaseUrl?.(
  `${import.meta.env.BASE_URL}cesium/`,
)

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <TerraWalletProvider>
      <App />
    </TerraWalletProvider>
  </StrictMode>,
)
