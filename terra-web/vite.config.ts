import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { copyFileSync, mkdirSync, readdirSync, statSync, existsSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

// --- Cesium static asset handling -----------------------------------------
// CesiumJS must be served its Workers/Assets/ThirdParty/Widgets from a base
// URL. We copy them from the installed package into public/cesium (gitignored)
// so both `vite dev` and `vite build` can resolve them via CESIUM_BASE_URL.
const root = dirname(fileURLToPath(import.meta.url))
const cesiumPkg = join(root, 'node_modules', 'cesium', 'Build', 'Cesium')
const publicCesium = join(root, 'public', 'cesium')

function copyDir(src: string, dest: string) {
  if (!existsSync(src)) return
  for (const entry of readdirSync(src)) {
    const s = join(src, entry)
    const d = join(dest, entry)
    if (statSync(s).isDirectory()) copyDir(s, d)
    else {
      mkdirSync(dirname(d), { recursive: true })
      copyFileSync(s, d)
    }
  }
}

function cesiumStaticPlugin(): { name: string; buildStart(): void } {
  return {
    name: 'terra:cesium-static',
    buildStart() {
      for (const dir of ['Workers', 'Assets', 'ThirdParty', 'Widgets']) {
        copyDir(join(cesiumPkg, dir), join(publicCesium, dir))
      }
    },
  }
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), cesiumStaticPlugin()],
  build: {
    target: 'es2022',
    chunkSizeWarningLimit: 5000,
  },
  server: {
    port: 5173,
    proxy: {
      // Terra off-chain API (Axum). Adjust target if your backend runs elsewhere.
      '/api': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
    },
  },
})
