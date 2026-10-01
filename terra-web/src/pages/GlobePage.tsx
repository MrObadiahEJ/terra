import { useCallback, useEffect, useMemo, useState } from 'react'
import TerraGlobe, { type DrawVertex } from '../components/map/TerraGlobe'
import RegisterParcelPanel from '../components/panels/RegisterParcelPanel'
import ParcelListPanel from '../components/panels/ParcelListPanel'
import ParcelPanel from '../components/panels/ParcelPanel'
import OffChainParcelPanel from '../components/panels/OffChainParcelPanel'
import { useAppStore, type OnChainParcelItem } from '../store/appStore'
import { api, parseGeoJSON, type RoadRow, type PoiRow } from '../lib/api'
import { snapPoint, type LonLat } from '../lib/geo'
import { DEMO_ROADS, DEMO_POIS } from '../lib/demoData'
import { ChevronDown, ChevronUp, Route, MapPin, Square } from 'lucide-react'

export default function GlobePage() {
  const {
    offChainParcels,
    refreshOffChain,
    geoStats,
    fusionStats,
    loadStats,
    selectedParcel,
    selectParcel,
    selectedOffChain,
    selectOffChain,
    demoMode,
  } = useAppStore()

  const [drawing, setDrawing] = useState(false)
  const [webglStatus, setWebglStatus] = useState<string | null>(null)
  const [drawVertices, setDrawVertices] = useState<DrawVertex[]>([])
  const [roads, setRoads] = useState<RoadRow[]>([])
  const [pois, setPois] = useState<PoiRow[]>([])
  const [layersDemo, setLayersDemo] = useState(false)
  const [tab, setTab] = useState<'register' | 'browse'>('browse')
  const [showLayers, setShowLayers] = useState({ parcels: true, roads: true, pois: true })
  const [snapGeom, setSnapGeom] = useState(true)
  const [snapGrid, setSnapGrid] = useState(false)

  // Initial load of off-chain data + stats.
  useEffect(() => {
    refreshOffChain()
    loadStats()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // Roads/POIs with bundled demo fallback (API down or empty database).
  useEffect(() => {
    let cancelled = false
    let demo = false
    api
      .roads()
      .then((r) => {
        if (cancelled) return
        if (r.length === 0) {
          demo = true
          setRoads(DEMO_ROADS)
        } else {
          setRoads(r)
        }
      })
      .catch(() => {
        if (cancelled) return
        demo = true
        setRoads(DEMO_ROADS)
      })
    api
      .poisFusion()
      .then((p) => {
        if (cancelled) return
        if (p.length === 0) {
          demo = true
          setPois(DEMO_POIS)
        } else {
          setPois(p)
        }
        if (demo) setLayersDemo(true)
      })
      .catch(() => {
        if (cancelled) return
        setPois(DEMO_POIS)
        setLayersDemo(true)
      })
    return () => {
      cancelled = true
    }
  }, [])

  const shownParcels = useMemo(
    () => (showLayers.parcels ? offChainParcels : []),
    [showLayers.parcels, offChainParcels],
  )
  const shownRoads = showLayers.roads ? roads : []
  const shownPois = showLayers.pois ? pois : []
  const usingDemo = demoMode || layersDemo

  // Parcel boundaries as snap candidates while drawing.
  const snapRings = useMemo(() => {
    const rings: LonLat[] = []
    for (const p of shownParcels) {
      const poly = parseGeoJSON<{ type: string; coordinates: number[][][] }>(p.geometry)
      if (!poly || poly.type !== 'Polygon') continue
      rings.push(poly.coordinates[0].map(([lon, lat]) => [lon, lat] as LonLat))
    }
    return rings
  }, [shownParcels])

  const finishDrawing = useCallback(() => {
    if (drawVertices.length >= 3) setDrawing(false)
  }, [drawVertices])

  const onVertexAdd = useCallback(
    (raw: DrawVertex) => {
      const current = drawVertices
      const snapped = snapPoint([raw.lon, raw.lat], {
        own: current.map((v) => [v.lon, v.lat] as LonLat),
        rings: snapGeom ? snapRings : undefined,
        grid: snapGrid,
      })
      const [lon, lat] = snapped
      const dup = current.findIndex((v) => v.lon === lon && v.lat === lat)
      // Clicking the first corner closes the ring; other repeats are dropped.
      if (dup === 0 && current.length >= 3) {
        finishDrawing()
        return
      }
      if (dup >= 0) return
      setDrawVertices([...current, { lon, lat }])
    },
    [drawVertices, finishDrawing, snapGeom, snapGrid, snapRings],
  )

  const onVertexUndo = useCallback(() => setDrawVertices((vs) => vs.slice(0, -1)), [])

  const onVertexRemove = useCallback(
    (index: number) => setDrawVertices((vs) => vs.filter((_, i) => i !== index)),
    [],
  )

  const onToggleSnap = useCallback((key: 'geom' | 'grid') => {
    if (key === 'geom') setSnapGeom((v) => !v)
    else setSnapGrid((v) => !v)
  }, [])

  const selectedSummary = useMemo(() => {
    if (!selectedParcel) return null
    const off = offChainParcels.find((p) => p.holder === selectedParcel.holder)
    return { onchain: selectedParcel, off }
  }, [selectedParcel, offChainParcels])

  const onSelectParcel = (p: OnChainParcelItem) => selectParcel(p)

  const onParcelClick = useCallback(
    (id: string) => {
      const off = offChainParcels.find((p) => p.id === id)
      if (!off) return
      // Link the off-chain geometry record to its on-chain ownership by
      // matching on holder + name (both are written at registration time).
      const onchain = useAppStore
        .getState()
        .parcels.find((p) => p.holder === off.holder && p.account.name === off.name)
      if (onchain) selectParcel(onchain)
      else selectOffChain(off)
    },
    [offChainParcels, selectParcel, selectOffChain],
  )

  return (
    <div className="globe-layout">
      {/* 3D globe (2D Leaflet fallback without WebGL) */}
      <main className="flex-1 relative flex flex-col">
        {webglStatus && (
          <div className="globe-notice">
            WebGL unavailable — showing the 2D map. Cesium: {webglStatus}
          </div>
        )}
        <div className="globe-stage">
        <TerraGlobe
          offChainParcels={shownParcels}
          roads={shownRoads}
          pois={shownPois}
          drawing={drawing}
          drawVertices={drawVertices}
          onDrawVertexAdd={onVertexAdd}
          onDrawFinish={finishDrawing}
          onParcelClick={onParcelClick}
          onWebGLStatus={setWebglStatus}
        />

        {/* stats overlay */}
        <div className="globe-stats absolute top-3 left-3 bg-surface/90 rounded-lg shadow px-3 py-2 text-[11px] pointer-events-none flex flex-wrap gap-3 items-center">
          <span>
            🗺️ Parcels: <b>{offChainParcels.length}</b>
          </span>
          <span>
            🛣️ Roads: <b>{fusionStats?.roads ?? geoStats?.roads ?? roads.length}</b>
          </span>
          <span>
            📍 POIs: <b>{fusionStats?.pois ?? geoStats?.pois ?? pois.length}</b>
          </span>
          <span>
            📏 {geoStats?.road_length_km ? `${geoStats.road_length_km.toFixed(0)} km` : 'OSM off'}
          </span>
          {usingDemo && (
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-indigo-100 text-indigo-800">
              demo data
            </span>
          )}
        </div>

        {/* layer toggles */}
        <div className="globe-chips absolute top-3 right-3 bg-surface/90 rounded-lg shadow px-2 py-1.5 flex gap-1 text-[11px]">
          {(
            [
              ['parcels', 'Parcels', Square],
              ['roads', 'Roads', Route],
              ['pois', 'POIs', MapPin],
            ] as const
          ).map(([key, label, Icon]) => (
            <button
              key={key}
              className={`btn btn-ghost px-2 py-1 gap-1 ${showLayers[key] ? 'text-emerald-700' : 'text-muted'}`}
              onClick={() => setShowLayers((s) => ({ ...s, [key]: !s[key] }))}
              title={`Toggle ${label}`}
            >
              <Icon size={12} />
              {label}
            </button>
          ))}
        </div>
        </div>
      </main>

      {/* Sidebar */}
      <aside className="globe-side bg-surface flex flex-col">
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
              onUndoVertex={onVertexUndo}
              onRemoveVertex={onVertexRemove}
              onFinishDrawing={finishDrawing}
              snapGeom={snapGeom}
              snapGrid={snapGrid}
              onToggleSnap={onToggleSnap}
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
              <ParcelPanel address={selectedParcel.address} account={selectedParcel.account} holder={selectedParcel.holder} />
              {selectedSummary?.off && (
                <div className="px-3 pb-3 text-[12px] text-muted">
                  Off-chain record: <b>{selectedSummary.off.status}</b> ·{' '}
                  {selectedSummary.off.area_m2 != null
                    ? `${selectedSummary.off.area_m2.toFixed(0)} m²`
                    : 'no area'}
                </div>
              )}
            </>
          ) : selectedOffChain ? (
            <>
              <div className="flex items-center justify-between px-3 pt-2">
                <span className="text-[12px] text-muted">Selected map parcel</span>
                <button
                  className="btn btn-ghost p-1"
                  onClick={() => selectOffChain(null)}
                  title="Close"
                >
                  ✕
                </button>
              </div>
              <OffChainParcelPanel parcel={selectedOffChain} />
            </>
          ) : (
            <ParcelListPanel onSelect={onSelectParcel} onSelectOffChain={selectOffChain} />
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
