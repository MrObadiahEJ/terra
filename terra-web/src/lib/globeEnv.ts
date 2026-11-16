// Shared 3D environment configuration for all Terra pages.
//
// Every page that renders the Cesium globe (Atlas, Demo, Land detail, MVP)
// should use `useGlobeEnv()` so the 3D environment is identical to the /atlas
// page: same basemap default, same graphics tier selection, same tier
// persistence, same camera focus handling, same WebGL fallback behaviour.
// Pages that only need a subset can destructure what they want; the hook
// returns the full superset of TerraGlobe props.
//
// Design constraints:
//   - Zero budget: keyless basemaps (Esri World Imagery, OpenStreetMap,
//     OpenTopoMap), no Cesium Ion token by default.
//   - Offline-first: works with the API unreachable (demo data fallback).
//   - No mandatory wallet, no transaction signing for map rendering.

import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { detectHardwareCap, loadTierPreference, saveTierPreference, stepDownTier, TIER_SETTINGS, type GraphicsTier, type TierPreference } from './perfTier'
import { DEFAULT_FOCUS } from './constants'
import { useAppStore } from '../store/appStore'
import { loadAtlasLayers, saveAtlasLayers, type AtlasLayers } from './atlasLayers'
import { loadPlannedRoads, savePlannedRoads, type PlannedRoad } from './atlasLayers'
import { lodFromHeight, lodFromLeafletZoom } from './lod'
import { parseGeoJSON, type OffChainParcel, type OsmBuildingFootprint, type RoadRow, type PoiRow } from './api'
import { selectBuildingsForTier } from './atlasLayers'
import { type FocusTarget } from '../components/map/TerraGlobe'
import { type QuadCellRow, type QuadCoverRow } from './runtime'
import { type SceneNodeRow } from './runtime'
import { type TerrainStatus } from './cesiumTerrain'
import { polygonCentroid, type LonLat } from './geo'
import { useLocale } from './locale'

// ---------------------------------------------------------------------------
// Hook
// ---------------------------------------------------------------------------

export interface GlobeEnv {
  // ---- core data ----------------------------------------------------------
  offChainParcels: OffChainParcel[]
  roads: RoadRow[]
  pois: PoiRow[]
  buildings: OsmBuildingFootprint[]
  plannedRoads: PlannedRoad[]
  layers: AtlasLayers
  // ---- view state ---------------------------------------------------------
  viewMode: '2d' | '3d'
  basemap: 'imagery' | 'terrain' | 'osm'
  graphicsTier: GraphicsTier
  tierPref: TierPreference
  autoTier: boolean
  focus: FocusTarget | null
  // ---- derived / progressive-disclosure ----------------------------------
  shownParcels: OffChainParcel[]
  visibleRoads: RoadRow[]
  visiblePois: PoiRow[]
  visibleBuildings: OsmBuildingFootprint[]
  showLabels: boolean
  lod: number
  // ---- 3D scene graph / runtime (REQ-027 / REQ-028) -----------------------
  tilesetUrl: string | null
  runtimeNodes: SceneNodeRow[]
  coverCells: QuadCoverRow | null
  fetchedNodeIds: string[]
  fetchedAssets: Map<string, { data: ArrayBuffer; transform: any | null; crs: string | null }>
  // ---- actions ------------------------------------------------------------
  setViewMode: (mode: '2d' | '3d') => void
  setBasemap: (basemap: 'imagery' | 'terrain' | 'osm') => void
  changeTierPref: (pref: TierPreference) => void
  setFocus: (focus: FocusTarget) => void
  setLod: (lod: number) => void
  setLayers: (layers: AtlasLayers) => void
  setPlannedRoads: (roads: PlannedRoad[]) => void
  onCameraMoved: (cam: { longitude: number; latitude: number; height: number }) => void
  onMapZoom: (zoom: number) => void
  onTierCapped: () => void
  settleMapResize: () => void
  // runtime setters
  setTilesetUrl: (url: string | null) => void
  setRuntimeNodes: (nodes: SceneNodeRow[]) => void
  setCoverCells: (cells: QuadCoverRow | null) => void
  setFetchedNodeIds: (ids: string[]) => void
  setFetchedAssets: (assets: Map<string, { data: ArrayBuffer; transform: any | null; crs: string | null }>) => void
}

export function useGlobeEnv(): GlobeEnv {
  const { t } = useLocale()
  const offChainParcels = useAppStore((s) => s.offChainParcels)
  const refreshOffChain = useAppStore((s) => s.refreshOffChain)
  const loadStats = useAppStore((s) => s.loadStats)
  const selectedParcel = useAppStore((s) => s.selectedParcel)
  const selectedOffChain = useAppStore((s) => s.selectedOffChain)
  const demoMode = useAppStore((s) => s.demoMode)

  // Immersive atlas chrome.
  useEffect(() => {
    document.body.classList.add('atlas-immersive')
    return () => document.body.classList.remove('atlas-immersive')
  }, [])

  // Initial load of off-chain data + stats.
  useEffect(() => {
    refreshOffChain()
    loadStats()
  }, [])

  // ---- layers / planned roads ---------------------------------------------
  const [layers, setLayers] = useState<AtlasLayers>(loadAtlasLayers)
  const [plannedRoads, setPlannedRoadsState] = useState<PlannedRoad[]>(loadPlannedRoads)

  useEffect(() => {
    saveAtlasLayers(layers)
  }, [layers])

  useEffect(() => {
    savePlannedRoads(plannedRoads)
  }, [plannedRoads])

  // ---- basemap (persisted) -------------------------------------------------
  const [basemap, setBasemapState] = useState<'imagery' | 'terrain' | 'osm'>(() => {
    const saved = localStorage.getItem('terra-atlas-basemap')
    return saved === 'osm' || saved === 'terrain' ? saved : 'imagery'
  })

  useEffect(() => {
    localStorage.setItem('terra-atlas-basemap', basemap)
  }, [basemap])

  const setBasemap = useCallback((b: 'imagery' | 'terrain' | 'osm') => {
    setBasemapState(b)
  }, [])

  // ---- graphics tier -------------------------------------------------------
  const [tierPref, setTierPrefState] = useState<TierPreference>(() => loadTierPreference())
  const [tierStep, setTierStep] = useState(0)
  const hardwareCap = useMemo(() => detectHardwareCap(), [])

  const graphicsTier = useMemo<GraphicsTier>(() => {
    if (tierPref !== 'auto') return tierPref
    let tier = hardwareCap
    for (let i = 0; i < tierStep && tier !== 'low'; i += 1) tier = stepDownTier(tier)
    return tier
  }, [tierPref, tierStep, hardwareCap])

  const autoTier = tierPref === 'auto'

  useEffect(() => {
    saveTierPreference(tierPref)
  }, [tierPref])

  const changeTierPref = useCallback((pref: TierPreference) => {
    setTierPrefState(pref)
    setTierStep(0)
  }, [])

  const onTierCapped = useCallback(() => {
    if (tierPref !== 'auto' || graphicsTier === 'low') return
    const next = stepDownTier(graphicsTier)
    setTierStep((step) => step + 1)
  }, [tierPref, graphicsTier])

  // ---- camera / focus ------------------------------------------------------
  const [focus, setFocusState] = useState<FocusTarget>(() => {
    const c = new URLSearchParams(window.location.search).get('c')
    if (c) {
      const [lon, lat, h] = c.split(',').map(Number)
      if (Number.isFinite(lon) && Number.isFinite(lat)) {
        const cameraHeight =
          Number.isFinite(h) && h >= 100
            ? h
            : Number.isFinite(h) && h > 0
              ? Math.max(25_000, 591657550 / 2 ** h)
              : 2_500_000
        return {
          longitude: lon,
          latitude: lat,
          height: cameraHeight,
          duration: 0,
        }
      }
    }
    return parcelFocus(selectedOffChain) ?? DEFAULT_FOCUS
  })

  const setFocus = useCallback((f: FocusTarget) => {
    setFocusState(f)
  }, [])

  const lastCamRef = useRef({ ...focus })
  const [lod, setLodState] = useState(() => lodFromHeight(DEFAULT_FOCUS.height))

  const setLod = useCallback((l: number) => {
    setLodState(l)
  }, [])

  const onCameraMoved = useCallback((cam: { longitude: number; latitude: number; height: number }) => {
    lastCamRef.current = cam
    setLodState((prev) => {
      const next = lodFromHeight(cam.height)
      return next === prev ? prev : next
    })
  }, [])

  const onMapZoom = useCallback((zoom: number) => {
    setLodState((prev) => {
      const next = lodFromLeafletZoom(zoom)
      return next === prev ? prev : next
    })
  }, [])

  const setViewMode = useCallback((mode: '2d' | '3d') => {
    setViewModeState(mode)
  }, [])

  const [viewMode, setViewModeState] = useState<'2d' | '3d'>(() =>
    localStorage.getItem('terra-map-view') === '2d' ? '2d' : '3d',
  )

  // ---- resize key ----------------------------------------------------------
  const [mapResizeKey, setMapResizeKey] = useState(0)
  const settleMapResize = useCallback(() => {
    window.setTimeout(() => setMapResizeKey((k) => k + 1), 260)
  }, [])

  // ---- progressive disclosure -----------------------------------------------
  const shownParcels = useMemo(
    () => (layers.parcels && lod >= 4 ? offChainParcels : []),
    [layers.parcels, lod, offChainParcels],
  )
  const usingDemo = demoMode

  const visibleRoads = useMemo(
    () => (layers.roads && lod >= 3 ? roads : []),
    [layers.roads, lod, roads],
  )
  const visiblePois = useMemo(
    () => (layers.pois && lod >= 4 ? pois ?? [] : []),
    [layers.pois, lod, pois],
  )
  const visibleBuildings = useMemo(() => {
    if (lod < 5) return []
    if (!layers.buildings && !usingDemo) return []
    const source = buildings?.length ? buildings : []
    return selectBuildingsForTier(source, TIER_SETTINGS[graphicsTier].buildingCap, graphicsTier)
  }, [lod, layers.buildings, buildings, graphicsTier, usingDemo])
  const showLabels = layers.labels && lod >= TIER_SETTINGS[graphicsTier].labelMinLod

  // ---- runtime scene graph (REQ-027 / REQ-028) ------------------------------
  const [tilesetUrl, setTilesetUrlState] = useState<string | null>(null)
  const [runtimeNodes, setRuntimeNodesState] = useState<SceneNodeRow[]>([])
  const [coverCells, setCoverCellsState] = useState<QuadCoverRow | null>(null)
  const [fetchedNodeIds, setFetchedNodeIdsState] = useState<string[]>([])
  const [fetchedAssets, setFetchedAssetsState] = useState<Map<string, {
    data: ArrayBuffer
    transform: any | null
    crs: string | null
  }>>(new Map())

  // ---- parcel focus helper -------------------------------------------------
  function parcelFocus(parcel: OffChainParcel | null): FocusTarget | null {
    if (!parcel) return null
    const polygon = parseGeoJSON<{ type: string; coordinates: number[][][] }>(parcel.geometry)
    if (!polygon || polygon.type !== 'Polygon' || polygon.coordinates[0].length < 3) return null
    const [longitude, latitude] = polygonCentroid(
      polygon.coordinates[0].map(([lon, lat]) => [lon, lat] as LonLat),
    )
    return { longitude, latitude, height: 1800 }
  }

  return {
    // core data
    offChainParcels,
    roads: visibleRoads,
    pois: visiblePois,
    buildings: visibleBuildings,
    plannedRoads,
    layers,
    // view state
    viewMode,
    basemap,
    graphicsTier,
    tierPref,
    autoTier,
    focus,
    // derived
    shownParcels,
    visibleRoads,
    visiblePois,
    visibleBuildings,
    showLabels,
    lod,
    // 3D scene graph / runtime
    tilesetUrl,
    runtimeNodes,
    coverCells,
    fetchedNodeIds,
    fetchedAssets,
    // actions
    setViewMode,
    setBasemap,
    changeTierPref,
    setFocus,
    setLod,
    setLayers,
    setPlannedRoads: setPlannedRoadsState,
    onCameraMoved,
    onMapZoom,
    onTierCapped,
    settleMapResize,
    // runtime setters
    setTilesetUrl: setTilesetUrlState,
    setRuntimeNodes: setRuntimeNodesState,
    setCoverCells: setCoverCellsState,
    setFetchedNodeIds: setFetchedNodeIdsState,
    setFetchedAssets: setFetchedAssetsState,
  } as GlobeEnv
}
