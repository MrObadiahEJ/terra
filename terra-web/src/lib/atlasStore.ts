// Atlas window system, wallet, and tour state.
// Persisted window layout lives in localStorage; nothing is ever written to
// the on-chain program or any backend. All values here are demo/simulated
// unless a real wallet is connected (read-only).
import { create } from 'zustand'
import { persist } from 'zustand/middleware'

export type AtlasWindowId =
  | 'asset-inspector'
  | 'wallet-rights'
  | 'layers'
  | 'provenance'
  | 'tour'

export interface AtlasWindowRect {
  x: number
  y: number
  w: number
  h: number
  minimized: boolean
}

interface AtlasState {
  // Windows
  openWindows: Set<AtlasWindowId>
  focusedWindow: AtlasWindowId | null
  windowRects: Record<AtlasWindowId, AtlasWindowRect>
  dockedWindows: Set<AtlasWindowId>
  toggleWindow: (id: AtlasWindowId) => void
  focusWindow: (id: AtlasWindowId) => void
  closeWindow: (id: AtlasWindowId) => void
  minimizeWindow: (id: AtlasWindowId) => void
  restoreWindow: (id: AtlasWindowId) => void
  setWindowRect: (id: AtlasWindowId, rect: Partial<AtlasWindowRect>) => void

  // Wallet
  connected: boolean
  address: string | null
  balance: string | null // devnet SOL, display string
  setWallet: (a: string | null, bal: string | null) => void
  disconnectWallet: () => void

  // Tour
  tourActive: boolean
  tourStep: number // 0..5
  startTour: () => void
  nextTourStep: () => void
  endTour: () => void

  // Asset inspector selection
  selectedAssetId: string | null
  selectedAssetType: 'parcel' | 'building' | null
  selectAsset: (id: string | null, type: 'parcel' | 'building' | null) => void
}

export const DEFAULT_RECTS: Record<AtlasWindowId, AtlasWindowRect> = {
  'asset-inspector': { x: 560, y: 140, w: 420, h: 520, minimized: false },
  'wallet-rights': { x: 560, y: 680, w: 340, h: 420, minimized: false },
  layers: { x: 20, y: 140, w: 300, h: 420, minimized: false },
  provenance: { x: 340, y: 140, w: 300, h: 420, minimized: false },
  tour: { x: 560, y: 680, w: 420, h: 360, minimized: false },
}

export const useAtlasStore = create<AtlasState>()(
  persist(
    (set, get) => ({
      openWindows: new Set(),
      focusedWindow: null,
      windowRects: DEFAULT_RECTS,
      dockedWindows: new Set(),

      toggleWindow: (id) => {
        const open = new Set(get().openWindows)
        if (open.has(id)) open.delete(id)
        else {
          open.add(id)
          set({ focusedWindow: id })
        }
        set({ openWindows: open })
      },
      focusWindow: (id) => set({ focusedWindow: id }),
      closeWindow: (id) => {
        const open = new Set(get().openWindows)
        open.delete(id)
        set({ openWindows: open, focusedWindow: get().focusedWindow === id ? null : get().focusedWindow })
      },
      minimizeWindow: (id) => {
        const docked = new Set(get().dockedWindows)
        docked.add(id)
        set({ dockedWindows: docked })
      },
      restoreWindow: (id) => {
        const docked = new Set(get().dockedWindows)
        docked.delete(id)
        set({ dockedWindows: docked })
      },
      setWindowRect: (id, rect) =>
        set({ windowRects: { ...get().windowRects, [id]: { ...get().windowRects[id], ...rect } } }),

      connected: false,
      address: null,
      balance: null,
      setWallet: (address, balance) =>
        set({ connected: !!address, address, balance }),
      disconnectWallet: () => set({ connected: false, address: null, balance: null }),

      tourActive: false,
      tourStep: 0,
      startTour: () => set({ tourActive: true, tourStep: 0 }),
      nextTourStep: () =>
        set((s) => ({ tourStep: s.tourStep + 1 })),
      endTour: () => set({ tourActive: false, tourStep: 0 }),

      selectedAssetId: null,
      selectedAssetType: null,
      selectAsset: (id, type) => set({ selectedAssetId: id, selectedAssetType: type }),
    }),
    {
      name: 'terra.atlas',
      partialize: (s) => ({ windowRects: s.windowRects, dockedWindows: Array.from(s.dockedWindows) }),
      // Hydrate sets back into Set types
      merge: (persistedState: unknown, currentState: AtlasState) => {
        const ps = persistedState as Partial<AtlasState> & { openWindows?: AtlasWindowId[]; dockedWindows?: AtlasWindowId[] }
        const openWindows = new Set((ps.openWindows as AtlasWindowId[]) ?? [])
        const dockedWindows = new Set((ps.dockedWindows as AtlasWindowId[]) ?? [])
        return {
          ...currentState,
          ...ps,
          openWindows,
          dockedWindows,
        }
      },
    },
  ),
)
