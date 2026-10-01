import { useCallback, useEffect, useRef, useState } from 'react'
import { Link } from 'react-router-dom'
import {
  Activity,
  AlertTriangle,
  ChevronRight,
  Fingerprint,
  ListChecks,
  Pause,
  Play,
  RotateCcw,
} from 'lucide-react'
import {
  api,
  type DemoScenarioInfo,
  type DemoScenarioResult,
} from '../lib/api'

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
])

type DetState = 'idle' | 'checking' | 'same' | 'diff' | 'error'

export default function DemoPage() {
  const [catalogue, setCatalogue] = useState<DemoScenarioInfo[]>(FALLBACK)
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
  const [speed, setSpeed] = useState(2)
  const tlRef = useRef<HTMLDivElement | null>(null)

  useEffect(() => {
    let live = true
    api
      .listDemoScenarios()
      .then((r) => {
        if (live && r.scenarios?.length) setCatalogue(r.scenarios)
      })
      .catch(() => {
        /* API down — keep the fallback catalogue so the page stays usable */
      })
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

  const run = useCallback(async (name?: string, sd?: string) => {
    const scenario = name ?? sel
    const seedVal = (sd ?? seed) || undefined
    setBusy(true)
    setErr(null)
    setDet('idle')
    try {
      const r = await api.runDemoScenario(scenario, seedVal)
      setData(r)
      setPhase(0)
      setPlaying(r.events.length > 1)
    } catch (e) {
      setData(null)
      setPlaying(false)
      setErr(e instanceof Error ? e.message : String(e))
    } finally {
      setBusy(false)
    }
  }, [sel, seed])

  // Deep link: /demo?scenario=verification&seed=… auto-runs once on mount.
  const autoRan = useRef(false)
  useEffect(() => {
    if (autoRan.current) return
    autoRan.current = true
    const q = new URLSearchParams(window.location.search)
    const name = q.get('scenario')
    if (!name) return
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
      const a = await api.runDemoScenario(sel, seed || undefined)
      const b = await api.runDemoScenario(sel, seed || undefined)
      setDet(JSON.stringify(a) === JSON.stringify(b) ? 'same' : 'diff')
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
    <div className="st-page">
      <header className="land-head">
        <div className="min-w-0">
          <h1 className="land-title">
            <Activity size={18} /> Terra — Demo scenario player
          </h1>
          <p className="text-xs text-muted max-w-3xl">
            Deterministic, seeded previews served by{' '}
            <span className="font-mono">POST /api/v1/demo/scenarios/*</span>. The same
            scenario + seed always produces a byte-identical payload — hit{' '}
            <strong>Verify determinism</strong> to prove it. Event kinds mirror
            on-chain instruction names 1:1.
          </p>
        </div>
        <div className="land-badges">
          <span className="lab-badge lab-badge-warn">demo state — not chain state</span>
          <Link className="btn btn-secondary px-2 py-1" to="/status">
            Engine status ↗
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
          Pick a scenario, optionally change the seed, then <strong>Run scenario</strong> —
          the endpoint returns a full deterministic story (entities + ordered events +
          result) in one round-trip. No wallet, no chain, no randomness you cannot
          reproduce.
        </div>
      )}
    </div>
  )
}
