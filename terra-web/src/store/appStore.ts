import { create } from 'zustand'
import { api, type OffChainParcel, type GeoStats, type FusionStats } from '../lib/api'
import { bytesToHex } from '../lib/codec'
import type { ParcelAccount } from '../lib/program'

export interface OnChainParcelItem {
  address: string
  id: string // hex of the 32-byte parcel id
  account: ParcelAccount
}

interface AppState {
  // On-chain parcels (decoded from program accounts via the Anchor client).
  parcels: OnChainParcelItem[]
  loadingParcels: boolean
  parcelsError: string | null
  refreshParcels: () => Promise<void>

  // Off-chain parcels from PostGIS (carry real geometry).
  offChainParcels: OffChainParcel[]
  loadingOffChain: boolean
  offChainError: string | null
  refreshOffChain: () => Promise<void>

  geoStats: GeoStats | null
  fusionStats: FusionStats | null
  loadStats: () => Promise<void>

  selectedParcel: OnChainParcelItem | null
  selectParcel: (p: OnChainParcelItem | null) => void

  lastSignature: string | null
  setLastSignature: (sig: string | null) => void
}

export const useAppStore = create<AppState>((set) => ({
  parcels: [],
  loadingParcels: false,
  parcelsError: null,
  refreshParcels: async () => {
    set({ loadingParcels: true, parcelsError: null })
    try {
      // Dynamic import keeps the Anchor client out of the initial bundle and
      // avoids a hard crash when the wallet is not yet connected.
      const { getProgram } = await import('../lib/program')
      const program = getProgram()
      const accounts = await program.account.parcel.all()
      const items: OnChainParcelItem[] = accounts.map((a) => ({
        address: a.publicKey.toBase58(),
        id: bytesToHex(a.account.id),
        account: a.account,
      }))
      set({ parcels: items, loadingParcels: false })
    } catch (err) {
      set({
        loadingParcels: false,
        parcelsError: err instanceof Error ? err.message : 'Failed to load on-chain parcels',
      })
    }
  },

  offChainParcels: [],
  loadingOffChain: false,
  offChainError: null,
  refreshOffChain: async () => {
    set({ loadingOffChain: true, offChainError: null })
    try {
      const list = await api.listParcels()
      set({ offChainParcels: list, loadingOffChain: false })
    } catch (err) {
      set({
        loadingOffChain: false,
        offChainError: err instanceof Error ? err.message : 'Failed to load off-chain parcels',
      })
    }
  },

  geoStats: null,
  fusionStats: null,
  loadStats: async () => {
    const [geoStats, fusionStats] = await Promise.allSettled([api.geoStats(), api.fusionStats()])
    set({
      geoStats: geoStats.status === 'fulfilled' ? geoStats.value : null,
      fusionStats: fusionStats.status === 'fulfilled' ? fusionStats.value : null,
    })
  },

  selectedParcel: null,
  selectParcel: (p) => set({ selectedParcel: p }),

  lastSignature: null,
  setLastSignature: (sig) => set({ lastSignature: sig }),
}))
