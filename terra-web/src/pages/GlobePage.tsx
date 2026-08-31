import { useEffect, useMemo, useState } from 'react'
import TerraGlobe, { type DrawVertex } from '../components/map/TerraGlobe'
import RegisterParcelPanel from '../components/panels/RegisterParcelPanel'
import ParcelListPanel from '../components/panels/ParcelListPanel'
import ParcelPanel from '../components/panels/ParcelPanel'
import { useAppStore, type OnChainParcelItem } from '../store/appStore'
import { api, type RoadRow, type PoiRow } from '../lib/api'
import { ChevronDown, ChevronUp } from 'lucide-react'

export default function GlobePage() {
  const {
    offChainParcels,
    refreshOffChain,
    geoStats,
    fusionStats,
    loadStats,
    selectedParcel,
    selectParcel,
  } = useAppStore()

  const [drawing, setDrawing] = useState(false)
  const [drawVertices, setDrawVertices] = useState<DrawVertex[]>([])
  const [roads, setRoads] = useState<RoadRow[]>([])
  const [pois, setPois] = useState<PoiRow[]>([])
  const [tab, setTab] = useState<'register' | 'browse'>('browse')

  // Initial load of off-chain data + stats.
  useEffect(() => {
    refreshOffChain()
    loadStats()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  useEffect(() => {
    let cancelled = false
    api
      .roads()
      .then((r) => !cancelled && setRoads(r))
      .catch(() => {})
    api
      .poisFusion()
      .then((p) => !cancelled && setPois(p))
      .catch(() => {})
    return () => {
      cancelled = true
    }
  }, [])

  const selectedSummary = useMemo(() => {
    if (!selectedParcel) return null
    const off = offChainParcels.find((p) => p.owner === selectedParcel.account.owner.toBase58())
    return { onchain: selectedParcel, off }
  }, [selectedParcel, offChainParcels])

  const onSelectParcel = (p: OnChainParcelItem) => selectParcel(p)

  return (
    <div className="flex h-[calc(100vh-3.5rem)]">
      {/* 3D globe */}
      <main className="flex-1 relative">
        <TerraGlobe
          offChainParcels={offChainParcels}
          roads={roads}
          pois={pois}
          drawing={drawing}
          drawVertices={drawVertices}
          onDrawVertexAdd={(v) => setDrawVertices((vs) => [...vs, v])}
          onParcelClick={(id) => {
            const off = offChainParcels.find((p) => p.id === id)
            if (off) {
              // Link the off-chain geometry record to its on-chain ownership by
              // matching on owner + name (both are written at registration time).
              const onchain = useAppStore
                .getState()
                .parcels.find(
                  (p) =>
                    p.account.owner.toBase58() === off.owner &&
                    p.account.name === off.name,
                )
              if (onchain) selectParcel(onchain)
            }
          }}
        />

        {/* stats overlay */}
        {(geoStats?.roads || fusionStats?.roads) && (
          <div className="absolute top-3 left-3 bg-surface/90 backdrop-blur rounded-lg shadow px-3 py-2 text-[11px] pointer-events-none">
            <div className="flex gap-3">
              <span>🏛️ Roads: <b>{fusionStats?.roads ?? geoStats?.roads ?? 0}</b></span>
              <span>📍 POIs: <b>{fusionStats?.pois ?? geoStats?.pois ?? 0}</b></span>
              <span>📏 {geoStats?.road_length_km ? `${geoStats.road_length_km.toFixed(0)} km` : 'OSM off'}</span>
            </div>
          </div>
        )}
      </main>

      {/* Sidebar */}
      <aside className="w-[360px] border-l bg-surface flex flex-col shrink-0">
        <div className="flex border-b">
          {(['register', 'browse'] as const).map((t) => (
            <button
              key={t}
              className={`flex-1 py-2 text-[13px] font-medium capitalize hover:bg-bg ${
                tab === t ? 'border-b-2 border-emerald-500' : 'text-muted'
              }`}
              onClick={() => setTab(t)}
            >
              {t === 'register' ? 'Register' : 'Browse'}
            </button>
          ))}
        </div>

        <div className="flex-1 overflow-y-auto">
          {tab === 'register' ? (
            <RegisterParcelPanel
              drawing={drawing}
              drawVertices={drawVertices}
              onToggleDrawing={() => setDrawing((d) => !d)}
              onClearDrawing={() => setDrawVertices([])}
            />
          ) : selectedParcel ? (
            <>
              <div className="flex items-center justify-between px-3 pt-2">
                <span className="text-[12px] text-muted">Selected parcel</span>
                <button
                  className="btn btn-ghost p-1"
                  onClick={() => {
                    selectParcel(null)
                    setTab('browse')
                  }}
                  title="Close"
                >
                  ✕
                </button>
              </div>
              <ParcelPanel address={selectedParcel.address} account={selectedParcel.account} />
              {selectedSummary?.off && (
                <div className="px-3 pb-3 text-[12px] text-muted">
                  Off-chain record: <b>{selectedSummary.off.status}</b> ·{' '}
                  {selectedSummary.off.area_m2 != null
                    ? `${selectedSummary.off.area_m2.toFixed(0)} m²`
                    : 'no area'}
                </div>
              )}
            </>
          ) : (
            <ParcelListPanel onSelect={onSelectParcel} />
          )}
        </div>

        {/* collapsible mini stats footer */}
        <DetailsFooter geoStats={geoStats} fusionStats={fusionStats} />
      </aside>
    </div>
  )
}

function DetailsFooter({
  geoStats,
  fusionStats,
}: {
  geoStats: ReturnType<typeof useAppStore.getState>['geoStats']
  fusionStats: ReturnType<typeof useAppStore.getState>['fusionStats']
}) {
  const [open, setOpen] = useState(false)
  const rows: [string, string][] = []
  if (geoStats?.nodes != null) rows.push(['OSM nodes', String(geoStats.nodes)])
  if (geoStats?.roads != null) rows.push(['OSM roads', String(geoStats.roads)])
  if (geoStats?.road_length_km != null) rows.push(['Road length', `${geoStats.road_length_km.toFixed(1)} km`])
  if (geoStats?.pois != null) rows.push(['OSM POIs', String(geoStats.pois)])
  if (fusionStats?.roads != null) rows.push(['DB roads', String(fusionStats.roads)])
  if (fusionStats?.pois != null) rows.push(['DB POIs', String(fusionStats.pois)])
  if (fusionStats?.pilot_zones != null) rows.push(['Pilot zones', String(fusionStats.pilot_zones)])
  if (rows.length === 0) rows.push(['Data not loaded', '—'])

  return (
    <div className="border-t">
      <button
        className="w-full flex items-center justify-between px-3 py-2 text-[12px] font-medium hover:bg-bg"
        onClick={() => setOpen((o) => !o)}
      >
        <span>Geo / data stats</span>
        {open ? <ChevronDown size={14} /> : <ChevronUp size={14} />}
      </button>
      {open && (
        <dl className="px-3 pb-2 grid grid-cols-2 gap-x-4 gap-y-0.5 text-[11px]">
          {rows.map(([k, v]) => (
            <div key={k} className="flex justify-between">
              <dt className="text-muted">{k}</dt>
              <dd className="font-medium">{v}</dd>
            </div>
          ))}
        </dl>
      )}
    </div>
  )
}
