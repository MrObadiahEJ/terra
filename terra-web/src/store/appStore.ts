import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import { api, type OffChainParcel, type GeoStats, type FusionStats } from '../lib/api'
import { DEMO_PARCELS, DEMO_GEO_STATS, DEMO_FUSION_STATS } from '../lib/demoData'
import { bytesToHex } from '../lib/codec'
import type { ParcelAccount, RightsAccount } from '../lib/program'

export interface OnChainParcelItem {
  address: string
  id: string // hex of the 32-byte parcel id
  account: ParcelAccount
  holder: string // base58 wallet (or identity) from the canonical ownership Rights PDA
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

  // Demo layer: true when the bundle's sample data is shown (API down or DB
  // empty). localParcels are demo registrations made without a wallet/API
  // (persisted in localStorage so they survive a refresh).
  demoMode: boolean
  localParcels: OffChainParcel[]
  addLocalParcel: (p: OffChainParcel) => void

  geoStats: GeoStats | null
  fusionStats: FusionStats | null
  loadStats: () => Promise<void>

  selectedParcel: OnChainParcelItem | null
  selectParcel: (p: OnChainParcelItem | null) => void

  // Off-chain-only selection (demo/local/DB parcels with no on-chain match).
  selectedOffChain: OffChainParcel | null
  selectOffChain: (p: OffChainParcel | null) => void

  lastSignature: string | null
  setLastSignature: (sig: string | null) => void
}

/** First list wins on duplicate ids (locals are prepended, so they stay on top). */
function mergeParcels(locals: OffChainParcel[], rest: OffChainParcel[]): OffChainParcel[] {
  const seen = new Set<string>()
  const out: OffChainParcel[] = []
  for (const p of [...locals, ...rest]) {
    if (seen.has(p.id)) continue
    seen.add(p.id)
    out.push(p)
  }
  return out
}

export const useAppStore = create<AppState>()(
  persist(
    (set, get) => ({
      parcels: [],
      loadingParcels: false,
      parcelsError: null,
      refreshParcels: async () => {
        set({ loadingParcels: true, parcelsError: null })
        try {
          // Dynamic import keeps the Anchor client out of the initial bundle and
          // avoids a hard crash when the wallet is not yet connected.
          const { getProgram } = await import('../lib/program')
          const program = getProgram() as unknown as {
            account: {
              parcel: { all(): Promise<{ publicKey: { toBase58(): string }; account: ParcelAccount }[]> }
              rights: { all(): Promise<{ account: RightsAccount }[]> }
            }
          }
          const [accounts, rights] = await Promise.all([
            program.account.parcel.all(),
            program.account.rights.all(),
          ])
          // Ownership is the Rights PDA with rightsKind === 0 (OWNERSHIP); it is
          // the single source of truth for who holds a parcel.
          const holders = new Map<string, string>()
          for (const r of rights) {
            if (r.account.rightsKind === 0) {
              holders.set(r.account.parcel.toBase58(), r.account.holder.toBase58())
            }
          }
          const items: OnChainParcelItem[] = accounts.map((a) => ({
            address: a.publicKey.toBase58(),
            id: bytesToHex(a.account.id),
            account: a.account,
            holder: holders.get(a.publicKey.toBase58()) ?? '',
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
        const locals = get().localParcels
        try {
          const list = await api.listParcels()
          if (list.length === 0) {
            // Fresh/empty database — seed the bundled demo so a bare install
            // is still presentable.
            set({
              offChainParcels: mergeParcels(locals, DEMO_PARCELS),
              demoMode: true,
              loadingOffChain: false,
            })
          } else {
            set({
              offChainParcels: mergeParcels(locals, list),
              demoMode: false,
              loadingOffChain: false,
            })
          }
        } catch (err) {
          set({
            offChainParcels: mergeParcels(locals, DEMO_PARCELS),
            demoMode: true,
            loadingOffChain: false,
            offChainError: err instanceof Error ? err.message : 'Failed to load off-chain parcels',
          })
        }
      },

      demoMode: false,
      localParcels: [],
      addLocalParcel: (p) => {
        const locals = [p, ...get().localParcels.filter((x) => x.id !== p.id)]
        set({
          localParcels: locals,
          offChainParcels: mergeParcels(locals, get().offChainParcels.filter((x) => x.id !== p.id)),
        })
      },

      geoStats: null,
      fusionStats: null,
      loadStats: async () => {
        const [geoStats, fusionStats] = await Promise.allSettled([api.geoStats(), api.fusionStats()])
        set({
          geoStats: geoStats.status === 'fulfilled' ? geoStats.value : DEMO_GEO_STATS,
          fusionStats: fusionStats.status === 'fulfilled' ? fusionStats.value : DEMO_FUSION_STATS,
        })
      },

      selectedParcel: null,
      selectParcel: (p) => set({ selectedParcel: p, ...(p ? { selectedOffChain: null } : {}) }),

      selectedOffChain: null,
      selectOffChain: (p) => set({ selectedOffChain: p, ...(p ? { selectedParcel: null } : {}) }),

      lastSignature: null,
      setLastSignature: (sig) => set({ lastSignature: sig }),
    }),
    {
      name: 'terra.demo',
      // Only the user-created local parcels are persisted; everything else is
      // runtime state.
      partialize: (s) => ({ localParcels: s.localParcels }),
    },
  ),
)
