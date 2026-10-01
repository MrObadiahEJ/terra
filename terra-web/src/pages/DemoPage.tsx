import { useCallback, useEffect, useRef, useState } from 'react'
import { Link } from 'react-router-dom'
import {
  Activity,
  AlertTriangle,
  Box,
  ChevronRight,
  Fingerprint,
  ListChecks,
  Map as MapIcon,
  Pause,
  Play,
  RotateCcw,
  ShieldCheck,
} from 'lucide-react'
import {
  api,
  type DemoScenarioInfo,
  type DemoScenarioResult,
  type OffChainParcel,
  type OsmBuildingFootprint,
  parseGeoJSON,
} from '../lib/api'
import { useActivityStore } from '../lib/activityStore'
import ActivityFeed from '../components/ActivityFeed'
import { buildLocalDemoScenario } from '../lib/localDemo'
import TerraGlobe from '../components/map/TerraGlobe'
import { DEMO_PARCELS } from '../lib/demoData'
import { polygonAreaM2, polygonCentroid, type LonLat } from '../lib/geo'
import { DEFAULT_FOCUS } from '../lib/constants'
import { useLocale } from '../lib/locale'

// Demo scenario player (B5/B6 frontend) — renders a deterministic scenario
// from POST /api/v1/demo/scenarios/* as an animated event timeline.
// Everything on this page is DEMO STATE (payload carries demo:true) and is
// labelled as such — it is never presented as live chain state.

const FALLBACK: DemoScenarioInfo[] = [
  { name: 'land-registration', description: 'parcel → ownership right → spatial asset → geometry v0', method: 'POST', path: '/demo/scenarios/land-registration' },
  { name: 'verification', description: 'claim → evidence → observation → task → routing → quorum → verified', method: 'POST', path: '/demo/scenarios/verification' },
  { name: 'cross-border', description: 'jurisdictions → identity binding → membership → cross-border verification', method: 'POST', path: '/demo/scenarios/cross-border' },
  { name: 'dispute', description: 'dispute → freeze → adjudication → judgment → execution', method: 'POST', path: '/demo/scenarios/dispute' },
  { name: 'validator-routing', description: 'requirement → eligibility gates → deterministic winner + random gate', method: 'POST', path: '/demo/scenarios/validator-routing' },
  { name: 'spatial-update', description: 'canonical v1 → new observation → candidate → quorum → v2 appended', method: 'POST', path: '/demo/scenarios/spatial-update' },
  { name: 'fraud-review', description: 'fraud report → committee → votes → finalize → restriction', method: 'POST', path: '/demo/scenarios/fraud-review' },
  { name: 'recovery', description: 'heartbeats → validator offline → quorum check → injection → restored', method: 'POST', path: '/demo/scenarios/recovery' },
]

const ENVELOPE_KEYS = new Set([
  'demo',
  'deterministic',
  'scenario',
  'seed',
  'note',
  'program',
  'events',
  'result',
  'source',
])

type DetState = 'idle' | 'checking' | 'same' | 'diff' | 'error'
type ScenarioSource = 'api' | 'local'
type PolygonGeometry = { type: string; coordinates: number[][][] }

function parcelRing(parcel: OffChainParcel | null): LonLat[] {
  const geometry = parseGeoJSON<PolygonGeometry>(parcel?.geometry)
  if (!geometry || geometry.type !== 'Polygon' || !geometry.coordinates[0]) return []
  return geometry.coordinates[0].map(([lon, lat]) => [lon, lat] as LonLat)
}

function parcelFocus(parcel: OffChainParcel | null) {
  const ring = parcelRing(parcel)
  if (ring.length < 3) return null
  const [longitude, latitude] = polygonCentroid(ring)
  const footprintHeight = Math.sqrt(Math.max(1, parcel?.area_m2 ?? 400)) * 4
  return { longitude, latitude, height: Math.max(55, Math.min(180, footprintHeight)) }
}

function footprintsAsParcels(footprints: OsmBuildingFootprint[]): OffChainParcel[] {
  return footprints.map((footprint) => ({
    id: `osm-building-${footprint.osm_id}`,
    name: footprint.name ?? `OSM ${footprint.building} · ${footprint.osm_id}`,
    holder: 'openstreetmap-contributor-data',
    status: 'mapped building footprint',
    geometry: JSON.stringify(footprint.geometry),
    area_m2: polygonAreaM2(footprint.geometry.coordinates[0].map(([lon, lat]) => [lon, lat] as LonLat)),
    created_at: '',
    updated_at: '',
  }))
}

function shuffle<T>(items: T[]): T[] {
  const result = [...items]
  for (let i = result.length - 1; i > 0; i -= 1) {
    const j = Math.floor(Math.random() * (i + 1))
    ;[result[i], result[j]] = [result[j], result[i]]
  }
  return result
}

async function executeScenario(name: string, seed: string) {
  try {
    return { data: await api.runDemoScenario(name, seed), source: 'api' as const, reason: null }
  } catch (error) {
    const data = buildLocalDemoScenario(name, seed)
    return {
      data,
      source: 'local' as const,
      reason: error instanceof Error ? error.message : String(error),
    }
  }
}

export default function DemoPage() {
  const { t } = useLocale()
  const [catalogue, setCatalogue] = useState<DemoScenarioInfo[]>(FALLBACK)
  const [catalogueSource, setCatalogueSource] = useState<'checking' | 'api' | 'local'>('checking')
  const [sel, setSel] = useState(
    () => new URLSearchParams(window.location.search).get('scenario') ?? 'verification',
  )
  const [seed, setSeed] = useState(
    () => new URLSearchParams(window.location.search).get('seed') ?? 'default',
  )
  const [data, setData] = useState<DemoScenarioResult | null>(null)
  const [phase, setPhase] = useState(-1)
  const [playing, setPlaying] = useState(false)
  const [busy, setBusy] = useState(false)
  const [err, setErr] = useState<string | null>(null)
  const [det, setDet] = useState<DetState>('idle')
  const [scenarioSource, setScenarioSource] = useState<ScenarioSource | null>(null)
  const [offlineReason, setOfflineReason] = useState<string | null>(null)
  const [speed, setSpeed] = useState(2)
  const [runId, setRunId] = useState(0)
  const [sceneParcels, setSceneParcels] = useState<OffChainParcel[]>(DEMO_PARCELS)
  const [selectedSceneParcelId, setSelectedSceneParcelId] = useState(DEMO_PARCELS[0].id)
  const [sceneSource, setSceneSource] = useState<'loading' | 'osm' | 'api' | 'sample'>('loading')
  const [sceneView, setSceneView] = useState<'2d' | '3d'>('3d')
  const [sceneWebglError, setSceneWebglError] = useState<string | null>(null)
  const tlRef = useRef<HTMLDivElement | null>(null)

  useEffect(() => {
    let live = true
    api
      .listDemoScenarios()
      .then((r) => {
        if (!live) return
        if (r.scenarios?.length) setCatalogue(r.scenarios)
        setCatalogueSource('api')
      })
      .catch(() => {
        if (live) setCatalogueSource('local')
      })
    return () => {
      live = false
    }
  }, [])

  const selectedSceneParcel = sceneParcels.find((parcel) => parcel.id === selectedSceneParcelId) ?? sceneParcels[0] ?? null
  const sceneFocus = parcelFocus(selectedSceneParcel) ?? DEFAULT_FOCUS
  const includesValidatorReview = ['verification', 'validator-routing', 'spatial-update', 'fraud-review', 'recovery']
    .includes(data?.scenario ?? '')

  useEffect(() => {
    let live = true
    const loadFallbackParcels = async () => {
      try {
        const parcels = await api.listParcels()
        if (!live) return
        const usable = parcels.filter((parcel) => parcelRing(parcel).length >= 3)
        if (usable.length) {
          setSceneParcels(usable)
          setSelectedSceneParcelId(usable[0].id)
          setSceneSource('api')
          return
        }
      } catch {
        if (!live) return
      }
      if (live) {
        setSceneParcels(DEMO_PARCELS)
        setSelectedSceneParcelId(DEMO_PARCELS[0].id)
        setSceneSource('sample')
      }
    }
    const loadFootprints = async () => {
      try {
        const stats = await api.geoStats()
        const total = stats.building_footprints ?? 0
        const offset = total > 500 ? Math.floor(Math.random() * (total - 500)) : 0
        const footprints = await api.osmBuildingFootprints(500, offset)
        if (!live) return
        const usable = shuffle(footprints).slice(0, 36)
        if (usable.length) {
          const mapped = footprintsAsParcels(usable)
          setSceneParcels(mapped)
          setSelectedSceneParcelId(mapped[0].id)
          setSceneSource('osm')
          return
        }
      } catch {
        if (!live) return
      }
      await loadFallbackParcels()
    }
    void loadFootprints()
    return () => {
      live = false
    }
  }, [])

  const events = data?.events ?? []
  const last = events.length - 1

  // Playback: reveal events using the payload's own deterministic timestamps.
  useEffect(() => {
    if (!playing || !data) return
    if (phase >= data.events.length - 1) {
      const stop = setTimeout(() => setPlaying(false), 0)
      return () => clearTimeout(stop)
    }
    const prev = data.events[Math.max(phase, 0)].t_offset_ms
    const next = data.events[phase + 1].t_offset_ms
    const delay = Math.min(Math.max((next - prev) / speed, 140), 1400)
    const id = setTimeout(() => setPhase((p) => p + 1), delay)
    return () => clearTimeout(id)
  }, [playing, phase, data, speed])

  // Keep the newest revealed event in view.
  useEffect(() => {
    const el = tlRef.current
    if (el) el.scrollTop = el.scrollHeight
  }, [phase])

  // Feed (B4): stream each revealed event into the global activity feed.
  // Ids are run-scoped, so StrictMode double-effects and replays dedupe.
  useEffect(() => {
    if (!data || phase < 0) return
    const e = data.events[phase]
    useActivityStore.getState().push({
      id: `${data.scenario}:${data.seed}:${runId}:${e.seq}`,
      at: Date.now(),
      kind: e.kind.toLowerCase(),
      summary: e.detail,
      source: 'demo',
      link: `/demo?scenario=${data.scenario}&seed=${encodeURIComponent(data.seed)}`,
    })
  }, [phase, data, runId])

  const run = useCallback(async (name?: string, sd?: string) => {
    const scenario = name ?? sel
    const seedVal = (sd ?? seed) || undefined
    setBusy(true)
    setErr(null)
    setDet('idle')
    try {
      const result = await executeScenario(scenario, seedVal ?? 'default')
      setData(result.data)
      setScenarioSource(result.source)
      setOfflineReason(result.reason)
      setPhase(0)
      setRunId((n) => n + 1)
      setPlaying(result.data.events.length > 1)
    } catch (e) {
      setData(null)
      setPlaying(false)
      setScenarioSource(null)
      setOfflineReason(null)
      setErr(e instanceof Error ? e.message : String(e))
    } finally {
      setBusy(false)
    }
  }, [sel, seed])

  // Start the walkthrough on arrival; query parameters select a specific replay.
  const autoRan = useRef(false)
  useEffect(() => {
    if (autoRan.current) return
    autoRan.current = true
    const q = new URLSearchParams(window.location.search)
    const name = q.get('scenario') ?? 'verification'
    const sd = q.get('seed') ?? undefined
    // Deliberately no cleanup: StrictMode runs setup→cleanup→setup, and the
    // autoRan guard means only the first setup schedules this timeout — a
    // clearTimeout here would cancel the only run. The 0ms timer fires once.
    setTimeout(() => {
      void run(name, sd)
    }, 0)
  }, [run])

  const checkDeterminism = async () => {
    setDet('checking')
    try {
      const [a, b] = await Promise.all([
        executeScenario(sel, seed || 'default'),
        executeScenario(sel, seed || 'default'),
      ])
      setDet(a.source === b.source && JSON.stringify(a.data) === JSON.stringify(b.data) ? 'same' : 'diff')
      setScenarioSource(a.source)
      setOfflineReason(a.reason)
    } catch {
      setDet('error')
    }
  }

  const step = () => {
    setPlaying(false)
    setPhase((p) => Math.min(p + 1, last))
  }

  const restart = () => {
    setPlaying(false)
    setPhase(0)
  }

  const togglePlay = () => {
    if (!data) return
    if (phase >= last) setPhase(0)
    setPlaying((p) => !p)
  }

  const entities = data
    ? Object.entries(data).filter(([k]) => !ENVELOPE_KEYS.has(k))
    : []

  const badge: { cls: string; text: string } | null = {
    idle: null,
    checking: { cls: 'lab-badge-info', text: 'checking…' },
    same: { cls: 'lab-badge-ok', text: 'byte-identical ✓' },
    diff: { cls: 'lab-badge-err', text: 'MISMATCH ✗' },
    error: { cls: 'lab-badge-err', text: 'API unreachable' },
  }[det]

  return (
    <div className="st-page dm-page">
      <header className="land-head">
        <div className="min-w-0">
          <h1 className="land-title">
            <Activity size={18} /> Terra — {t('scenarioPlayer')}
          </h1>
          <p className="text-xs text-muted max-w-3xl">{t('demoIntro')}</p>
        </div>
        <div className="land-badges">
          <span className="lab-badge lab-badge-warn">{t('demoState')}</span>
          <Link className="btn btn-secondary px-2 py-1" to="/status">
            {t('engineStatus')} ↗
          </Link>
        </div>
      </header>

      {/* scenario picker */}
      <div className="dm-row">
        {catalogue.map((s) => (
          <button
            key={s.name}
            className={`dm-chip ${sel === s.name ? 'on' : ''}`}
            onClick={() => setSel(s.name)}
            title={s.description}
          >
            {s.name}
          </button>
        ))}
      </div>
      <div className="dm-catalogue-status">
        <span className={`lab-badge ${catalogueSource === 'local' ? 'lab-badge-warn' : 'lab-badge-info'}`}>
          {catalogueSource === 'checking' ? 'CHECKING SCENARIO API' : catalogueSource === 'api' ? 'API CATALOGUE' : 'BUILT-IN CATALOGUE'}
        </span>
        <span className="text-xs text-muted">
          {catalogueSource === 'local' ? 'The scenario API is unavailable; previews still run in this browser.' : 'Seeded demos are reproducible. No wallet approval or chain write is required.'}
        </span>
      </div>
      <p className="text-xs text-muted mb-2">
        {catalogue.find((s) => s.name === sel)?.description}
      </p>

      {/* controls */}
      <div className="dm-row">
        <label className="text-xs text-muted" htmlFor="dm-seed">
          seed
        </label>
        <input
          id="dm-seed"
          className="dm-input"
          value={seed}
          spellCheck={false}
          onChange={(e) => setSeed(e.target.value)}
        />
        <button className="btn btn-primary px-3 py-1.5" onClick={() => void run()} disabled={busy}>
          {busy ? 'Running…' : 'Run scenario'}
        </button>
        <button
          className="btn btn-secondary px-3 py-1.5 gap-1"
          onClick={checkDeterminism}
          disabled={busy || det === 'checking'}
        >
          <Fingerprint size={13} /> Verify determinism
        </button>
        {badge && (
          <span className={`lab-badge ${badge.cls}`}>{badge.text}</span>
        )}
        <span className="text-xs text-muted">speed</span>
        <div className="land-viewer-seg">
          {[1, 2, 4].map((s) => (
            <button
              key={s}
              className={`land-viewer-segbtn ${speed === s ? 'on' : ''}`}
              onClick={() => setSpeed(s)}
            >
              {s}×
            </button>
          ))}
        </div>
      </div>

      {err && (
        <div className="dm-banner dm-banner-err">
          <AlertTriangle size={14} /> {err} — is the API running on :8080?
        </div>
      )}

      {data && (
        <>
          {/* result strip */}
          <div className="dm-result">
            <span className="lab-badge lab-badge-ok">{data.result}</span>
            <span className={`lab-badge ${scenarioSource === 'local' ? 'lab-badge-warn' : 'lab-badge-info'}`}>
              {scenarioSource === 'local' ? 'LOCAL PREVIEW' : 'API ENGINE'}
            </span>
            <span className="text-xs text-muted">
              <ListChecks size={12} className="dm-inline" /> {data.scenario} · seed{' '}
              <span className="font-mono">{data.seed}</span> · {events.length} events ·{' '}
              {Math.round(events[last]?.t_offset_ms ?? 0)} ms
            </span>
            <span className="dm-ctrls">
              <button
                className="btn btn-ghost px-2 py-1 gap-1"
                onClick={togglePlay}
                disabled={events.length < 2}
              >
                {playing ? <Pause size={13} /> : <Play size={13} />}
                {playing ? 'Pause' : phase >= last ? 'Replay' : 'Play'}
              </button>
              <button
                className="btn btn-ghost px-2 py-1 gap-1"
                onClick={step}
                disabled={playing || phase >= last}
              >
                <ChevronRight size={13} /> Step
              </button>
              <button
                className="btn btn-ghost px-2 py-1 gap-1"
                onClick={restart}
                disabled={phase <= 0 && !playing}
              >
                <RotateCcw size={13} /> Restart
              </button>
            </span>
          </div>

          {offlineReason && (
            <div className="dm-banner dm-banner-offline">
              <AlertTriangle size={14} />
              <span>Scenario API unavailable ({offlineReason}). This repeatable browser-only preview keeps the demo interactive; it does not write chain state.</span>
            </div>
          )}

          <section className="dm-spatial-grid" aria-label="Spatial parcel and validator replay">
            <div className="dm-scene-panel">
              <div className="dm-panel-h">
                <span><MapIcon size={14} /> Parcel in context</span>
                <div className="dm-scene-actions">
                  {sceneParcels.length > 1 && (
                    <select
                      className="dm-scene-select"
                      value={selectedSceneParcel?.id ?? ''}
                      onChange={(event) => setSelectedSceneParcelId(event.target.value)}
                      aria-label="Choose a parcel for the demo scene"
                    >
                      {sceneParcels.map((parcel) => <option key={parcel.id} value={parcel.id}>{parcel.name}</option>)}
                    </select>
                  )}
                  <div className="land-viewer-seg" role="group" aria-label="Parcel map view">
                    <button className={`land-viewer-segbtn ${sceneView === '2d' ? 'on' : ''}`} onClick={() => setSceneView('2d')} aria-pressed={sceneView === '2d'}><MapIcon size={12} /> 2D</button>
                    <button className={`land-viewer-segbtn ${sceneView === '3d' ? 'on' : ''}`} onClick={() => setSceneView('3d')} aria-pressed={sceneView === '3d'} disabled={Boolean(sceneWebglError)} title={sceneWebglError ? '3D view requires WebGL' : '3D globe view'}><Box size={12} /> 3D</button>
                  </div>
                </div>
              </div>
              <div className="dm-scene-map">
                {sceneSource === 'loading' ? (
                  <div className="dm-scene-placeholder">Loading parcel geometry…</div>
                ) : (
                  <TerraGlobe
                    offChainParcels={sceneParcels}
                    roads={[]}
                    pois={[]}
                    drawing={false}
                    drawVertices={[]}
                    onDrawVertexAdd={() => undefined}
                    onDrawFinish={() => undefined}
                    onParcelClick={setSelectedSceneParcelId}
                    viewMode={sceneView}
                    focus={sceneFocus}
                    basemap="imagery"
                    onWebGLStatus={(error) => {
                      setSceneWebglError(error)
                      if (error) setSceneView('2d')
                    }}
                  />
                )}
                <span className={`dm-scene-source ${sceneSource === 'sample' ? 'sample' : ''}`}>
                  {sceneSource === 'loading'
                    ? t('loadingMap')
                    : sceneSource === 'osm'
                      ? t('osmBuilding')
                      : sceneSource === 'sample'
                        ? t('sampleLot')
                        : t('registeredGeometry')}
                </span>
                {sceneWebglError && <span className="dm-scene-fallback">3D unavailable — showing 2D map</span>}
              </div>
              <div className="dm-scene-foot">
                <strong>{selectedSceneParcel?.name ?? t('parcelContext')}</strong>
                <span>{selectedSceneParcel?.area_m2 ? `${(selectedSceneParcel.area_m2 / 10_000).toFixed(2)} ha` : 'Area not recorded'}</span>
                <span>{sceneSource === 'osm' ? t('imageryContext') : sceneSource === 'sample' ? t('sampleLot') : t('noMapOverlays')}</span>
              </div>
            </div>

            <aside className="dm-review-panel">
              <div className="dm-panel-h"><span><ShieldCheck size={14} /> {includesValidatorReview ? t('validatorReplay') : t('protocolReplay')}</span><span className="dm-sim-badge">{t('simulated')}</span></div>
              <div className="dm-review-summary">
                <span className={`dm-review-indicator ${phase >= 0 ? 'running' : ''}`} />
                <div><strong>{phase < 0 ? t('awaitingScenario') : events[phase]?.kind.replaceAll('_', ' ')}</strong><small>{phase < 0 ? t('startPlayback') : events[phase]?.detail}</small></div>
              </div>
              {includesValidatorReview ? (
                <>
                  <div className="dm-reviewers">
                    {['Reviewer 01', 'Reviewer 02', 'Reviewer 03'].map((reviewer, index) => {
                      const kind = phase >= 0 ? events[phase]?.kind ?? '' : ''
                      const hasReviewStarted = phase >= 0 && phase >= index + 1
                      const decisionReady = kind.includes('QUORUM') || kind.includes('VERIFIED') || kind.includes('FINALIZED')
                      const selectedWinner = data.scenario === 'validator-routing' && kind === 'WINNER_SELECTED'
                      const isNotSelected = selectedWinner && index > 0
                      const settled = decisionReady || (selectedWinner && index === 0)
                      return (
                        <div className="dm-reviewer" key={reviewer}>
                          <span className={`dm-review-avatar ${settled ? 'approved' : hasReviewStarted ? 'checking' : ''}`}>{String(index + 1).padStart(2, '0')}</span>
                          <span><strong>{t((['reviewerOne', 'reviewerTwo', 'reviewerThree'] as const)[index])}</strong><small>{decisionReady ? t('reviewRecorded') : selectedWinner && index === 0 ? t('seededWinner') : isNotSelected ? t('notSelected') : hasReviewStarted ? t('checkingGates') : t('inQueue')}</small></span>
                          <i className={settled ? 'approved' : hasReviewStarted ? 'checking' : ''} />
                        </div>
                      )
                    })}
                  </div>
                  <div className="dm-review-foot"><Fingerprint size={13} /> {t('seededReplay')}</div>
                </>
              ) : (
                <div className="dm-review-no-votes">{t('noVotes')}</div>
              )}
              <ActivityFeed pollMs={5000} />
            </aside>
          </section>

          <div className="dm-grid">
            {/* event timeline */}
            <div className="dm-panel">
              <div className="dm-panel-h">
                Event timeline <span className="text-muted">(deterministic t offsets)</span>
                <span className="dm-progress">
                  <span
                    className="dm-progress-bar"
                    style={{ width: `${((phase + 1) / events.length) * 100}%` }}
                  />
                </span>
              </div>
              <div className="dm-tl" ref={tlRef}>
                {events.map((e, i) =>
                  i <= phase ? (
                    <div
                      key={e.seq}
                      className={`dm-ev ${i === phase ? 'cur' : ''}`}
                    >
                      <span className="dm-ev-seq">{e.seq}</span>
                      <div className="min-w-0">
                        <div className="dm-kind">{e.kind}</div>
                        <div className="text-xs text-muted">{e.detail}</div>
                        <div className="dm-meta font-mono">
                          t+{(e.t_offset_ms / 1000).toFixed(2)}s · {e.ref.slice(0, 16)}…
                        </div>
                      </div>
                    </div>
                  ) : null,
                )}
                {phase < 0 && <div className="text-xs text-muted">Press Run…</div>}
              </div>
            </div>

            {/* entities from the payload */}
            <div className="dm-panel">
              <div className="dm-panel-h">Scenario entities</div>
              <div className="dm-ents">
                {entities.map(([k, v]) => (
                  <div key={k} className="dm-ent">
                    <span className="dm-ent-k">{k}</span>
                    {typeof v === 'string' ? (
                      <span className="dm-ent-v font-mono">{v}</span>
                    ) : (
                      <pre className="dm-pre">{JSON.stringify(v, null, 2)}</pre>
                    )}
                  </div>
                ))}
              </div>
              <div className="text-xs text-muted mt-3 font-mono">{data.note}</div>
            </div>
          </div>
        </>
      )}

      {!data && !err && (
        <div className="dm-panel text-xs text-muted">
        The verification walkthrough starts automatically. Choose another scenario or seed
        above, then use <strong>Run scenario</strong> to replay it. The API or deterministic
        local preview returns entities, ordered events and a result. No wallet, chain writes
        or unrepeatable randomness.
        </div>
      )}
    </div>
  )
}
