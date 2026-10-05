import { useMemo, useState } from 'react'
import { useAppStore, type OnChainParcelItem } from '../../store/appStore'
import type { OffChainParcel } from '../../lib/api'
import { PARCEL_STATUS } from '../../lib/constants'
import { Loader2, RefreshCw, Search, X } from 'lucide-react'

interface Props {
  onSelect: (p: OnChainParcelItem) => void
  onSelectOffChain: (p: OffChainParcel) => void
}

export default function ParcelListPanel({ onSelect, onSelectOffChain }: Props) {
  const [query, setQuery] = useState('')
  const parcels = useAppStore((s) => s.parcels)
  const loading = useAppStore((s) => s.loadingParcels)
  const error = useAppStore((s) => s.parcelsError)
  const refresh = useAppStore((s) => s.refreshParcels)
  const selected = useAppStore((s) => s.selectedParcel)
  const offChainParcels = useAppStore((s) => s.offChainParcels)
  const demoMode = useAppStore((s) => s.demoMode)
  const selectedOff = useAppStore((s) => s.selectedOffChain)
  const normalizedQuery = query.trim().toLocaleLowerCase()
  const filteredParcels = useMemo(
    () =>
      parcels.filter((p) =>
        `${p.account.name} ${p.holder}`.toLocaleLowerCase().includes(normalizedQuery),
      ),
    [parcels, normalizedQuery],
  )
  const filteredOffChainParcels = useMemo(
    () =>
      offChainParcels.filter((p) =>
        `${p.name} ${p.holder} ${p.status}`.toLocaleLowerCase().includes(normalizedQuery),
      ),
    [offChainParcels, normalizedQuery],
  )

  if (loading) {
    return (
      <div className="p-3 flex justify-center py-6">
        <Loader2 size={18} className="animate-spin text-muted" />
      </div>
    )
  }

  if (parcels.length === 0 && offChainParcels.length === 0) {
    return (
      <div className="p-3 text-center">
        <p className="text-muted text-[12px] mb-2">
          {error ? error : 'No parcels registered yet.'}
        </p>
        <button className="btn btn-secondary w-full justify-center" onClick={refresh}>
          <RefreshCw size={14} /> Refresh
        </button>
      </div>
    )
  }

  return (
    <div className="parcel-list">
      <label className="parcel-search">
        <Search size={15} />
        <input
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder="Search parcels or owners"
          aria-label="Search parcels or owners"
        />
        {query && (
          <button type="button" onClick={() => setQuery('')} aria-label="Clear parcel search">
            <X size={14} />
          </button>
        )}
      </label>

      {filteredParcels.length > 0 && (
        <>
          <div className="flex items-center gap-2 px-3 py-1.5">
            <span className="text-[12px] font-semibold flex-1">
              On-chain parcels <span className="text-muted">({filteredParcels.length})</span>
            </span>
            <button className="btn btn-ghost p-1" onClick={refresh} title="Refresh">
              <RefreshCw size={13} />
            </button>
          </div>
          <ul className="max-h-[300px] overflow-y-auto">
            {filteredParcels.map((p) => {
              const active = selected?.address === p.address
              return (
                <li key={p.address}>
                  <button
                    onClick={() => onSelect(p)}
                    className={`w-full text-left px-3 py-2 flex items-center gap-2 hover:bg-bg border-l-2 ${
                      active ? 'border-emerald-500 bg-emerald-50' : 'border-transparent'
                    }`}
                  >
                    <div className="flex-1 min-w-0">
                      <div className="truncate text-[13px]">{p.account.name}</div>
                      <div className="font-mono text-[10px] text-muted truncate">
                        {(p.holder || '?').slice(0, 10)}…
                      </div>
                    </div>
                    <span className="text-[10px] px-1.5 py-0.5 rounded bg-amber-100 text-amber-800 shrink-0">
                      {PARCEL_STATUS[p.account.status] ?? '?'}
                    </span>
                  </button>
                </li>
              )
            })}
          </ul>
        </>
      )}

      {filteredOffChainParcels.length > 0 && (
        <>
          <div className="flex items-center gap-2 px-3 py-1.5 mt-1">
            <span className="text-[12px] font-semibold flex-1">
              Map parcels <span className="text-muted">({filteredOffChainParcels.length})</span>
            </span>
            {demoMode && (
              <span className="text-[10px] px-1.5 py-0.5 rounded bg-indigo-100 text-indigo-800">
                demo data
              </span>
            )}
          </div>
          <ul className="max-h-[300px] overflow-y-auto">
            {filteredOffChainParcels.map((p) => {
              const active = selectedOff?.id === p.id
              return (
                <li key={p.id}>
                  <button
                    onClick={() => onSelectOffChain(p)}
                    className={`w-full text-left px-3 py-2 flex items-center gap-2 hover:bg-bg border-l-2 ${
                      active ? 'border-emerald-500 bg-emerald-50' : 'border-transparent'
                    }`}
                  >
                    <div className="flex-1 min-w-0">
                      <div className="truncate text-[13px]">{p.name}</div>
                      <div className="font-mono text-[10px] text-muted truncate">
                        {p.area_m2 != null ? `${p.area_m2.toFixed(0)} m²` : 'no area'} ·{' '}
                        {p.holder.slice(0, 12)}…
                      </div>
                    </div>
                    <span className="text-[10px] px-1.5 py-0.5 rounded bg-indigo-100 text-indigo-800 shrink-0 capitalize">
                      {p.status}
                    </span>
                  </button>
                </li>
              )
            })}
          </ul>
        </>
      )}
      {filteredParcels.length === 0 && filteredOffChainParcels.length === 0 && (
        <p className="parcel-search-empty">
          {normalizedQuery ? `No parcels match “${query.trim()}”.` : 'No parcels to display.'}
        </p>
      )}
    </div>
  )
}
