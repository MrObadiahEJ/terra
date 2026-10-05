import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useSearchParams } from 'react-router-dom'
import TerraGlobe, { type DrawVertex, type FocusTarget } from '../components/map/TerraGlobe'
import LodLadder from '../components/map/LodLadder'
import Sidenav, { type SidenavMode } from '../components/layout/Sidenav'
import { lodFromHeight, lodFromLeafletZoom, LOD_TARGET_HEIGHT } from '../lib/lod'
import RegisterParcelPanel from '../components/panels/RegisterParcelPanel'
import ParcelListPanel from '../components/panels/ParcelListPanel'
import ParcelPanel from '../components/panels/ParcelPanel'
import OffChainParcelPanel from '../components/panels/OffChainParcelPanel'
import BuildingPanel from '../components/panels/BuildingPanel'
import RepresentationPanel from '../components/panels/RepresentationPanel'
import PlanRoadPanel from '../components/panels/PlanRoadPanel'
import { useAppStore, type OnChainParcelItem } from '../store/appStore'
import {
  api,
  parseGeoJSON,
  type OffChainParcel,
  type OsmBuildingFootprint,
  type RoadRow,
  type PoiRow,
} from '../lib/api'
import { DEMO_ROADS, DEMO_POIS } from '../lib/demoData'
import {
  buildingRing,
  loadAtlasLayers,
  saveAtlasLayers,
  loadPlannedRoads,
  savePlannedRoads,
  type AtlasLayerKey,
  type AtlasLayers,
  type PlannedRoad,
} from '../lib/atlasLayers'
import { lineLengthM, polygonCentroid, snapPoint, type LonLat } from '../lib/geo'
import {
  Box,
  Building2,
  Check,
  ChevronDown,
  ChevronUp,
  Layers,
  LocateFixed,
  Map as MapIcon,
  MapPin,
  Minus,
  PanelRight,
  PanelRightClose,
  PanelRightOpen,
  PictureInPicture2,
  Plus,
  Route,
  Share2,
  Square,
  Tag,
} from 'lucide-react'
import { DEFAULT_FOCUS } from '../lib/constants'
import { useLocale } from '../lib/locale'

function parcelFocus(parcel: OffChainParcel | null): FocusTarget | null {
  if (!parcel) return null
  const polygon = parseGeoJSON<{ type: string; coordinates: number[][][] }>(parcel.geometry)
  if (!polygon || polygon.type !== 'Polygon' || polygon.coordinates[0].length < 3) return null
  const [longitude, latitude] = polygonCentroid(
    polygon.coordinates[0].map(([lon, lat]) => [lon, lat] as LonLat),
  )
  return { longitude, latitude, height: 1800 }
}

const LAYER_ROWS: { key: AtlasLayerKey; icon: typeof Square; label: string }[] = [
  { key: 'parcels', icon: Square, label: 'Parcels' },
  { key: 'roads', icon: Route, label: 'Roads' },
  { key: 'buildings', icon: Building2, label: 'Buildings' },
  { key: 'pois', icon: MapPin, label: 'Places' },
  { key: 'labels', icon: Tag, label: 'Labels' },
]

export default function GlobePage() {
  const { t } = useLocale()
  const offChainParcels = useAppStore((s) => s.offChainParcels)
  const refreshOffChain = useAppStore((s) => s.refreshOffChain)
  const geoStats = useAppStore((s) => s.geoStats)
  const fusionStats = useAppStore((s) => s.fusionStats)
  const loadStats = useAppStore((s) => s.loadStats)
  const selectedParcel = useAppStore((s) => s.selectedParcel)
  const selectParcel = useAppStore((s) => s.selectParcel)
  const selectedOffChain = useAppStore((s) => s.selectedOffChain)
  const selectOffChain = useAppStore((s) => s.selectOffChain)
  const demoMode = useAppStore((s) => s.demoMode)

  const [drawing, setDrawing] = useState(false)
  const [drawKind, setDrawKind] = useState<'parcel' | 'road'>('parcel')
  const [webglStatus, setWebglStatus] = useState<string | null>(null)
  const [drawVertices, setDrawVertices] = useState<DrawVertex[]>([])
  const [tab, setTab] = useState<'register' | 'browse' | 'plan'>('browse')
  const [layers, setLayers] = useState<AtlasLayers>(loadAtlasLayers)
  const [layersCard, setLayersCard] = useState(false)
  const [plannedRoads, setPlannedRoads] = useState<PlannedRoad[]>(loadPlannedRoads)
  const [roads, setRoads] = useState<RoadRow[] | null>(null)
  const [pois, setPois] = useState<PoiRow[] | null>(null)
  const [buildings, setBuildings] = useState<OsmBuildingFootprint[] | null>(null)
  const [selectedBuilding, setSelectedBuilding] = useState<OsmBuildingFootprint | null>(null)
  const [repr, setRepr] = useState<{
    title: string
    ring: LonLat[]
    source: 'parcel' | 'building'
  } | null>(null)
  const [snapGeom, setSnapGeom] = useState(true)
  const [snapGrid, setSnapGrid] = useState(false)
  const [viewMode, setViewMode] = useState<'3d' | '2d'>(() =>
    localStorage.getItem('terra-map-view') === '2d' ? '2d' : '3d',
  )
  const [focus, setFocus] = useState<FocusTarget>(
    () => parcelFocus(useAppStore.getState().selectedOffChain) ?? DEFAULT_FOCUS,
  )
  const [lod, setLod] = useState(() => lodFromHeight(DEFAULT_FOCUS.height))
  const [basemap, setBasemap] = useState<'imagery' | 'terrain' | 'osm'>(
    () =>
      (localStorage.getItem('terra-atlas-basemap') as 'imagery' | 'terrain' | 'osm' | null) ??
      'imagery',
  )
  const lastCamRef = useRef({ ...DEFAULT_FOCUS })

  useEffect(() => {
    localStorage.setItem('terra-atlas-basemap', basemap)
  }, [basemap])

  useEffect(() => {
    saveAtlasLayers(layers)
  }, [layers])

  useEffect(() => {
    savePlannedRoads(plannedRoads)
  }, [plannedRoads])

  const [sidenavOpen, setSidenavOpen] = useState(true)
  const [sidenavMode, setSidenavMode] = useState<SidenavMode>(
    () => (localStorage.getItem('terra-atlas-sidenav-mode') === 'over' ? 'over' : 'side'),
  )
  const [mapResizeKey, setMapResizeKey] = useState(0)

  const settleMapResize = useCallback(() => {
    window.setTimeout(() => setMapResizeKey((k) => k + 1), 260)
  }, [])

  const changeSidenavOpen = useCallback(
    (open: boolean) => {
      setSidenavOpen(open)
      settleMapResize()
    },
    [settleMapResize],
  )

  const changeSidenavMode = useCallback(
    (mode: SidenavMode) => {
      setSidenavMode(mode)
      localStorage.setItem('terra-atlas-sidenav-mode', mode)
      settleMapResize()
    },
    [settleMapResize],
  )

  const closeSidenav = useCallback(() => changeSidenavOpen(false), [changeSidenavOpen])

  const onCameraMoved = useCallback((cam: { longitude: number; latitude: number; height: number }) => {
    lastCamRef.current = cam
    setLod((prev) => {
      const next = lodFromHeight(cam.height)
      return next === prev ? prev : next
    })
  }, [])

  const onMapZoom = useCallback((zoom: number) => {
    setLod((prev) => {
      const next = lodFromLeafletZoom(zoom)
      return next === prev ? prev : next
    })
  }, [])

  const jumpToLod = useCallback((target: number) => {
    const clamped = Math.max(0, Math.min(target, LOD_TARGET_HEIGHT.length - 1))
    setFocus({ ...lastCamRef.current, height: LOD_TARGET_HEIGHT[clamped], duration: 0.9 })
  }, [])

  const zoomStep = useCallback((factor: number) => {
    const height = lastCamRef.current.height * factor
    setFocus({
      ...lastCamRef.current,
      height: Math.min(20_000_000, Math.max(5, height)),
      duration: 0.45,
    })
  }, [])

  // Initial load of off-chain data + stats.
  useEffect(() => {
    refreshOffChain()
    loadStats()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const shownParcels = useMemo(
    () => (layers.parcels ? offChainParcels : []),
    [layers.parcels, offChainParcels],
  )
  const usingDemo = demoMode

  const visibleRoads = useMemo(() => (layers.roads ? roads ?? [] : []), [layers.roads, roads])
  const visiblePois = useMemo(() => (layers.pois ? pois ?? [] : []), [layers.pois, pois])
  const visibleBuildings = useMemo(
    () => (layers.buildings ? buildings ?? [] : []),
    [layers.buildings, buildings],
  )
  const showLabels = layers.labels && lod >= 4

  // Lazily fetch each dataset the first time its layer is switched on.
  useEffect(() => {
    let cancelled = false
    if (layers.roads && roads === null) {
      api
        .roads()
        .then((rows) => {
          if (!cancelled) setRoads(rows.length > 0 ? rows : DEMO_ROADS)
        })
        .catch(() => {
          if (!cancelled) setRoads(DEMO_ROADS)
        })
    }
    if (layers.pois && pois === null) {
      api
        .poisFusion()
        .then((rows) => {
          if (!cancelled) setPois(rows.length > 0 ? rows : DEMO_POIS)
        })
        .catch(() => {
          if (!cancelled) setPois(DEMO_POIS)
        })
    }
    if (layers.buildings && buildings === null) {
      api
        .osmBuildingFootprints(500)
        .then((rows) => {
          if (!cancelled) setBuildings(rows)
        })
        .catch(() => {
          if (!cancelled) setBuildings([])
        })
    }
    return () => {
      cancelled = true
    }
  }, [layers.roads, layers.pois, layers.buildings, roads, pois, buildings])

  useEffect(() => {
    localStorage.setItem('terra-map-view', viewMode)
  }, [viewMode])

  // Parcel boundaries as snap candidates — only while drawing.
  const snapRings = useMemo(() => {
    const rings: LonLat[][] = []
    if (!drawing) return rings
    for (const p of shownParcels) {
      const poly = parseGeoJSON<{ type: string; coordinates: number[][][] }>(p.geometry)
      if (!poly || poly.type !== 'Polygon') continue
      rings.push(poly.coordinates[0].map(([lon, lat]) => [lon, lat] as LonLat))
    }
    return rings
  }, [drawing, shownParcels])

  const finishDrawing = useCallback(() => {
    if (drawKind === 'road') {
      if (drawVertices.length < 2) return
      const vertices = drawVertices.map((v) => ({ lon: v.lon, lat: v.lat }))
      const lengthM = lineLengthM(vertices.map((v) => [v.lon, v.lat] as LonLat))
      setPlannedRoads((prev) => [
        ...prev,
        {
          id: `pr-${Date.now().toString(36)}-${prev.length}`,
          name: `Road ${prev.length + 1}`,
          vertices,
          lengthM,
        },
      ])
      setDrawVertices([])
      setDrawing(false)
      return
    }
    if (drawVertices.length >= 3) setDrawing(false)
  }, [drawKind, drawVertices])

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
      // Clicking the first corner closes the ring (roads finish after 2 points);
      // other repeats are dropped.
      const minClose = drawKind === 'road' ? 2 : 3
      if (dup === 0 && current.length >= minClose) {
        finishDrawing()
        return
      }
      if (dup >= 0) return
      setDrawVertices([...current, { lon, lat }])
    },
    [drawKind, drawVertices, finishDrawing, snapGeom, snapGrid, snapRings],
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

  const selectedOffRecord = selectedOffChain ?? selectedSummary?.off ?? null
  const selectedOffRing = useMemo(() => {
    if (!selectedOffRecord) return null
    const poly = parseGeoJSON<{ type: string; coordinates: number[][][] }>(
      selectedOffRecord.geometry,
    )
    if (!poly || poly.type !== 'Polygon' || poly.coordinates[0].length < 3) return null
    return poly.coordinates[0].map(([lon, lat]) => [lon, lat] as LonLat)
  }, [selectedOffRecord])

  const focusParcel = useCallback((parcel: OffChainParcel) => {
    const nextFocus = parcelFocus(parcel)
    if (nextFocus) setFocus(nextFocus)
  }, [])

  const onSelectParcel = useCallback(
    (p: OnChainParcelItem) => {
      selectParcel(p)
      setSelectedBuilding(null)
      setRepr(null)
      changeSidenavOpen(true)
      const offChain = offChainParcels.find(
        (parcel) => parcel.holder === p.holder && parcel.name === p.account.name,
      )
      if (offChain) focusParcel(offChain)
    },
    [changeSidenavOpen, focusParcel, offChainParcels, selectParcel],
  )

  const onSelectOffChain = useCallback(
    (parcel: OffChainParcel) => {
      selectOffChain(parcel)
      setSelectedBuilding(null)
      setRepr(null)
      changeSidenavOpen(true)
      focusParcel(parcel)
    },
    [changeSidenavOpen, focusParcel, selectOffChain],
  )

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
      setSelectedBuilding(null)
      setRepr(null)
      setTab('browse')
      changeSidenavOpen(true)
    },
    [changeSidenavOpen, offChainParcels, selectParcel, selectOffChain],
  )

  const onBuildingClick = useCallback(
    (osmId: number) => {
      const found = buildings?.find((b) => b.osm_id === osmId)
      if (!found) return
      selectParcel(null)
      selectOffChain(null)
      setRepr(null)
      setSelectedBuilding(found)
      setTab('browse')
      changeSidenavOpen(true)
      const ring = buildingRing(found)
      if (ring) {
        const [longitude, latitude] = polygonCentroid(ring)
        setFocus({ longitude, latitude, height: 900, duration: 0.8 })
      }
    },
    [buildings, changeSidenavOpen, selectOffChain, selectParcel],
  )

  const openRepresentation = useCallback(
    (title: string, ring: LonLat[], source: 'parcel' | 'building') => {
      if (ring.length < 3) return
      setRepr({ title, ring, source })
      setSelectedBuilding(null)
      setTab('browse')
      changeSidenavOpen(true)
    },
    [changeSidenavOpen],
  )

  const toggleParcelDraw = useCallback(() => {
    if (drawing && drawKind === 'parcel') {
      setDrawing(false)
      return
    }
    setDrawKind('parcel')
    setDrawVertices([])
    setDrawing(true)
  }, [drawing, drawKind])

  const toggleRoadDraw = useCallback(() => {
    if (drawing && drawKind === 'road') {
      setDrawing(false)
      return
    }
    setDrawKind('road')
    setDrawVertices([])
    setDrawing(true)
    setTab('plan')
  }, [drawing, drawKind])

  const viewBuilding3D = useCallback(() => {
    if (!selectedBuilding) return
    const ring = buildingRing(selectedBuilding)
    if (!ring) return
    openRepresentation(
      selectedBuilding.name ?? `Building ${selectedBuilding.osm_id}`,
      ring,
      'building',
    )
  }, [openRepresentation, selectedBuilding])

  const viewSelectedOff3D = useCallback(() => {
    if (!selectedOffRecord || !selectedOffRing) return
    openRepresentation(selectedOffRecord.name, selectedOffRing, 'parcel')
  }, [openRepresentation, selectedOffRecord, selectedOffRing])

  // Deep link: /atlas?parcel=<id|name|holder|address> selects and frames the land.
  const [searchParams] = useSearchParams()
  const deepLinkParcel = searchParams.get('parcel')
  const deepLinkDone = useRef(false)
  /* eslint-disable react-hooks/set-state-in-effect -- URL query is an external system; syncing it into view state */
  useEffect(() => {
    if (deepLinkDone.current || !deepLinkParcel || offChainParcels.length === 0) return
    deepLinkDone.current = true
    const query = deepLinkParcel.toLowerCase()
    const off = offChainParcels.find(
      (p) =>
        p.id === deepLinkParcel ||
        p.name.toLowerCase() === query ||
        p.holder.toLowerCase() === query,
    )
    if (off) {
      selectOffChain(off)
      setTab('browse')
      changeSidenavOpen(true)
      const nextFocus = parcelFocus(off)
      if (nextFocus) setFocus(nextFocus)
      return
    }
    const onchain = useAppStore
      .getState()
      .parcels.find(
        (p) =>
          p.address === deepLinkParcel ||
          p.holder.toLowerCase() === query ||
          p.account.name.toLowerCase() === query,
      )
    if (onchain) {
      selectParcel(onchain)
      setTab('browse')
      changeSidenavOpen(true)
      const linked = offChainParcels.find(
        (p) => p.holder === onchain.holder && p.name === onchain.account.name,
      )
      const nextFocus = linked ? parcelFocus(linked) : null
      if (nextFocus) setFocus(nextFocus)
    }
  }, [changeSidenavOpen, deepLinkParcel, offChainParcels, selectOffChain, selectParcel])
  /* eslint-enable react-hooks/set-state-in-effect */

  const shareTarget = useMemo(() => {
    if (selectedOffChain) return { id: selectedOffChain.id, name: selectedOffChain.name }
    if (selectedSummary?.off) return { id: selectedSummary.off.id, name: selectedSummary.off.name }
    if (selectedParcel) return { id: selectedParcel.address, name: selectedParcel.account.name }
    return null
  }, [selectedOffChain, selectedSummary, selectedParcel])

  const [shareCopied, setShareCopied] = useState(false)
  const shareLand = useCallback(() => {
    if (!shareTarget) return
    const url = `${window.location.origin}/atlas?parcel=${encodeURIComponent(shareTarget.id)}`
    const confirm = () => {
      setShareCopied(true)
      window.setTimeout(() => setShareCopied(false), 2000)
    }
    if (navigator.clipboard?.writeText) {
      navigator.clipboard.writeText(url).then(confirm).catch(confirm)
    } else {
      confirm()
    }
  }, [shareTarget])

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
          roads={visibleRoads}
          pois={visiblePois}
          buildings={visibleBuildings}
          plannedRoads={plannedRoads}
          viewMode={viewMode}
          focus={focus}
          drawing={drawing}
          drawKind={drawKind}
          drawVertices={drawVertices}
          onDrawVertexAdd={onVertexAdd}
          onDrawFinish={finishDrawing}
          onParcelClick={onParcelClick}
          onBuildingClick={onBuildingClick}
          onCameraMoved={onCameraMoved}
          onMapZoom={onMapZoom}
          showLabels={showLabels}
          basemap={basemap}
          resizeKey={mapResizeKey}
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

        {/* layer toggles + basemap switcher */}
        <div className="globe-chips absolute top-3 right-3 bg-surface/90 rounded-lg shadow px-2 py-1.5 flex gap-1 text-[11px]">
          <button
            className={`btn btn-ghost px-2 py-1 gap-1 ${layers.parcels ? 'text-emerald-700' : 'text-muted'}`}
            onClick={() => setLayers((l) => ({ ...l, parcels: !l.parcels }))}
            aria-pressed={layers.parcels}
            title="Toggle parcel outlines"
          >
            <Square size={12} /> Parcels
          </button>
          <span className="globe-chip-sep" aria-hidden="true" />
          {([
            ['imagery', 'Satellite'],
            ['terrain', 'Terrain'],
            ['osm', 'Streets'],
          ] as const).map(([style, label]) => (
            <button
              key={style}
              className={`btn btn-ghost px-2 py-1 ${basemap === style ? 'globe-basemap-on' : 'text-muted'}`}
              onClick={() => setBasemap(style)}
              aria-pressed={basemap === style}
              title={`Switch basemap to ${label.toLowerCase()}`}
            >
              {label}
            </button>
          ))}
          <span className="globe-chip-sep" aria-hidden="true" />
          <button
            className={`btn btn-ghost px-2 py-1 gap-1 ${layersCard ? 'globe-basemap-on' : 'text-muted'}`}
            onClick={() => setLayersCard((open) => !open)}
            aria-pressed={layersCard}
            aria-expanded={layersCard}
            title="Map layers"
          >
            <Layers size={12} /> Layers
          </button>
          {layersCard && (
            <div className="globe-layers-card" role="group" aria-label="Map layers">
              <div className="globe-layers-card-head">
                <span>Map details</span>
                <button
                  onClick={() => setLayersCard(false)}
                  aria-label="Close layer panel"
                  title="Close"
                >
                  ✕
                </button>
              </div>
              {LAYER_ROWS.map(({ key, icon: Icon, label }) => (
                <button
                  key={key}
                  className="layer-row"
                  aria-pressed={layers[key]}
                  onClick={() => setLayers((l) => ({ ...l, [key]: !l[key] }))}
                >
                  <span className={`layer-box ${layers[key] ? 'on' : ''}`} aria-hidden="true">
                    {layers[key] ? <Check size={11} /> : null}
                  </span>
                  <Icon size={14} />
                  <span className="flex-1 text-left">{label}</span>
                </button>
              ))}
            </div>
          )}
        </div>

        {/* progressive zoom ladder */}
        <LodLadder lod={lod} onSelect={jumpToLod} />

        <div className="map-controls">
          {!sidenavOpen && (
            <button
              className="map-icon-button"
              onClick={() => changeSidenavOpen(true)}
              title="Show land properties"
              aria-label="Show land properties"
            >
              <PanelRightOpen size={16} />
            </button>
          )}
          <div className="map-zoom-group">
            <button
              className="map-icon-button"
              onClick={() => zoomStep(0.5)}
              title="Zoom in"
              aria-label="Zoom in"
            >
              <Plus size={16} />
            </button>
            <button
              className="map-icon-button"
              onClick={() => zoomStep(2)}
              title="Zoom out"
              aria-label="Zoom out"
            >
              <Minus size={16} />
            </button>
          </div>
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
      <Sidenav
        open={sidenavOpen}
        mode={sidenavMode}
        onClose={closeSidenav}
        label="Land properties"
        className="globe-side"
      >
        <div className="land-panel-heading">
          <div>
            <span className="panel-eyebrow">YOUR PORTFOLIO</span>
            <h2>{t('ownedLandAssets')}</h2>
          </div>
          <div className="sidenav-tools">
            <span className="asset-count">{offChainParcels.length}</span>
            <div className="sidenav-mode" role="group" aria-label="Sidebar mode">
              <button
                className={sidenavMode === 'side' ? 'active' : ''}
                onClick={() => changeSidenavMode('side')}
                aria-pressed={sidenavMode === 'side'}
                title="Docked sidebar"
                aria-label="Docked sidebar"
              >
                <PanelRight size={13} />
              </button>
              <button
                className={sidenavMode === 'over' ? 'active' : ''}
                onClick={() => changeSidenavMode('over')}
                aria-pressed={sidenavMode === 'over'}
                title="Floating sidebar"
                aria-label="Floating sidebar"
              >
                <PictureInPicture2 size={13} />
              </button>
            </div>
            <button
              className="sidenav-reduce"
              onClick={closeSidenav}
              title="Minimize sidebar"
              aria-label="Minimize sidebar"
            >
              <PanelRightClose size={15} />
            </button>
          </div>
        </div>
        <div className="portfolio-summary">
          <div><span>Registered parcels</span><strong>{offChainParcels.length}</strong></div>
          <div><span>Mapped area</span><strong>{formatArea(offChainParcels.reduce((total, parcel) => total + (parcel.area_m2 ?? 0), 0))}</strong></div>
        </div>
        {shareTarget && (
          <button className="atlas-share" onClick={shareLand} title={`Copy link to ${shareTarget.name}`}>
            <Share2 size={13} />
            {shareCopied ? 'Link copied!' : 'Share this land'}
          </button>
        )}
        <div className="flex border-b panel-tabs">
          {(['register', 'browse', 'plan'] as const).map((tabId) => (
            <button
              key={tabId}
              className={`flex-1 py-2 text-[13px] font-medium capitalize hover:bg-bg ${
                tab === tabId ? 'border-b-2 border-emerald-500' : 'text-muted'
              }`}
              onClick={() => setTab(tabId)}
              aria-pressed={tab === tabId}
            >
              {tabId === 'register' ? 'Register' : tabId === 'browse' ? 'Browse' : 'Plan'}
            </button>
          ))}
        </div>

        <div className="flex-1 overflow-y-auto">
          {tab === 'register' ? (
            <RegisterParcelPanel
              drawing={drawing}
              drawVertices={drawVertices}
              onToggleDrawing={toggleParcelDraw}
              onClearDrawing={() => setDrawVertices([])}
              onUndoVertex={onVertexUndo}
              onRemoveVertex={onVertexRemove}
              onFinishDrawing={finishDrawing}
              snapGeom={snapGeom}
              snapGrid={snapGrid}
              onToggleSnap={onToggleSnap}
            />
          ) : tab === 'plan' ? (
            <PlanRoadPanel
              drawing={drawing}
              drawKind={drawKind}
              drawVertices={drawVertices}
              plannedRoads={plannedRoads}
              onToggleDrawing={toggleRoadDraw}
              onClearDrawing={() => setDrawVertices([])}
              onUndoVertex={onVertexUndo}
              onFinishDrawing={finishDrawing}
              onRemoveRoad={(id) =>
                setPlannedRoads((prev) => prev.filter((road) => road.id !== id))
              }
              snapGeom={snapGeom}
              snapGrid={snapGrid}
              onToggleSnap={onToggleSnap}
            />
          ) : repr ? (
            <RepresentationPanel
              title={repr.title}
              ring={repr.ring}
              source={repr.source}
              onBack={() => setRepr(null)}
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
              <ParcelPanel
                address={selectedParcel.address}
                account={selectedParcel.account}
                holder={selectedParcel.holder}
                onView3D={viewSelectedOff3D ?? undefined}
              />
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
              <OffChainParcelPanel
                parcel={selectedOffChain}
                onView3D={viewSelectedOff3D ?? undefined}
              />
            </>
          ) : selectedBuilding ? (
            <BuildingPanel
              footprint={selectedBuilding}
              onView3D={viewBuilding3D}
              onClose={() => setSelectedBuilding(null)}
            />
          ) : (
            <ParcelListPanel onSelect={onSelectParcel} onSelectOffChain={onSelectOffChain} />
          )}
        </div>

        {/* collapsible mini stats footer */}
        <DetailsFooter geoStats={geoStats} fusionStats={fusionStats} />
      </Sidenav>
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
