import { useCallback, useEffect, useMemo, useState } from 'react'
import TerraGlobe, { type DrawVertex } from '../components/map/TerraGlobe'
import RegisterParcelPanel from '../components/panels/RegisterParcelPanel'
import ParcelListPanel from '../components/panels/ParcelListPanel'
import ParcelPanel from '../components/panels/ParcelPanel'
import OffChainParcelPanel from '../components/panels/OffChainParcelPanel'
import { useAppStore, type OnChainParcelItem } from '../store/appStore'
import { parseGeoJSON, type OffChainParcel } from '../lib/api'
import { polygonCentroid, snapPoint, type LonLat } from '../lib/geo'
import {
  Box,
  ChevronDown,
  ChevronUp,
  LocateFixed,
  Map as MapIcon,
  Square,
} from 'lucide-react'
import { DEFAULT_FOCUS } from '../lib/constants'

function parcelFocus(parcel: OffChainParcel | null): typeof DEFAULT_FOCUS | null {
  if (!parcel) return null
  const polygon = parseGeoJSON<{ type: string; coordinates: number[][][] }>(parcel.geometry)
  if (!polygon || polygon.type !== 'Polygon' || polygon.coordinates[0].length < 3) return null
  const [longitude, latitude] = polygonCentroid(
    polygon.coordinates[0].map(([lon, lat]) => [lon, lat] as LonLat),
  )
  return { longitude, latitude, height: 1800 }
}

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
  const [tab, setTab] = useState<'register' | 'browse'>('browse')
  const [showParcels, setShowParcels] = useState(true)
  const [snapGeom, setSnapGeom] = useState(true)
  const [snapGrid, setSnapGrid] = useState(false)
  const [viewMode, setViewMode] = useState<'3d' | '2d'>(() =>
    localStorage.getItem('terra-map-view') === '2d' ? '2d' : '3d',
  )
  const [focus, setFocus] = useState(
    () => parcelFocus(useAppStore.getState().selectedOffChain) ?? DEFAULT_FOCUS,
  )

  // Initial load of off-chain data + stats.
  useEffect(() => {
    refreshOffChain()
    loadStats()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const shownParcels = useMemo(
    () => (showParcels ? offChainParcels : []),
    [showParcels, offChainParcels],
  )
  const usingDemo = demoMode

  useEffect(() => {
    localStorage.setItem('terra-map-view', viewMode)
  }, [viewMode])

  // Parcel boundaries as snap candidates while drawing.
  const snapRings = useMemo(() => {
    const rings: LonLat[][] = []
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

  const focusParcel = useCallback((parcel: OffChainParcel) => {
    const nextFocus = parcelFocus(parcel)
    if (nextFocus) setFocus(nextFocus)
  }, [])

  const onSelectParcel = useCallback((p: OnChainParcelItem) => {
    selectParcel(p)
    const offChain = offChainParcels.find(
      (parcel) => parcel.holder === p.holder && parcel.name === p.account.name,
    )
    if (offChain) focusParcel(offChain)
  }, [focusParcel, offChainParcels, selectParcel])

  const onSelectOffChain = useCallback((parcel: OffChainParcel) => {
    selectOffChain(parcel)
    focusParcel(parcel)
  }, [focusParcel, selectOffChain])

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
          roads={[]}
          pois={[]}
          viewMode={viewMode}
          focus={focus}
          drawing={drawing}
          drawVertices={drawVertices}
          onDrawVertexAdd={onVertexAdd}
          onDrawFinish={finishDrawing}
          onParcelClick={onParcelClick}
          onWebGLStatus={(status) => {
            setWebglStatus(status)
            if (status) setViewMode('2d')
          }}
        />

        <div className="explore-heading">
          <span className="explore-kicker"><span className="live-dot" /> LAND INTELLIGENCE</span>
          <h1>Explore the atlas</h1>
          <p>Discover, verify and manage land from one place.</p>
        </div>

        {/* stats overlay */}
        <div className="globe-stats absolute top-3 left-3 bg-surface/90 rounded-lg shadow px-3 py-2 text-[11px] pointer-events-none flex flex-wrap gap-3 items-center">
          <span>
            <b>{offChainParcels.length}</b> parcels
          </span>
          <span><b>{fusionStats?.roads ?? geoStats?.roads ?? 0}</b> mapped roads in dataset</span>
          <span><b>{fusionStats?.pois ?? geoStats?.pois ?? 0}</b> mapped places in dataset</span>
          <span>
            {geoStats?.road_length_km ? `${geoStats.road_length_km.toFixed(0)} km mapped` : 'Spatial data'}
          </span>
          {usingDemo && (
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-indigo-100 text-indigo-800">
              demo data
            </span>
          )}
        </div>

        {/* layer toggles */}
        <div className="globe-chips absolute top-3 right-3 bg-surface/90 rounded-lg shadow px-2 py-1.5 flex gap-1 text-[11px]">
          <button
            className={`btn btn-ghost px-2 py-1 gap-1 ${showParcels ? 'text-emerald-700' : 'text-muted'}`}
            onClick={() => setShowParcels((visible) => !visible)}
            aria-pressed={showParcels}
            title="Toggle parcel outlines"
          >
            <Square size={12} /> Parcels
          </button>
        </div>

        <div className="map-controls">
          <div className="map-mode-switch" role="group" aria-label="Map view">
            <button
              className={viewMode === '2d' ? 'active' : ''}
              onClick={() => setViewMode('2d')}
              aria-pressed={viewMode === '2d'}
            >
              <MapIcon size={14} /> 2D
            </button>
            <button
              className={viewMode === '3d' ? 'active' : ''}
              onClick={() => setViewMode('3d')}
              aria-pressed={viewMode === '3d'}
              disabled={Boolean(webglStatus)}
              title={webglStatus ? '3D view requires WebGL' : '3D globe view'}
            >
              <Box size={14} /> 3D
            </button>
          </div>
          <button
            className="map-icon-button"
            onClick={() => setFocus({ ...DEFAULT_FOCUS })}
            title="Return to pilot area"
            aria-label="Return to pilot area"
          >
            <LocateFixed size={16} />
          </button>
        </div>
        </div>
      </main>

      {/* Sidebar */}
      <aside className="globe-side bg-surface flex flex-col">
        <div className="land-panel-heading">
          <div>
            <span className="panel-eyebrow">YOUR PORTFOLIO</span>
            <h2>Land assets</h2>
          </div>
          <span className="asset-count">{offChainParcels.length}</span>
        </div>
        <div className="portfolio-summary">
          <div><span>Registered parcels</span><strong>{offChainParcels.length}</strong></div>
          <div><span>Mapped area</span><strong>{formatArea(offChainParcels.reduce((total, parcel) => total + (parcel.area_m2 ?? 0), 0))}</strong></div>
        </div>
        <div className="flex border-b panel-tabs">
          {(['register', 'browse'] as const).map((t) => (
            <button
              key={t}
              className={`flex-1 py-2 text-[13px] font-medium capitalize hover:bg-bg ${
                tab === t ? 'border-b-2 border-emerald-500' : 'text-muted'
              }`}
              onClick={() => setTab(t)}
              aria-pressed={tab === t}
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
            <ParcelListPanel onSelect={onSelectParcel} onSelectOffChain={onSelectOffChain} />
          )}
        </div>

        {/* collapsible mini stats footer */}
        <DetailsFooter geoStats={geoStats} fusionStats={fusionStats} />
      </aside>
    </div>
  )
}

function formatArea(areaM2: number): string {
  if (areaM2 >= 1_000_000) return `${(areaM2 / 1_000_000).toFixed(2)} km²`
  if (areaM2 >= 10_000) return `${(areaM2 / 10_000).toFixed(2)} ha`
  return `${Math.round(areaM2).toLocaleString()} m²`
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
