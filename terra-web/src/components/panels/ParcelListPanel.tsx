import { useAppStore, type OnChainParcelItem } from '../../store/appStore'
import { PARCEL_STATUS } from '../../lib/constants'
import { Loader2, RefreshCw } from 'lucide-react'

interface Props {
  onSelect: (p: OnChainParcelItem) => void
}

export default function ParcelListPanel({ onSelect }: Props) {
  const parcels = useAppStore((s) => s.parcels)
  const loading = useAppStore((s) => s.loadingParcels)
  const error = useAppStore((s) => s.parcelsError)
  const refresh = useAppStore((s) => s.refreshParcels)
  const selected = useAppStore((s) => s.selectedParcel)

  if (loading) {
    return (
      <div className="p-3 flex justify-center py-6">
        <Loader2 size={18} className="animate-spin text-muted" />
      </div>
    )
  }

  if (parcels.length === 0) {
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
    <div className="py-1">
      <div className="flex items-center gap-2 px-3 py-1.5">
        <span className="text-[12px] font-semibold flex-1">On-chain parcels</span>
        <button className="btn btn-ghost p-1" onClick={refresh} title="Refresh">
          <RefreshCw size={13} />
        </button>
      </div>
      <ul className="max-h-[300px] overflow-y-auto">
        {parcels.map((p) => {
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
                    {p.account.owner.toBase58().slice(0, 10)}…
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
    </div>
  )
}
