// Terra Atlas — investor-facing planetary observation surface.
// Frontend-only enhancement of the /atlas route: renders the existing
// GlobePage map as the map background, overlays the violet gradient mesh,
// glass window system (Asset Inspector, Wallet/Rights, Layers, Provenance,
// Tour), and a guided investor tour.
//
// Everything here is frontend-only. No terra-core changes, no backend
// dependency — works with the API unreachable (demo data fallback).
// No mandatory wallet, no transaction signing.
import { Suspense, lazy, useCallback, useEffect, useState } from 'react'
import { Sparkles, Wallet, Layers, Info, Compass, Minimize2 } from 'lucide-react'
import GlobePage from '../../pages/GlobePage'
import { useAtlasStore, type AtlasWindowId, DEFAULT_RECTS, type AtlasWindowRect } from '../../lib/atlasStore'
import { useLocale, type TranslationKey } from '../../lib/locale'

// Lazy-loaded heavy windows (kept out of the initial bundle per R-012).
const AssetInspector = lazy(() => import('./AssetInspector'))
const WalletPanel = lazy(() => import('./WalletPanel'))
const LayersPanel = lazy(() => import('./LayersPanel'))
const ProvenancePanel = lazy(() => import('./ProvenancePanel'))
const InvestorTour = lazy(() => import('./InvestorTour'))

const WINDOW_DEFS: Record<AtlasWindowId, { title: TranslationKey; icon: React.FC<{ size?: number; className?: string }> }> = {
  'asset-inspector': { title: 'atlasWindowAssetInspector', icon: Sparkles },
  'wallet-rights': { title: 'atlasWindowWalletRights', icon: Wallet },
  layers: { title: 'atlasWindowLayers', icon: Layers },
  provenance: { title: 'atlasWindowProvenance', icon: Info },
  tour: { title: 'atlasWindowTour', icon: Sparkles },
}

export default function AtlasPage() {
  const { t } = useLocale()

  const openWindows = useAtlasStore((s) => s.openWindows)
  const focusedWindow = useAtlasStore((s) => s.focusedWindow)
  const toggleWindow = useAtlasStore((s) => s.toggleWindow)
  const focusWindow = useAtlasStore((s) => s.focusWindow)

  // Gradient mesh sits behind the globe chrome.
  useEffect(() => {
    document.body.classList.add('atlas-immersive')
    return () => document.body.classList.remove('atlas-immersive')
  }, [])

  return (
    <div className="atlas-page">
      {/* Ambient gradient mesh behind the globe chrome */}
      <div className="atlas-gradient-mesh" aria-hidden="true" />

      {/* The existing immersive globe (map background) */}
      <GlobePage />

      {/* Window system container (lazy-loaded windows render here) */}
      <WindowSystem
        openWindows={openWindows}
        focusedWindow={focusedWindow}
        toggleWindow={toggleWindow}
        focusWindow={focusWindow}
        t={t}
      />

      {/* Guided investor tour (lazy-loaded, overlays everything) */}
      {openWindows.has('tour') && (
        <Suspense fallback={null}>
          <InvestorTour />
        </Suspense>
      )}
    </div>
  )
}

// Glass window system: draggable, resizable, focusable windows with
// minimize-to-dock and localStorage-persisted layout.
function WindowSystem({
  openWindows,
  focusedWindow,
  toggleWindow,
  focusWindow,
  t,
}: {
  openWindows: Set<AtlasWindowId>
  focusedWindow: AtlasWindowId | null
  toggleWindow: (id: AtlasWindowId) => void
  focusWindow: (id: AtlasWindowId) => void
  t: (key: TranslationKey) => string
}) {
  return (
    <>
      {/* Minimized dock */}
      <div className="atlas-dock" aria-label="Minimized windows">
        {[...openWindows].map((id) => {
          const def = WINDOW_DEFS[id]
          const Icon = def.icon
          return (
            <button
              key={id}
              type="button"
              className="atlas-dock-item"
              aria-label={t(def.title)}
              onClick={() => {
                toggleWindow(id)
                focusWindow(id)
              }}
            >
              <Icon size={16} />
            </button>
          )
        })}
      </div>

      {/* Windows */}
      {[...openWindows].map((id) => {
        const def = WINDOW_DEFS[id]
        const Icon = def.icon
        const isFocused = focusedWindow === id
        const Component = id === 'tour' ? InvestorTour : id === 'wallet-rights' ? WalletPanel : id === 'layers' ? LayersPanel : id === 'provenance' ? ProvenancePanel : AssetInspector

        return (
          <Suspense key={id} fallback={<AtlasWindowPlaceholder title={t(def.title)} />}>
            <AtlasWindow
              id={id}
              title={t(def.title)}
              icon={Icon}
              isFocused={isFocused}
              toggleWindow={toggleWindow}
              focusWindow={focusWindow}
            >
              <Component />
            </AtlasWindow>
          </Suspense>
        )
      })}
    </>
  )
}

function AtlasWindowPlaceholder({ title }: { title: string }) {
  return (
    <div className="atlas-window-body">
      <div className="atlas-inspector-placeholder">
        <Compass size={24} className="text-muted" />
        <div>{title}</div>
        <div className="atlas-honesty">loading…</div>
      </div>
    </div>
  )
}

// Draggable, resizable, focusable glass window.
function AtlasWindow({
  id,
  title,
  icon: Icon,
  isFocused,
  toggleWindow,
  focusWindow,
  children,
}: {
  id: AtlasWindowId
  title: string
  icon: React.FC<{ size?: number; className?: string }>
  isFocused: boolean
  toggleWindow: (id: AtlasWindowId) => void
  focusWindow: (id: AtlasWindowId) => void
  children: React.ReactNode
}) {
  const rect = useAtlasStore((s) => s.windowRects[id]) ?? DEFAULT_RECTS[id]
  const setWindowRect = useAtlasStore((s) => s.setWindowRect)
  const minimizeWindow = useAtlasStore((s) => s.minimizeWindow)

  const [isDragging, setIsDragging] = useState(false)
  const [isResizing, setIsResizing] = useState(false)
  const [localRect, setLocalRect] = useState<AtlasWindowRect>(rect)

  const onPointerDown = useCallback(
    (e: React.PointerEvent) => {
      e.stopPropagation()
      e.preventDefault()
      focusWindow(id)
      setIsDragging(true)
      const startX = e.clientX
      const startY = e.clientY
      const startXRect = localRect.x
      const startYRect = localRect.y
      const onMove = (ev: PointerEvent) => {
        setLocalRect((prev) => ({ ...prev, x: startXRect + (ev.clientX - startX), y: startYRect + (ev.clientY - startY) }))
      }
      const onUp = () => {
        setIsDragging(false)
        const fullRect: AtlasWindowRect = { ...localRect, x: localRect.x, y: localRect.y }
        setWindowRect(id, fullRect)
        window.removeEventListener('pointermove', onMove)
        window.removeEventListener('pointerup', onUp)
      }
      window.addEventListener('pointermove', onMove)
      window.addEventListener('pointerup', onUp)
    },
    [id, focusWindow, localRect, setWindowRect]
  )

  const onResizeDown = useCallback(
    (e: React.PointerEvent) => {
      e.stopPropagation()
      e.preventDefault()
      setIsResizing(true)
      const startX = e.clientX
      const startY = e.clientY
      const startW = localRect.w
      const startH = localRect.h
      const onMove = (ev: PointerEvent) => {
        setLocalRect((prev) => ({ ...prev, w: Math.max(280, startW + (ev.clientX - startX)), h: Math.max(240, startH + (ev.clientY - startY)) }))
      }
      const onUp = () => {
        setIsResizing(false)
        setWindowRect(id, { ...localRect, w: localRect.w, h: localRect.h } as AtlasWindowRect)
        window.removeEventListener('pointermove', onMove)
        window.removeEventListener('pointerup', onUp)
      }
      window.addEventListener('pointermove', onMove)
      window.addEventListener('pointerup', onUp)
    },
    [id, localRect, setWindowRect]
  )

  if (rect.minimized) return null

  const isMobile = typeof window !== 'undefined' && window.matchMedia('(max-width: 767px)').matches

  const style: React.CSSProperties = isMobile
    ? { position: 'fixed', inset: 'auto 0 0 0', width: '100%', maxHeight: '75dvh', borderRadius: '24px 24px 0 0', border: 'none', borderTop: '1px solid var(--glass-border)' }
    : {
        position: 'absolute',
        left: localRect.x,
        top: localRect.y,
        width: localRect.w,
        height: localRect.h,
      } as React.CSSProperties

  return (
    <div
      className={`atlas-window${isFocused ? ' focused' : ''}${isDragging || isResizing ? ' pointer-events-none' : ''}`}
      style={style}
      aria-label={title}
    >
      <div
        className="atlas-window-header"
        onPointerDown={onPointerDown}
        role="heading"
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <Icon size={14} className="text-muted" />
          <span>{title}</span>
        </div>
        <div className="atlas-window-header-actions">
          <button
            type="button"
            className="atlas-window-btn"
            aria-label="Minimize"
            title="Minimize"
            onClick={() => {
              toggleWindow(id)
              minimizeWindow(id)
            }}
          >
            <Minimize2 size={12} />
          </button>
          <button
            type="button"
            className="atlas-window-btn close"
            aria-label="Close"
            title="Close"
            onClick={() => toggleWindow(id)}
          >
            <span style={{ fontSize: 16, lineHeight: 1 }}>&times;</span>
          </button>
        </div>
      </div>
      <div className="atlas-window-body">
        {isMobile ? (
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, paddingBottom: 8, borderBottom: '1px solid var(--glass-border)', marginBottom: 8, fontSize: 11, color: 'var(--muted)', fontFamily: 'var(--font-mono)' }}>
            <span>mobile sheet — swipe down to close</span>
          </div>
        ) : null}
        {children}
        <div className="atlas-window resize-handle" onPointerDown={onResizeDown} aria-hidden="true" />
      </div>
    </div>
  )
}

// Honesty label helper used by all atlas surfaces.
export function AtlasHonestyLabel({ mode = 'simulation' }: { mode?: 'simulation' | 'local' | 'prototype' | 'vision' }) {
  const labels: Record<string, string> = {
    simulation: 'simulation',
    local: 'local',
    prototype: 'prototype',
    vision: 'vision',
  }
  return <span className="atlas-honesty">{labels[mode]}</span>
}

// Re-export default export name for the lazy import in App.tsx if needed.
// The route points to this component; GlobePage renders inside it.
