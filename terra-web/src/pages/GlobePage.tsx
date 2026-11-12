import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { NavLink, useSearchParams } from 'react-router-dom'
import TerraGlobe, { type DrawVertex, type FocusTarget, type ViewCommand } from '../components/map/TerraGlobe'
import AtlasControlsPanel from '../components/panels/AtlasControlsPanel'
import AtlasSearchPill from '../components/map/AtlasSearchPill'
import Sidenav, { type SidenavMode } from '../components/layout/Sidenav'
import { lodFromHeight, lodFromLeafletZoom, LOD_TARGET_HEIGHT } from '../lib/lod'
import RegisterParcelPanel from '../components/panels/RegisterParcelPanel'
import ParcelListPanel from '../components/panels/ParcelListPanel'
import MediaPanel from '../components/panels/MediaPanel'
import ParcelPanel from '../components/panels/ParcelPanel'
import OffChainParcelPanel from '../components/panels/OffChainParcelPanel'
import BuildingPanel from '../components/panels/BuildingPanel'
import RepresentationPanel from '../components/panels/RepresentationPanel'
import PlanRoadPanel from '../components/panels/PlanRoadPanel'
import { useAppStore, type OnChainParcelItem } from '../store/appStore'
import { DEMO_BUILDINGS } from '../lib/demoData'
import {
  api,
  parseGeoJSON,
  type OffChainParcel,
  type OsmBuildingFootprint,
  type RoadRow,
  type PoiRow,
} from '../lib/api'
import {
  resolveSceneAtLod,
  fetchComponentAsset,
  createManifest,
  listManifests,
  type SceneWithNodes,
  type SceneNodeRow,
  type ManifestRow,
  runtimeLodFromHeight,
} from '../lib/runtime'
import { listModels, type ThreeDModel } from '../lib/reconstruction'
import SpatialHistoryPanel from '../components/panels/SpatialHistoryPanel'
import { listSpatialModels } from '../lib/spatial'
import { type SearchResult } from '../lib/search'
import { DEMO_ROADS, DEMO_POIS } from '../lib/demoData'
import {
  buildingRing,
  loadAtlasLayers,
  saveAtlasLayers,
  loadPlannedRoads,
  savePlannedRoads,
  selectBuildingsForTier,
  type AtlasLayerKey,
  type AtlasLayers,
  type PlannedRoad,
} from '../lib/atlasLayers'
import { lineLengthM, polygonCentroid, snapPoint, type LonLat } from '../lib/geo'
import {
  BookOpen,
  Box,
  Compass,
  Layers,
  LocateFixed,
  Map as MapIcon,
  Maximize2,
  Minimize2,
  Minus,
  Mountain,
  PanelRight,
  PanelRightClose,
  PanelRightOpen,
  Pentagon,
  PictureInPicture2,
  Plus,
  Ruler,
  Satellite,
  Share2,
} from 'lucide-react'
import { DEFAULT_FOCUS } from '../lib/constants'
import { useLocale, type TranslationKey } from '../lib/locale'
import {
  detectHardwareCap,
  loadTierPreference,
  saveTierPreference,
  stepDownTier,
  TIER_SETTINGS,
  type GraphicsTier,
  type TierPreference,
} from '../lib/perfTier'
import { type TerrainStatus } from '../lib/cesiumTerrain'

function parcelFocus(parcel: OffChainParcel | null): FocusTarget | null {
  if (!parcel) return null
  const polygon = parseGeoJSON<{ type: string; coordinates: number[][][] }>(parcel.geometry)
  if (!polygon || polygon.type !== 'Polygon' || polygon.coordinates[0].length < 3) return null
  const [longitude, latitude] = polygonCentroid(
    polygon.coordinates[0].map(([lon, lat]) => [lon, lat] as LonLat),
  )
  return { longitude, latitude, height: 1800 }
}

const TIER_LABEL: Record<GraphicsTier, TranslationKey> = {
  low: 'tierLow',
  medium: 'tierMedium',
  high: 'tierHigh',
  ultra: 'tierUltra',
}

export default function GlobePage() {
  const { t } = useLocale()
  const offChainParcels = useAppStore((s) => s.offChainParcels)
  const refreshOffChain = useAppStore((s) => s.refreshOffChain)
  const loadStats = useAppStore((s) => s.loadStats)
  const selectedParcel = useAppStore((s) => s.selectedParcel)
  const selectParcel = useAppStore((s) => s.selectParcel)
  const selectedOffChain = useAppStore((s) => s.selectedOffChain)
  const selectOffChain = useAppStore((s) => s.selectOffChain)
  const demoMode = useAppStore((s) => s.demoMode)

  // Immersive atlas chrome: the global header floats over the globe and the
  // document itself stops scrolling (Google-Earth-style full-viewport map).
  useEffect(() => {
    document.body.classList.add('atlas-immersive')
    return () => document.body.classList.remove('atlas-immersive')
  }, [])

  const [drawing, setDrawing] = useState(false)
  // One-shot camera commands (reset / north-up / tilt / front / street).
  const [viewCommand, setViewCommand] = useState<ViewCommand | null>(null)
  const [drawKind, setDrawKind] = useState<'parcel' | 'road' | 'measure'>('parcel')
  const [webglStatus, setWebglStatus] = useState<string | null>(null)
  const [drawVertices, setDrawVertices] = useState<DrawVertex[]>([])
  const [tab, setTab] = useState<'register' | 'browse' | 'plan' | 'media' | 'scene'>('browse')
  const [layers, setLayers] = useState<AtlasLayers>(loadAtlasLayers)
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
  const [focus, setFocus] = useState<FocusTarget>(() => {
    // Deep link: ?c=lon,lat,height — shared camera position (GEV share links).
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
    return parcelFocus(useAppStore.getState().selectedOffChain) ?? DEFAULT_FOCUS
  })
  const [lod, setLod] = useState(() => lodFromHeight(DEFAULT_FOCUS.height))
  const [basemap, setBasemap] = useState<'imagery' | 'terrain' | 'osm'>(
    () => {
      const saved = localStorage.getItem('terra-atlas-basemap')
      return saved === 'imagery' || saved === 'terrain' ? saved : 'osm'
    },
  )
  const lastCamRef = useRef({ ...DEFAULT_FOCUS })
  // Mirror of the camera for render output: refs must not be read during
  // render (React compiler rule), so the status bar reads this state instead.
  const [cameraReadout, setCameraReadout] = useState({ ...DEFAULT_FOCUS })

  useEffect(() => {
    localStorage.setItem('terra-atlas-basemap', basemap)
  }, [basemap])

  const [tierPref, setTierPref] = useState<TierPreference>(() => loadTierPreference())
  const [tierStep, setTierStep] = useState(0)
  const [fps, setFps] = useState<number | null>(null)
  const [, setTerrainStatus] = useState<TerrainStatus>('flat')
  const [hudNotice, setHudNotice] = useState<string | null>(null)
  const hardwareCap = useMemo(() => detectHardwareCap(), [])

  const resolvedTier = useMemo<GraphicsTier>(() => {
    if (tierPref !== 'auto') return tierPref
    let tier = hardwareCap
    for (let i = 0; i < tierStep && tier !== 'low'; i += 1) tier = stepDownTier(tier)
    return tier
  }, [tierPref, tierStep, hardwareCap])

  useEffect(() => {
    saveTierPreference(tierPref)
  }, [tierPref])

  useEffect(() => {
    if (!hudNotice) return
    const timer = window.setTimeout(() => setHudNotice(null), 6000)
    return () => window.clearTimeout(timer)
  }, [hudNotice])

  const changeTierPref = useCallback((pref: TierPreference) => {
    setTierPref(pref)
    setTierStep(0)
  }, [])

  const onTierCapped = useCallback(() => {
    if (tierPref !== 'auto' || resolvedTier === 'low') return
    const next = stepDownTier(resolvedTier)
    setTierStep((step) => step + 1)
    setHudNotice(t('tierDowngraded', { tier: t(TIER_LABEL[next]) }))
  }, [tierPref, resolvedTier, t])

  useEffect(() => {
    saveAtlasLayers(layers)
  }, [layers])

  useEffect(() => {
    savePlannedRoads(plannedRoads)
  }, [plannedRoads])

  const [sidenavOpen, setSidenavOpen] = useState(() =>
    typeof window.matchMedia === 'function' ? window.matchMedia('(min-width: 861px)').matches : true,
  )
  const [sidenavMode, setSidenavMode] = useState<SidenavMode>(
    () => (localStorage.getItem('terra-atlas-sidenav-mode') === 'over' ? 'over' : 'side'),
  )
  const [mapResizeKey, setMapResizeKey] = useState(0)
  const [fullscreen, setFullscreen] = useState(false)

  useEffect(() => {
    const onChange = () => setFullscreen(Boolean(document.fullscreenElement))
    document.addEventListener('fullscreenchange', onChange)
    return () => document.removeEventListener('fullscreenchange', onChange)
  }, [])

  const toggleFullscreen = useCallback(() => {
    const request = document.fullscreenElement
      ? document.exitFullscreen()
      : document.documentElement.requestFullscreen()
    void request?.catch(() => {
      /* browser denied fullscreen (permissions / non-user gesture) */
    })
  }, [])

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

  const camMoveTickRef = useRef(0)
  const [camMoveTick, setCamMoveTick] = useState(0)

  const onCameraMoved = useCallback((cam: { longitude: number; latitude: number; height: number }) => {
    lastCamRef.current = cam
    setCameraReadout(cam)
    setLod((prev) => {
      const next = lodFromHeight(cam.height)
      return next === prev ? prev : next
    })
    camMoveTickRef.current += 1
    setCamMoveTick(camMoveTickRef.current)
  }, [])

  const onMapZoom = useCallback((zoom: number) => {
    setLod((prev) => {
      const next = lodFromLeafletZoom(zoom)
      return next === prev ? prev : next
    })
  }, [])

  const switchViewMode = useCallback((mode: '2d' | '3d') => {
    // TerraGlobe restores the camera across Cesium's 2D/3D morph, so toggling
    // keeps the parcel in view instead of jumping back to the whole planet.
    setViewMode(mode)
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

  // Progressive disclosure: each dataset only joins the scene once the camera
  // reaches its level (parcels at neighborhood, buildings at parcel level),
  // so a zoomed-out globe never renders a whole collection at once.
  const shownParcels = useMemo(
    () => (layers.parcels && lod >= 4 ? offChainParcels : []),
    [layers.parcels, lod, offChainParcels],
  )
  const usingDemo = demoMode

  const visibleRoads = useMemo(
    () => (layers.roads && lod >= 3 ? roads ?? [] : []),
    [layers.roads, lod, roads],
  )
  const visiblePois = useMemo(
    () => (layers.pois && lod >= 4 ? pois ?? [] : []),
    [layers.pois, lod, pois],
  )
  const visibleBuildings = useMemo(() => {
    if (lod < 5) return []
    if (!layers.buildings && !usingDemo) return []
    const source = buildings?.length ? buildings : usingDemo ? DEMO_BUILDINGS : []
    return selectBuildingsForTier(
      source,
      TIER_SETTINGS[resolvedTier].buildingCap,
      resolvedTier,
    )
  }, [lod, layers.buildings, buildings, resolvedTier, usingDemo])
  const showLabels = layers.labels && lod >= TIER_SETTINGS[resolvedTier].labelMinLod

  // 3D reconstruction models for the selected parcel (RFC-013 Ph.C). Only
  // high/ultra load the tileset — weak machines keep the memory headroom.
  const [parcelModels, setParcelModels] = useState<{
    parcelId: string
    rows: ThreeDModel[]
  } | null>(null)
  // Distributed runtime scene graph (REQ-027/REQ-028). Resolved on demand for a
  // manifest id; nodes reference independently stored components that the
  // runtime fetches with provider fallback.
  const [sceneGraph, setSceneGraph] = useState<SceneWithNodes | null>(null)
  const [sceneError, setSceneError] = useState<string | null>(null)
  const [sceneResolving, setSceneResolving] = useState(false)
  const [assetFetching, setAssetFetching] = useState<string | null>(null) // component_id in flight
  const [fetchedAsset, setFetchedAsset] = useState<{
    componentId: string
    componentVersion: string
    sizeBytes: number
    data: ArrayBuffer
    transform: any | null
    crs: string | null
  } | null>(null)
  // All fetched component assets, keyed by component ID, for rendering on the globe.
  const [fetchedAssets, setFetchedAssets] = useState<Map<string, {
    data: ArrayBuffer
    transform: any | null
    crs: string | null
  }>>(new Map())
  const [fetchedNodeIds, setFetchedNodeIds] = useState<string[]>([])
  // Distributed runtime: current manifest for the camera's region
  const [currentManifest, setCurrentManifest] = useState<ManifestRow | null>(null)
  const [currentRegion, setCurrentRegion] = useState<string | null>(null)
  // Regions the runtime has already tried (and failed) to resolve. Without this
  // the effect retries on every camera tick while the backend is down, which
  // produced a storm of error banners; a region only re-tries when it changes.
  const failedRegionsRef = useRef<Set<string>>(new Set())
  const manifestRev = useRef(0)
  // Explicit "open on globe" from a Phase C model claim (bypasses the tier
  // gate); bound to the parcel it was requested for, so a selection change
  // invalidates it by derivation — no setState-in-effect reset needed.
  const [tilesetOverride, setTilesetOverride] = useState<{ parcelId: string; uri: string } | null>(
    null,
  )
  const modelTokenRef = useRef(0)
  const selectedParcelId = selectedParcel?.address ?? selectedOffChain?.id ?? null
  useEffect(() => {
    if (!selectedParcelId) return
    const token = ++modelTokenRef.current
    // Pipeline models (0029) + on-chain model claims mirrored for the parcel
    // PDA (RFC-013 Phases C–E); tileset-shaped rows feed the auto loader.
    const pipeline = listModels(selectedParcelId).catch(() => [] as ThreeDModel[])
    const onchain = selectedParcel
      ? listSpatialModels(selectedParcel.address)
          .then((rows) =>
            rows
              .filter(
                (m) => m.model_format === 3 || m.storage_reference.includes('tileset.json'),
              )
              .map(
                (m): ThreeDModel => ({
                  id: m.pubkey,
                  parcel_id: m.parcel_pubkey,
                  reconstruction_job_id: null,
                  uri: m.storage_reference,
                  format: '3dtiles',
                  content_hash: m.model_hash,
                  created_at: m.chain_submitted_at ?? '',
                }),
              ),
          )
          .catch(() => [] as ThreeDModel[])
      : Promise.resolve([] as ThreeDModel[])
    Promise.all([pipeline, onchain])
      .then(([rows, chainRows]) => {
        if (token === modelTokenRef.current) {
          setParcelModels({ parcelId: selectedParcelId, rows: [...rows, ...chainRows] })
        }
      })
      .catch(() => {
        if (token === modelTokenRef.current) setParcelModels({ parcelId: selectedParcelId, rows: [] })
      })
  }, [selectedParcelId, selectedParcel])
  const tilesetUrl = useMemo(() => {
    if (tilesetOverride && tilesetOverride.parcelId === selectedParcelId) {
      return tilesetOverride.uri
    }
    if (resolvedTier !== 'high' && resolvedTier !== 'ultra') return null
    if (!parcelModels || parcelModels.parcelId !== selectedParcelId) return null
    const model =
      parcelModels.rows.find((m) => m.uri.includes('tileset.json') || m.format === '3dtiles') ??
      parcelModels.rows[0]
    return model?.uri ?? null
  }, [parcelModels, selectedParcelId, resolvedTier, tilesetOverride])

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
    return () => {
      cancelled = true
    }
  }, [layers.roads, layers.pois, roads, pois])

  // ---- buildings: viewport-bounded, tier-capped (never fetched on Low) -----
  const buildingQueryRef = useRef({ lon: Number.NaN, lat: Number.NaN, tier: '', token: 0 })
  const refreshBuildings = useCallback(
    (
      cam: { longitude: number; latitude: number; height: number },
      tier: GraphicsTier,
      layerOn: boolean,
    ) => {
      const cap = TIER_SETTINGS[tier].buildingCap
      // Buildings are only drawn within a few km of the camera, so at
      // neighbourhood-plus heights we skip the OSM query entirely; reset the
      // query key so the same grid cell re-fetches when we descend again.
      if (!layerOn || cap <= 0 || cam.height > 40000) {
        buildingQueryRef.current.lon = Number.NaN
        buildingQueryRef.current.lat = Number.NaN
        setBuildings((prev) => (prev && prev.length === 0 ? prev : []))
        return
      }
      const span = Math.min(0.06, Math.max(0.015, cam.height / 9000))
      const grid = 0.008
      const lon = Math.round(cam.longitude / grid) * grid
      const lat = Math.round(cam.latitude / grid) * grid
      const query = buildingQueryRef.current
      if (query.lon === lon && query.lat === lat && query.tier === tier) return
      query.lon = lon
      query.lat = lat
      query.tier = tier
      const token = ++query.token
      api
        .osmBuildingFootprints(cap, 0, {
          minx: lon - span,
          miny: lat - span * 0.7,
          maxx: lon + span,
          maxy: lat + span * 0.7,
        })
        .then((rows) => {
          if (buildingQueryRef.current.token === token) setBuildings(rows)
        })
        .catch(() => {
          if (buildingQueryRef.current.token === token) setBuildings([])
        })
    },
    [],
  )
  useEffect(() => {
    refreshBuildings(lastCamRef.current, resolvedTier, layers.buildings)
  }, [refreshBuildings, resolvedTier, layers.buildings])

  // Auto-create/resolve manifest when camera enters a new region (grid cell).
  // Resolves scene graph at current LOD whenever manifest or LOD changes.
  useEffect(() => {
    const cam = lastCamRef.current
    const regionId = `region-${Math.round(cam.longitude)}-${Math.round(cam.latitude)}`
    // Already resolved this region: just ensure the scene graph matches LOD.
    if (currentRegion === regionId && currentManifest) {
      const lod = runtimeLodFromHeight(cam.height)
      if (sceneGraph && sceneGraph.scene.lod_level !== lod) {
        resolveSceneAtLod(currentManifest.id, lod)
          .then((graph) => setSceneGraph(graph))
          .catch((err) => setSceneError(err instanceof Error ? err.message : String(err)))
      }
      return
    }
    // Backend down (no API): skip silently instead of spamming an error banner
    // on every camera tick. The "Scene" tab still offers a manual retry.
    if (failedRegionsRef.current.has(regionId)) return
    setSceneResolving(true)
    setSceneError(null)
    let cancelled = false
    ;(async () => {
      try {
        const existing = await listManifests(regionId)
        const manifest =
          existing[0] ??
          (await createManifest({
            region_id: regionId,
            region_name: `${cam.latitude.toFixed(2)}°N, ${cam.longitude.toFixed(2)}°E`,
            description: 'Auto-created manifest for camera region',
          }))
        if (cancelled) return
        setCurrentManifest(manifest)
        setCurrentRegion(regionId)
        failedRegionsRef.current.delete(regionId)
        manifestRev.current += 1
        const lod = runtimeLodFromHeight(cam.height)
        const graph = await resolveSceneAtLod(manifest.id, lod)
        if (cancelled) return
        setSceneGraph(graph)
        setSceneError(null)
      } catch (err) {
        if (cancelled) return
        failedRegionsRef.current.add(regionId)
        setSceneError(err instanceof Error ? err.message : String(err))
      } finally {
        if (cancelled) return
        setSceneResolving(false)
      }
    })()
    return () => {
      cancelled = true
    }
  }, [camMoveTick, listManifests, createManifest, runtimeLodFromHeight, currentRegion, currentManifest, sceneGraph])

  // Manual trigger: create/resolve manifest for current camera region
  const createManifestForRegion = useCallback(async () => {
    const cam = lastCamRef.current
    const regionId = `region-${Math.round(cam.longitude)}-${Math.round(cam.latitude)}`
    const regionName = `${cam.latitude.toFixed(2)}°N, ${cam.longitude.toFixed(2)}°E`
    failedRegionsRef.current.delete(regionId)
    try {
      const existing = await listManifests(regionId)
      const manifest =
        existing[0] ??
        (await createManifest({
          region_id: regionId,
          region_name: regionName,
          description: 'Manually created manifest for camera region',
        }))
      setCurrentManifest(manifest)
      setCurrentRegion(regionId)
      manifestRev.current += 1
      const lod = runtimeLodFromHeight(cam.height)
      const graph = await resolveSceneAtLod(manifest.id, lod)
      setSceneGraph(graph)
      setSceneError(null)
    } catch (err) {
      setSceneError(err instanceof Error ? err.message : String(err))
    }
  }, [listManifests, createManifest, runtimeLodFromHeight, t])

  // ---- Distributed runtime scene graph (REQ-027 / REQ-028) ----
  const fetchComponent = useCallback(
    async (node: SceneNodeRow) => {
      setAssetFetching(node.component_version_id)
      try {
      const { data, retrieval } = await fetchComponentAsset(
          node.object_id,
          node.component_version_id,
        )
        setFetchedAsset({
          componentId: node.object_id,
          componentVersion: node.component_version_id,
          sizeBytes: data.byteLength,
          data,
          transform: retrieval.global_transform,
          crs: retrieval.crs,
        })
        setFetchedAssets((prev) => {
          const next = new Map(prev)
          next.set(node.object_id, {
            data,
            transform: retrieval.global_transform,
            crs: retrieval.crs,
          })
          return next
        })
        setFetchedNodeIds((prev) => (prev.includes(node.object_id) ? prev : [...prev, node.object_id]))
      } catch (err) {
        setSceneError(
          err instanceof Error
            ? err.message
            : `failed to fetch ${node.object_id}`,
        )
      } finally {
        setAssetFetching(null)
      }
    },
    [],
  )
  useEffect(() => {
    const timer = window.setTimeout(() => {
      refreshBuildings(lastCamRef.current, resolvedTier, layers.buildings)
      // Shareable camera link: keep ?c=lon,lat,height in sync (replaceState
      // so history is not spammed by every pan/zoom).
      const cam = lastCamRef.current
      const url = new URL(window.location.href)
      url.searchParams.set(
        'c',
        `${cam.longitude.toFixed(5)},${cam.latitude.toFixed(5)},${Math.round(cam.height)}`,
      )
      window.history.replaceState(null, '', url)
    }, 2200)
    return () => window.clearTimeout(timer)
  }, [camMoveTick, refreshBuildings, resolvedTier, layers.buildings])

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
    if (drawKind === 'measure') {
      // Keep the measured line on screen; just stop placing points.
      if (drawVertices.length >= 2) setDrawing(false)
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
      // Clicking the first corner closes the ring (roads finish after 2 points,
      // measurements never close); other repeats are dropped.
      const minClose = drawKind === 'road' ? 2 : drawKind === 'measure' ? Number.POSITIVE_INFINITY : 3
      if (dup === 0 && current.length >= minClose) {
        finishDrawing()
        return
      }
      if (dup >= 0) return
      setDrawVertices([...current, { lon, lat, h: raw.h }])
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

  // ---- sidebar controls: search / camera / goto ----------------------------
  const pickSearch = useCallback((r: SearchResult) => {
    setFocus({ longitude: r.longitude, latitude: r.latitude, height: r.height, duration: 1.4 })
  }, [])

  const gotoCoordinates = useCallback((longitude: number, latitude: number, height: number) => {
    setFocus({ longitude, latitude, height, duration: 1.2 })
  }, [])

  const runViewCommand = useCallback((kind: 'reset' | 'north' | 'tilt') => {
    setViewCommand({ kind, nonce: Date.now() })
  }, [])

  const toggleLayer = useCallback((key: AtlasLayerKey) => {
    setLayers((l) => ({ ...l, [key]: !l[key] }))
  }, [])
  const toggleMeasure = useCallback(() => {
    if (drawKind === 'measure') {
      setDrawing(false)
      setDrawKind('parcel')
    } else {
      setDrawVertices([])
      setDrawKind('measure')
      setDrawing(true)
    }
  }, [drawKind])
  const measureLenM = useMemo(() => {
    if (drawKind !== 'measure' || drawVertices.length < 2) return 0
    return lineLengthM(drawVertices.map((v) => [v.lon, v.lat] as LonLat))
  }, [drawKind, drawVertices])

  const selectedSummary = useMemo(() => {
    if (!selectedParcel) return null
    const off = offChainParcels.find((p) => p.holder === selectedParcel.holder)
    return { onchain: selectedParcel, off }
  }, [selectedParcel, offChainParcels])

  const selectedOffRecord = selectedOffChain ?? selectedSummary?.off ?? null
  // Off-chain geometry linked to the selected on-chain parcel (detail actions).
  const linkedOffParcel = selectedSummary?.off ?? null
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
        ?? DEMO_BUILDINGS.find((b) => b.osm_id === osmId)
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
        const closeUpHeight = found.height_m && found.height_m > 0
          ? Math.max(120, found.height_m * 5)
          : 900
        setFocus({ longitude, latitude, height: closeUpHeight, duration: 0.8 })
      }
    },
    [buildings, changeSidenavOpen, selectOffChain, selectParcel],
  )

  const focusDemoBuilding = useCallback((osmId: number) => {
    const building = DEMO_BUILDINGS.find((item) => item.osm_id === osmId)
    if (!building) return
    const ring = buildingRing(building)
    if (!ring) return
    const [longitude, latitude] = polygonCentroid(ring)
    setSelectedBuilding(building)
    selectParcel(null)
    selectOffChain(null)
    setTab('browse')
    changeSidenavOpen(true)
    setFocus({ longitude, latitude, height: 120, duration: 0.9 })
  }, [changeSidenavOpen, selectOffChain, selectParcel])

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

  // Close observation commands: fly the camera to a frontal or street-level
  // position over the given geometry's centroid (parcels and buildings alike).
  const sendCloseView = useCallback(
    (kind: 'front' | 'street', longitude: number, latitude: number) => {
      setViewMode('3d')
      setViewCommand({ kind, longitude, latitude, nonce: Date.now() })
    },
    [],
  )

  const frontViewOf = useCallback(
    (parcel: OffChainParcel) => {
      const target = parcelFocus(parcel)
      if (target) sendCloseView('front', target.longitude, target.latitude)
    },
    [sendCloseView],
  )

  const streetViewOf = useCallback(
    (parcel: OffChainParcel) => {
      const target = parcelFocus(parcel)
      if (target) sendCloseView('street', target.longitude, target.latitude)
    },
    [sendCloseView],
  )

  // Close frontal / street-level observation of the selected building.
  const viewBuildingClose = useCallback(
    (kind: 'front' | 'street') => {
      if (!selectedBuilding) return
      const ring = buildingRing(selectedBuilding)
      if (!ring) return
      const [longitude, latitude] = polygonCentroid(ring)
      sendCloseView(kind, longitude, latitude)
    },
    [selectedBuilding, sendCloseView],
  )

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
      changeSidenavOpen(
        typeof window.matchMedia === 'function'
          ? window.matchMedia('(min-width: 861px)').matches
          : true,
      )
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
    const cam = lastCamRef.current
    const base = `${window.location.origin}${import.meta.env.BASE_URL}atlas`
    const url = shareTarget
      ? `${base}?parcel=${encodeURIComponent(shareTarget.id)}`
      : `${base}?c=${cam.longitude.toFixed(5)},${cam.latitude.toFixed(5)},${Math.round(cam.height)}`
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
            {t('atlasWebglNotice', { status: webglStatus })}
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
          collapsibleHud
          focus={focus}
          drawing={drawing}
          drawKind={drawKind}
          drawVertices={drawVertices}
          onDrawVertexAdd={onVertexAdd}
          onDrawFinish={finishDrawing}
          onParcelClick={onParcelClick}
          onBuildingClick={onBuildingClick}
          highlightParcelId={selectedOffRecord?.id ?? null}
          highlightBuildingId={selectedBuilding?.osm_id ?? null}
          onCameraMoved={onCameraMoved}
          onMapZoom={onMapZoom}
          showLabels={showLabels}
          basemap={basemap}
          resizeKey={mapResizeKey}
          onWebGLStatus={(status) => {
            setWebglStatus(status)
          }}
          graphicsTier={resolvedTier}
          autoTier={tierPref === 'auto'}
          onFps={setFps}
          onTierCapped={onTierCapped}
          onTerrainStatus={setTerrainStatus}
          tilesetUrl={tilesetUrl}
          runtimeNodes={sceneGraph?.nodes ?? []}
          fetchedNodeIds={fetchedNodeIds}
          fetchedAssets={fetchedAssets}
          onFetchComponent={fetchComponent}
          viewCommand={viewCommand}
        />

        <AtlasSearchPill onPick={pickSearch} />

        <nav className="atlas-rail" aria-label={t('mapControlsTitle')}>
          <button
            type="button"
            className={sidenavOpen ? 'active' : ''}
            aria-pressed={sidenavOpen}
            onClick={() => changeSidenavOpen(!sidenavOpen)}
            title={t('mapControlsTitle')}
          >
            <Layers size={18} />
            <span>{t('mapControlsTitle')}</span>
          </button>
          <NavLink to="/stories" className="atlas-rail-link" title={t('stories')}>
            <BookOpen size={17} />
            <span>{t('stories')}</span>
          </NavLink>
          <button
            type="button"
            className={drawing && drawKind === 'parcel' ? 'active' : ''}
            aria-pressed={drawing && drawKind === 'parcel'}
            onClick={toggleParcelDraw}
            title={t('tabRegister')}
          >
            <Pentagon size={17} />
            <span>{t('tabRegister')}</span>
          </button>
          <button
            type="button"
            className={drawing && drawKind === 'measure' ? 'active' : ''}
            aria-pressed={drawing && drawKind === 'measure'}
            onClick={toggleMeasure}
            title={t('measureAction')}
          >
            <Ruler size={17} />
            <span>{t('measureAction')}</span>
          </button>
          <button type="button" onClick={shareLand} title={t('shareThisLand')}>
            <Share2 size={17} />
            <span>{shareCopied ? t('linkCopied') : t('shareThisLand')}</span>
          </button>
          <button
            type="button"
            onClick={toggleFullscreen}
            title={fullscreen ? t('exitFullscreen') : t('enterFullscreen')}
          >
            {fullscreen ? <Minimize2 size={17} /> : <Maximize2 size={17} />}
            <span>{fullscreen ? t('exitFullscreen') : t('enterFullscreen')}</span>
          </button>
        </nav>

        <div className="atlas-basemap-switch" role="group" aria-label={t('basemapSectionTitle')}>
          {([
            ['imagery', 'basemapSatellite', Satellite],
            ['osm', 'basemapStreets', MapIcon],
            ['terrain', 'basemapTerrain', Mountain],
          ] as const).map(([style, labelKey, Icon]) => (
            <button
              key={style}
              type="button"
              className={basemap === style ? 'active' : ''}
              aria-pressed={basemap === style}
              onClick={() => setBasemap(style)}
              title={t(labelKey)}
            >
              <Icon size={14} />
              <span>{t(labelKey)}</span>
            </button>
          ))}
        </div>

        <div className="atlas-statusbar">
          <span>{cameraReadout.latitude.toFixed(4)}°, {cameraReadout.longitude.toFixed(4)}°</span>
          <span>{Math.round(cameraReadout.height).toLocaleString()} m</span>
          <span>{t('zoomDetail')} {lod}</span>
        </div>

        <div className="explore-heading">
          <span className="explore-kicker"><span className="live-dot" /> {t('atlasKicker')}</span>
          <h1>{t('atlasTitle')}</h1>
          <p>{t('atlasSubtitle')}</p>
        </div>

        {/* Transient measure readout — search, camera commands, layers and the
            zoom ladder all live in the single sidebar (AtlasControlsPanel). */}
        {drawKind === 'measure' && drawVertices.length >= 1 && (
            <div className="measure-bar">
              <Ruler size={12} className="text-muted" />
              <span>
                {t('labelLength')} <b>{measureLenM >= 1000 ? `${(measureLenM / 1000).toFixed(2)} km` : `${measureLenM.toFixed(1)} m`}</b>
                {' · '}
                {t('labelPoints')} <b>{drawVertices.length}</b>
              </span>
              <button
                className="btn btn-ghost px-1.5 py-0.5 text-[11px]"
                onClick={() => setDrawVertices([])}
                disabled={drawVertices.length === 0}
              >
                {t('clearAction')}
              </button>
            </div>
          )}

        {hudNotice && (
          <div className="hud-notice" role="status">
            {hudNotice}
          </div>
        )}

        <div className={`map-controls${sidenavOpen ? ' is-shifted' : ''}`}>
          <button
            className="map-icon-button"
            onClick={() => runViewCommand('north')}
            title={t('northUpTitle')}
            aria-label={t('northUpTitle')}
          >
            <Compass size={17} />
          </button>
          {!sidenavOpen && (
            <button
              className="map-icon-button"
              onClick={() => changeSidenavOpen(true)}
              title={t('showLandProperties')}
              aria-label={t('showLandProperties')}
            >
              <PanelRightOpen size={16} />
            </button>
          )}
          <div className="map-zoom-group">
            <button
              className="map-icon-button"
              onClick={() => zoomStep(0.5)}
              title={t('zoomIn')}
              aria-label={t('zoomIn')}
            >
              <Plus size={16} />
            </button>
            <button
              className="map-icon-button"
              onClick={() => zoomStep(2)}
              title={t('zoomOut')}
              aria-label={t('zoomOut')}
            >
              <Minus size={16} />
            </button>
          </div>
          <div className="map-mode-switch" role="group" aria-label={t('mapView')}>
            <button
              className={viewMode === '2d' ? 'active' : ''}
              onClick={() => switchViewMode('2d')}
              aria-pressed={viewMode === '2d'}
            >
              <MapIcon size={14} /> 2D
            </button>
            <button
              className={viewMode === '3d' ? 'active' : ''}
              onClick={() => switchViewMode('3d')}
              aria-pressed={viewMode === '3d'}
              title={webglStatus ? t('threeDGlobeView') + ' (lightweight fallback)' : t('threeDGlobeView')}
            >
              <Box size={14} /> 3D
            </button>
          </div>
          <button
            className="map-icon-button"
            onClick={() => setFocus({ ...DEFAULT_FOCUS })}
            title={t('returnPilotArea')}
            aria-label={t('returnPilotArea')}
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
        label={t('landProperties')}
        className="globe-side"
      >
        <div className="land-panel-heading">
          <div>
            <span className="panel-eyebrow">{t('yourPortfolio')}</span>
            <h2>{t('ownedLandAssets')}</h2>
          </div>
          <div className="sidenav-tools">
            <span className="asset-count">{offChainParcels.length}</span>
            <div className="sidenav-mode" role="group" aria-label={t('sidebarMode')}>
              <button
                className={sidenavMode === 'side' ? 'active' : ''}
                onClick={() => changeSidenavMode('side')}
                aria-pressed={sidenavMode === 'side'}
                title={t('dockedSidebar')}
                aria-label={t('dockedSidebar')}
              >
                <PanelRight size={13} />
              </button>
              <button
                className={sidenavMode === 'over' ? 'active' : ''}
                onClick={() => changeSidenavMode('over')}
                aria-pressed={sidenavMode === 'over'}
                title={t('floatingSidebar')}
                aria-label={t('floatingSidebar')}
              >
                <PictureInPicture2 size={13} />
              </button>
            </div>
            <button
              className="sidenav-reduce"
              onClick={closeSidenav}
              title={t('minimizeSidebar')}
              aria-label={t('minimizeSidebar')}
            >
              <PanelRightClose size={15} />
            </button>
          </div>
        </div>
        {shareTarget && (
          <button className="atlas-share" onClick={shareLand} title={t('copyLinkTo', { name: shareTarget.name })}>
            <Share2 size={13} />
            {shareCopied ? t('linkCopied') : t('shareThisLand')}
          </button>
        )}
        <AtlasControlsPanel
          onFlyTo={pickSearch}
          onGotoCoordinates={gotoCoordinates}
          onCommand={runViewCommand}
          onToggleMeasure={toggleMeasure}
          measuring={drawKind === 'measure' && drawing}
          layers={layers}
          onToggleLayer={toggleLayer}
          basemap={basemap}
          onBasemap={setBasemap}
          tierPref={tierPref}
          onTierPref={changeTierPref}
          fps={fps}
          lod={lod}
          onJumpLod={jumpToLod}
        />
        <div className="flex border-b panel-tabs">
          {(['register', 'browse', 'plan', 'media', 'scene'] as const).map((tabId) => (
            <button
              key={tabId}
              className={`flex-1 py-2 text-[13px] font-medium capitalize hover:bg-bg ${
                tab === tabId ? 'border-b-2 border-emerald-500' : 'text-muted'
              }`}
              onClick={() => setTab(tabId)}
              aria-pressed={tab === tabId}
            >
              {tabId === 'register'
                ? t('tabRegister')
                : tabId === 'browse'
                  ? t('tabBrowse')
                  : tabId === 'plan'
                    ? t('tabPlan')
                    : tabId === 'scene'
                      ? t('sceneTab')
                      : t('mediaGallery')}
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
          ) : tab === 'media' ? (
            <MediaPanel
              parcelId={selectedParcel?.address ?? selectedOffChain?.id ?? null}
            />
          ) : tab === 'scene' ? (
            /* Distributed runtime scene resolver (REQ-027 / REQ-028) */
            <div className="atlas-scene">
              <div className="atlas-scene-head">
                <span className="runtime-label">{t('runtimeSceneLabel')}</span>
                {currentManifest ? (
                  <>
                    <span className="runtime-manifest-badge">
                      {currentManifest.region_name} v{currentManifest.version}
                    </span>
                    <span className="runtime-region">{currentRegion}</span>
                    <button
                      className="btn btn-ghost px-2 py-1 text-[10px]"
                      onClick={() => setCurrentManifest(null)}
                      title={t('clearSceneTitle')}
                    >
                      ✕
                    </button>
                  </>
                ) : (
                  <button
                    className="btn btn-ghost px-2 py-1 text-[10px]"
                    onClick={createManifestForRegion}
                    title={t('createManifestTitle')}
                    disabled={sceneResolving}
                  >
                    {t('createManifestAction')}
                  </button>
                )}
                {sceneResolving && <span className="runtime-dots">…</span>}
              </div>
              {sceneError && (
                <div className="runtime-error">
                  <span className="runtime-error-icon">⚠</span>
                  {sceneError}
                </div>
              )}
              {sceneGraph ? (
                <div className="runtime-nodes">
                  <span className="runtime-nodes-label">
                    {t('sceneNodesLabel', {
                      n: sceneGraph.nodes.length,
                      lod: sceneGraph.scene.lod_level,
                    })}
                  </span>
                  <div className="runtime-node-list">
                    {sceneGraph.nodes.map((node) => (
                      <div key={node.id} className="runtime-node-row">
                        <span className="runtime-node-id">
                          {node.object_id.slice(0, 12)}…
                        </span>
                        <span className="runtime-node-ver">
                          {node.component_version_id.slice(0, 8)}…
                        </span>
                        <span className={`runtime-node-state runtime-state-${node.loading_state}`}>
                          {node.loading_state}
                        </span>
                        <button
                          className="btn btn-ghost px-1.5 py-0.5 text-[10px]"
                          onClick={() => fetchComponent(node)}
                          disabled={
                            assetFetching === node.component_version_id ||
                            node.loading_state !== 'pending'
                          }
                          title={t('fetchAssetTitle')}
                        >
                          {assetFetching === node.component_version_id ? '…' : t('fetchAction')}
                        </button>
                      </div>
                    ))}
                  </div>
                  {fetchedAsset && (
                    <div className="runtime-fetched">
                      {t('assetFetchedLabel', {
                        id: fetchedAsset.componentId.slice(0, 12),
                        ver: fetchedAsset.componentVersion.slice(0, 8),
                        size: fetchedAsset.sizeBytes,
                      })}
                    </div>
                  )}
                </div>
              ) : (
                !sceneResolving && <p className="atlas-scene-empty">{t('sceneEmpty')}</p>
              )}
            </div>
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
                <span className="text-[12px] text-muted">{t('selectedParcel')}</span>
                <button
                  className="btn btn-ghost p-1"
                  onClick={() => {
                    selectParcel(null)
                    setTab('browse')
                  }}
                  title={t('close')}
                >
                  ✕
                </button>
              </div>
              <ParcelPanel
                address={selectedParcel.address}
                account={selectedParcel.account}
                holder={selectedParcel.holder}
                onView3D={viewSelectedOff3D ?? undefined}
                onFrontView={
                  linkedOffParcel ? () => frontViewOf(linkedOffParcel) : undefined
                }
                onStreetView={
                  linkedOffParcel ? () => streetViewOf(linkedOffParcel) : undefined
                }
              />
              <SpatialHistoryPanel
                parcelPubkey={selectedParcel.address}
                onOpenModel={(uri) =>
                  setTilesetOverride({ parcelId: selectedParcel.address, uri })
                }
              />
              {selectedSummary?.off && (
                <div className="px-3 pb-3 text-[12px] text-muted">
                  {t('offChainRecord')} <b>{selectedSummary.off.status}</b> ·{' '}
                  {selectedSummary.off.area_m2 != null
                    ? `${selectedSummary.off.area_m2.toFixed(0)} m²`
                    : t('noArea')}
                </div>
              )}
            </>
          ) : selectedOffChain ? (
            <>
              <div className="flex items-center justify-between px-3 pt-2">
                <span className="text-[12px] text-muted">{t('selectedMapParcel')}</span>
                <button
                  className="btn btn-ghost p-1"
                  onClick={() => selectOffChain(null)}
                  title={t('close')}
                >
                  ✕
                </button>
              </div>
              <OffChainParcelPanel
                parcel={selectedOffChain}
                onView3D={viewSelectedOff3D ?? undefined}
                onFrontView={
                  selectedOffChain ? () => frontViewOf(selectedOffChain) : undefined
                }
                onStreetView={
                  selectedOffChain ? () => streetViewOf(selectedOffChain) : undefined
                }
              />
            </>
          ) : selectedBuilding ? (
            <BuildingPanel
              footprint={selectedBuilding}
              onView3D={viewBuilding3D}
              onFrontView={() => viewBuildingClose('front')}
              onStreetView={() => viewBuildingClose('street')}
              onClose={() => setSelectedBuilding(null)}
            />
          ) : (
            <ParcelListPanel
              onSelect={onSelectParcel}
              onSelectOffChain={onSelectOffChain}
              buildings={DEMO_BUILDINGS}
              onSelectBuilding={(building) => focusDemoBuilding(building.osm_id)}
              selectedBuildingId={null}
            />
          )}
        </div>

      </Sidenav>
    </div>
  )
}
