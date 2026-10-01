import { useCallback, useEffect, useMemo, useState } from 'react'
import { Link } from 'react-router-dom'
import {
  admitBps,
  drawBps,
  evaluateEligibility,
  passesRandomGate,
  routeSeed,
  selectWinner,
  AVAILABILITIES,
  CAPABILITY_CODES,
  CAPABILITY_LEVELS,
  MAX_REPUTATION,
  TIERS,
  type Eligibility,
  type GateResult,
} from '../lib/routingSim'
import { api, type AuthorityRegistry, type Jurisdiction } from '../lib/api'
import {
  DEMO_BLOCKHASH,
  DEMO_INITIAL_SLOT,
  DEMO_TASKS,
  DEMO_VALIDATORS,
  bindingActive,
  type DemoValidator,
} from '../lib/validatorDemo'
import { Activity, Building2, Globe2, MapPin, RotateCw } from 'lucide-react'
import { useLocale } from '../lib/locale'

// Validator network (B3) — demo dataset + a faithful JS mirror of routing.rs.
// Every ✓/✗ below runs the same predicates as route_task (availability →
// tier → reputation → capability → jurisdiction → geo → random gate → winner).
// No /validators API exists yet (PLANNED on /status) — dataset is deterministic.

const GATE_KEYS = ['availability', 'tier', 'reputation', 'capability', 'jurisdiction', 'geo'] as const
type GateKey = (typeof GATE_KEYS)[number]
type ProfileResults = [
  PromiseSettledResult<Jurisdiction[]>,
  PromiseSettledResult<AuthorityRegistry[]>,
]

function fetchNetworkProfiles(): Promise<ProfileResults> {
  return Promise.allSettled([api.listJurisdictions(), api.listRegistries()])
}
const GATE_LABELS: Record<GateKey, string> = {
  availability: 'avail',
  tier: 'tier',
  reputation: 'rep',
  capability: 'cap',
  jurisdiction: 'jur',
  geo: 'geo',
}

// Fixed clock for the whole session — react-hooks/purity forbids Date.now() in render.
const NOW_S = Math.floor(Date.now() / 1000)

const PW = 680
const PH = 340

function availColor(status: number): string {
  if (status === 0 || status === 1) return '#10b981'
  if (status === 2) return '#f59e0b'
  if (status === 5) return '#ef4444'
  return '#94a3b8'
}

function project(latE7: number, lonE7: number): { x: number; y: number } {
  return {
    x: ((lonE7 / 1e7 + 180) / 360) * PW,
    y: ((90 - latE7 / 1e7) / 180) * PH,
  }
}

/** Destination point on a sphere — for drawing the task radius ring. */
function destination(latE7: number, lonE7: number, bearingDeg: number, distM: number): { x: number; y: number } {
  const R = 6_371_000
  const lat = (latE7 / 1e7) * (Math.PI / 180)
  const lon = (lonE7 / 1e7) * (Math.PI / 180)
  const brg = (bearingDeg * Math.PI) / 180
  const ang = distM / R
  const lat2 = Math.asin(Math.sin(lat) * Math.cos(ang) + Math.cos(lat) * Math.sin(ang) * Math.cos(brg))
  const lon2 =
    lon + Math.atan2(Math.sin(brg) * Math.sin(ang) * Math.cos(lat), Math.cos(ang) - Math.sin(lat) * Math.sin(lat2))
  return project(Math.round(lat2 * 1e7), Math.round((lon2 * 180) / Math.PI * 1e7))
}

interface Row {
  v: DemoValidator
  e: Eligibility
}

export default function NetworkPage() {
  const { t } = useLocale()
  const [taskId, setTaskId] = useState(DEMO_TASKS[0].id)
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [slot, setSlot] = useState(DEMO_INITIAL_SLOT)
  const [jurisdictions, setJurisdictions] = useState<Jurisdiction[]>([])
  const [registries, setRegistries] = useState<AuthorityRegistry[]>([])
  const [profilesLoading, setProfilesLoading] = useState(true)
  const [profilesError, setProfilesError] = useState<string | null>(null)

  const applyProfiles = useCallback(([jurisdictionResult, registryResult]: ProfileResults) => {
    setJurisdictions(jurisdictionResult.status === 'fulfilled' ? jurisdictionResult.value : [])
    setRegistries(registryResult.status === 'fulfilled' ? registryResult.value : [])
    const errors = [jurisdictionResult, registryResult]
      .filter((result): result is PromiseRejectedResult => result.status === 'rejected')
      .map((result) => result.reason instanceof Error ? result.reason.message : String(result.reason))
    setProfilesError(errors.length ? errors.join(' · ') : null)
    setProfilesLoading(false)
  }, [])

  const loadProfiles = useCallback(async () => {
    setProfilesLoading(true)
    setProfilesError(null)
    applyProfiles(await fetchNetworkProfiles())
  }, [applyProfiles])

  useEffect(() => {
    let live = true
    fetchNetworkProfiles().then((results) => {
      if (live) applyProfiles(results)
    })
    return () => {
      live = false
    }
  }, [applyProfiles])

  const demoOrganizations = useMemo(() => {
    const countries = new Map<string, { countryName: string; organizations: string[] }>()
    for (const validator of DEMO_VALIDATORS) {
      const country = countries.get(validator.country) ?? {
        countryName: validator.countryName,
        organizations: [],
      }
      country.organizations.push(validator.name)
      countries.set(validator.country, country)
    }
    return [...countries.entries()].map(([countryCode, value]) => ({ countryCode, ...value }))
  }, [])

  const now = NOW_S
  const task = useMemo(() => DEMO_TASKS.find((t) => t.id === taskId) ?? DEMO_TASKS[0], [taskId])

  const rows = useMemo<Row[]>(
    () =>
      DEMO_VALIDATORS.map((v) => ({
        v,
        e: evaluateEligibility(
          task,
          {
            tier: v.tier,
            reputation: v.reputation,
            availability: v.availability,
            jurisdiction: v.country,
            capabilities: v.capabilities,
            presence: v.presence,
          },
          bindingActive(task.jurisdiction, v.country),
          now,
        ),
      })),
    [task, now],
  )

  const eligibleIds = useMemo(() => rows.filter((r) => r.e.eligible).map((r) => r.v.id), [rows])
  const competitors = Math.max(1, eligibleIds.length)
  const seed = routeSeed(task.id, DEMO_BLOCKHASH, slot)
  const winnerId = selectWinner(seed, eligibleIds)
  const winnerDraw = winnerId ? drawBps(seed, winnerId) : null
  const gatePass = winnerId !== null && winnerDraw !== null && passesRandomGate(winnerDraw, competitors)

  const detailRow = useMemo(() => {
    return (
      rows.find((r) => r.v.id === selectedId) ??
      rows.find((r) => r.v.id === winnerId) ??
      rows.find((r) => r.e.eligible) ??
      rows[0]
    )
  }, [rows, selectedId, winnerId])

  // presence plot geometry
  const plot = useMemo(() => {
    const lines: { x1: number; y1: number; x2: number; y2: number; strong: boolean }[] = []
    for (let lon = -180; lon <= 180; lon += 30) {
      const { x } = project(0, lon * 1e7)
      lines.push({ x1: x, y1: 0, x2: x, y2: PH, strong: lon === 0 })
    }
    for (let lat = -90; lat <= 90; lat += 30) {
      const { y } = project(lat * 1e7, 0)
      lines.push({ x1: 0, y1: y, x2: PW, y2: y, strong: lat === 0 })
    }
    let radiusPoly = ''
    let centerPt: { x: number; y: number } | null = null
    if (task.radiusM > 0) {
      centerPt = project(task.centerLatE7, task.centerLonE7)
      const pts: string[] = []
      for (let b = 0; b < 360; b += 5) {
        const p = destination(task.centerLatE7, task.centerLonE7, b, task.radiusM)
        pts.push(`${p.x.toFixed(1)},${p.y.toFixed(1)}`)
      }
      radiusPoly = pts.join(' ')
    }
    const dots = rows.map((r) => {
      const p = r.v.presence ? project(r.v.presence.latE7, r.v.presence.lonE7) : null
      return { row: r, p }
    })
    return { lines, radiusPoly, centerPt, dots }
  }, [rows, task])

  const gateCell = (g: GateResult) => (
    <span className={`net-gate ${g.pass ? 'ok' : 'no'}`} title={g.detail}>
      {g.pass ? '✓' : '✗'}
    </span>
  )

  return (
    <div className="net-page">
      <header className="land-head">
        <div className="min-w-0">
          <h1 className="land-title">
            <Activity size={18} /> Terra — {t('networkTitle')}
          </h1>
          <p className="text-[11px] text-muted max-w-3xl">
            Deterministic demo network (no <span className="font-mono">/validators</span> API yet — see{' '}
            <Link to="/status">/status</Link>). The inspector below runs a{' '}
            <strong>faithful JS mirror of routing.rs</strong>: the same six gates in the same order, then
            the deterministic winner pick and random admission gate. Click any row for "why selected".
          </p>
        </div>
        <div className="land-badges">
          <span className="lab-badge lab-badge-info">routing.rs mirror</span>
          <span className="lab-badge lab-badge-warn">DEMO dataset</span>
          <Link className="btn btn-secondary px-2 py-1" to="/lab?tab=pipeline">
            Verification lab ↗
          </Link>
        </div>
      </header>

      <section className="net-directory" aria-labelledby="net-directory-title">
        <div className="net-directory-head">
          <div>
            <span className="welcome-section-kicker">{t('networkDirectory')}</span>
            <h2 id="net-directory-title">{t('networkDirectory')}</h2>
            <p>{t('demoNetworkIntro')}</p>
          </div>
          <button className="btn btn-secondary px-2 py-1" onClick={() => void loadProfiles()} disabled={profilesLoading}>
            <RotateCw size={13} className={profilesLoading ? 'spin' : ''} /> {profilesLoading ? t('connecting') : t('refreshProfiles')}
          </button>
        </div>

        {profilesError && (
          <div className="net-directory-error">{t('profilesUnavailable')} {profilesError}</div>
        )}

        <div className="net-directory-grid">
          <article className="net-directory-panel">
            <div className="net-directory-panel-head">
              <span><Globe2 size={14} /> {t('registeredJurisdictions')}</span>
              <span className="net-directory-count">{jurisdictions.length}</span>
            </div>
            {jurisdictions.length ? jurisdictions.map((profile) => (
              <div className="net-profile-row" key={profile.id}>
                <span className="net-profile-mark">{profile.country_code.slice(0, 2).toUpperCase()}</span>
                <div><strong>{profile.jurisdiction_name}</strong><small>{profile.country_code.toUpperCase()} · {profile.authority || 'Authority not named'}</small></div>
                <span className={`net-profile-status ${profile.status.toLowerCase()}`}>{profile.status}</span>
              </div>
            )) : (
              <div className="net-directory-empty">{profilesLoading ? t('loadingJurisdictions') : profilesError ? t('jurisdictionUnavailable') : t('noJurisdictions')}</div>
            )}
          </article>

          <article className="net-directory-panel">
            <div className="net-directory-panel-head">
              <span><Building2 size={14} /> {t('authorityRegistries')}</span>
              <span className="net-directory-count">{registries.length}</span>
            </div>
            {registries.length ? registries.map((registry) => (
              <div className="net-profile-row" key={registry.pubkey}>
                <span className="net-profile-mark authority"><Building2 size={15} /></span>
                <div><strong>Authority registry</strong><small>{registry.validators.length} validators · {registry.mode === 1 ? 'peer consensus' : 'bootstrap'} · {registry.pubkey.slice(0, 8)}…</small></div>
                <span className="net-profile-status active">registered</span>
              </div>
            )) : (
              <div className="net-directory-empty">{profilesLoading ? t('loadingRegistries') : profilesError ? t('registryUnavailable') : t('noRegistries')}</div>
            )}
          </article>
        </div>

        <div className="net-demo-directory">
          <div className="net-directory-panel-head">
            <span><Activity size={14} /> {t('illustrativeOrgs')}</span>
            <span className="net-demo-tag">{t('demoProfiles')}</span>
          </div>
          <div className="net-demo-country-list">
            {demoOrganizations.map((country) => (
              <div key={country.countryCode}>
                <strong>{country.countryName} <span>{country.countryCode}</span></strong>
                <small>{country.organizations.join(' · ')}</small>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* task selector */}
      <div className="net-tasks">
        {DEMO_TASKS.map((t) => (
          <button
            key={t.id}
            className={`net-task ${t.id === taskId ? 'on' : ''}`}
            onClick={() => {
              setTaskId(t.id)
              setSelectedId(null)
            }}
          >
            <span className="net-task-label">{t.label}</span>
            <span className="net-task-sub">
              cap {t.capabilityCode === 255 ? 'ANY' : CAPABILITY_CODES[t.capabilityCode]} · min_rep{' '}
              {t.minReputation} · {t.jurisdiction ? `jur ${t.jurisdiction}` : 'any jur'}
              {t.radiusM > 0 ? ` · ${(t.radiusM / 1000).toFixed(0)} km` : ''}
            </span>
          </button>
        ))}
      </div>

      {/* requirement + routing result */}
      <div className="net-row">
        <div className="lab-card flex-1">
          <h3 className="text-[13px] font-semibold mb-2">Task requirement (mirrors TaskRequirement)</h3>
          <div className="lab-field-table">
            <div className="lab-field-row">
              <span className="font-mono text-[10px] text-muted">min_tier</span>
              <span className="font-mono text-[10px] col-span-2">
                {task.minTier} ({TIERS[task.minTier]})
              </span>
            </div>
            <div className="lab-field-row">
              <span className="font-mono text-[10px] text-muted">min_reputation</span>
              <span className="font-mono text-[10px] col-span-2">
                {task.minReputation} bps (MAX {MAX_REPUTATION})
              </span>
            </div>
            <div className="lab-field-row">
              <span className="font-mono text-[10px] text-muted">capability_code</span>
              <span className="font-mono text-[10px] col-span-2">
                {task.capabilityCode === 255 ? '255 (CAPABILITY_ANY)' : `${task.capabilityCode} (${CAPABILITY_CODES[task.capabilityCode]})`}
              </span>
            </div>
            <div className="lab-field-row">
              <span className="font-mono text-[10px] text-muted">jurisdiction</span>
              <span className="font-mono text-[10px] col-span-2">
                {task.jurisdiction ? `[${task.jurisdiction.charCodeAt(0)}, ${task.jurisdiction.charCodeAt(1)}] (${task.jurisdiction})` : '[0,0] — any'}
              </span>
            </div>
            <div className="lab-field-row">
              <span className="font-mono text-[10px] text-muted">radius_m / center</span>
              <span className="font-mono text-[10px] col-span-2">
                {task.radiusM.toLocaleString()} · {task.centerLatE7}, {task.centerLonE7} ({task.centerLabel})
              </span>
            </div>
          </div>
          <p className="text-[11px] text-muted mt-2">{task.description}</p>
        </div>

        <div className="lab-card" style={{ minWidth: 320 }}>
          <h3 className="text-[13px] font-semibold mb-2">route_task result</h3>
          <div className="net-flow">
            <span className="net-flow-step">
              {eligibleIds.length} / {rows.length} eligible
            </span>
            <span className="net-flow-arrow">→</span>
            <span className="net-flow-step mono">slot {slot}</span>
            <span className="net-flow-arrow">→</span>
            <span className={`net-flow-step ${gatePass ? 'win' : winnerId ? 'warn' : ''}`}>
              {winnerId ? (gatePass ? `winner ${winnerId}` : 'gate REJECTED') : 'no eligible'}
            </span>
          </div>
          <div className="net-seed mono" title="route_seed(task_id, blockhash, slot) — demo string hash">
            seed = {seed}
          </div>
          {winnerId && (
            <div className={`net-gatebox ${gatePass ? 'ok' : 'no'}`}>
              draw_bps({winnerId}) = {winnerDraw} {gatePass ? '<' : '≥'} admit {admitBps(competitors)} bps
              {' — '}
              {gatePass
                ? 'random gate PASS'
                : 'NotRouteWinner — the random admission gate rejected this draw at this slot'}
            </div>
          )}
          <div className="flex gap-1.5 mt-2 flex-wrap">
            <button className="btn btn-secondary px-2 py-1 gap-1" onClick={() => setSlot((s) => s + 1)}>
              <RotateCw size={12} /> Advance slot (re-roll)
            </button>
            <button className="btn btn-ghost px-2 py-1" onClick={() => setSlot(DEMO_INITIAL_SLOT)}>
              Reset slot
            </button>
          </div>
        </div>
      </div>

      {/* gate matrix */}
      <div className="lab-card">
        <div className="flex items-center justify-between gap-2 flex-wrap mb-2">
          <h3 className="text-[13px] font-semibold">
            Eligibility matrix — is_eligible() per candidate (click a row for details)
          </h3>
          <span className="lab-badge lab-badge-info">
            {GATE_KEYS.map((k) => GATE_LABELS[k]).join(' → ')} → random → winner
          </span>
        </div>
        <div className="net-tablewrap">
          <table className="net-table">
            <thead>
              <tr>
                <th>validator</th>
                <th>country</th>
                {GATE_KEYS.map((k) => (
                  <th key={k}>{GATE_LABELS[k]}</th>
                ))}
                <th>result</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((r) => {
                const isWinner = r.v.id === winnerId
                const isEligible = r.e.eligible
                return (
                  <tr
                    key={r.v.id}
                    className={`${isWinner && gatePass ? 'winner' : ''} ${selectedId === r.v.id ? 'sel' : ''}`}
                    onClick={() => setSelectedId(r.v.id)}
                  >
                    <td className="mono">{r.v.id}</td>
                    <td className="mono">{r.v.country}</td>
                    {GATE_KEYS.map((k) => (
                      <td key={k}>{gateCell(r.e[k])}</td>
                    ))}
                    <td>
                      {isWinner && gatePass ? (
                        <span className="lab-badge lab-badge-ok">WINNER</span>
                      ) : isEligible ? (
                        <span className="lab-badge lab-badge-info">eligible</span>
                      ) : (
                        <span className="text-muted">—</span>
                      )}
                    </td>
                  </tr>
                )
              })}
            </tbody>
          </table>
        </div>

        {/* why-selected detail */}
        <div className="net-detail">
          <div className="flex items-center gap-2 flex-wrap">
            <strong className="text-[12px]">Why {detailRow.v.id}?</strong>
            <span className="lab-badge lab-badge-info">{detailRow.v.name}</span>
            <span className="text-[10px] text-muted">
              {detailRow.v.countryName} · {TIERS[detailRow.v.tier]} · rep {detailRow.v.reputation} bps
            </span>
            {detailRow.v.id === winnerId && gatePass && (
              <span className="lab-badge lab-badge-ok">route winner</span>
            )}
          </div>
          <ul className="net-why">
            {GATE_KEYS.map((k) => (
              <li key={k} className={detailRow.e[k].pass ? 'ok' : 'no'}>
                <span className="net-why-k">{GATE_LABELS[k]}</span>
                <span>{detailRow.e[k].detail}</span>
                <span className="net-why-m">{detailRow.e[k].pass ? 'PASS' : 'FAIL'}</span>
              </li>
            ))}
            {detailRow.v.id === winnerId && winnerDraw !== null && (
              <li className={gatePass ? 'ok' : 'no'}>
                <span className="net-why-k">random</span>
                <span>
                  draw_bps {winnerDraw} vs admit {admitBps(competitors)} bps (competitor_count {competitors}
                  {competitors === 1 ? ' — always passes' : ''})
                </span>
                <span className="net-why-m">{gatePass ? 'PASS' : 'FAIL'}</span>
              </li>
            )}
          </ul>
          {!detailRow.e.eligible && (
            <p className="text-[10px] text-muted mt-1">
              First failing gate in route order decides — later gates are shown for transparency.
            </p>
          )}
        </div>
      </div>

      {/* presence plot */}
      <div className="lab-card">
        <div className="flex items-center justify-between gap-2 flex-wrap mb-2">
          <h3 className="text-[13px] font-semibold">Presence plot — validator locations (equirectangular)</h3>
          <span className="flex gap-2 text-[10px] text-muted items-center">
            <i className="net-dot" style={{ background: '#10b981' }} /> online/available
            <i className="net-dot" style={{ background: '#f59e0b' }} /> busy
            <i className="net-dot" style={{ background: '#ef4444' }} /> suspended
            {task.radiusM > 0 && (
              <>
                <i className="net-dot" style={{ background: 'transparent', border: '1.5px dashed #6366f1' }} />
                task radius
              </>
            )}
          </span>
        </div>
        <svg viewBox={`0 0 ${PW} ${PH}`} width="100%" height={PH} className="net-plot" role="img" aria-label="Validator presence plot">
          <rect x="0" y="0" width={PW} height={PH} rx="8" fill="#f8fafc" />
          {plot.lines.map((l, i) => (
            <line
              key={i}
              x1={l.x1}
              y1={l.y1}
              x2={l.x2}
              y2={l.y2}
              stroke={l.strong ? '#cbd5e1' : '#e2e8f0'}
              strokeWidth={l.strong ? 1 : 0.6}
              strokeDasharray={l.strong ? 'none' : '3 4'}
            />
          ))}
          {plot.radiusPoly && (
            <polygon points={plot.radiusPoly} fill="rgba(99,102,241,0.06)" stroke="#6366f1" strokeWidth="1.2" strokeDasharray="6 4" />
          )}
          {plot.centerPt && (
            <g>
              <line x1={plot.centerPt.x - 7} y1={plot.centerPt.y} x2={plot.centerPt.x + 7} y2={plot.centerPt.y} stroke="#6366f1" strokeWidth="1.5" />
              <line x1={plot.centerPt.x} y1={plot.centerPt.y - 7} x2={plot.centerPt.x} y2={plot.centerPt.y + 7} stroke="#6366f1" strokeWidth="1.5" />
              <text x={plot.centerPt.x + 10} y={plot.centerPt.y - 6} className="net-plot-t">
                {task.centerLabel}
              </text>
            </g>
          )}
          {plot.dots.map(({ row, p }) => {
            if (!p) return null
            const isWinner = row.v.id === winnerId && gatePass
            const isEligible = row.e.eligible
            return (
              <g key={row.v.id} className="net-mk" onClick={() => setSelectedId(row.v.id)}>
                <title>
                  {row.v.id} — {row.v.name} ({row.v.presence?.city ?? 'no presence'})
                  {isWinner ? ' — WINNER' : isEligible ? ' — eligible' : ''}
                </title>
                {isWinner && <circle cx={p.x} cy={p.y} r="10" fill="none" stroke="#10b981" strokeWidth="2" />}
                <circle
                  cx={p.x}
                  cy={p.y}
                  r="5"
                  fill={availColor(row.v.availability)}
                  stroke={isEligible ? '#065f46' : '#fff'}
                  strokeWidth={isEligible ? 2 : 1.2}
                />
                <text x={p.x} y={p.y - 9} textAnchor="middle" className="net-plot-t">
                  {row.v.id}
                </text>
              </g>
            )
          })}
        </svg>
      </div>

      {/* validator cards */}
      <div className="net-grid">
        {rows.map(({ v, e }) => (
          <article
            key={v.id}
            className={`net-card ${e.eligible ? 'eligible' : ''} ${v.id === winnerId && gatePass ? 'win' : ''} ${
              selectedId === v.id ? 'sel' : ''
            }`}
            onClick={() => setSelectedId(v.id)}
          >
            <div className="net-card-head">
              <span className="net-card-name">{v.name}</span>
              <span className="lab-badge lab-badge-info">{v.country}</span>
            </div>
            <div className="mono text-[10px] text-muted truncate" title={v.wallet}>
              {v.wallet}
            </div>
            <div className="net-chips">
              <span className="net-chip tier">{TIERS[v.tier]}</span>
              <span className="net-chip" style={{ color: availColor(v.availability) }}>
                ● {AVAILABILITIES[v.availability]}
              </span>
              <span className="net-chip">{v.countryName}</span>
            </div>
            <div className="net-rep">
              <div className="net-rep-bar">
                <div className="net-rep-fill" style={{ width: `${(v.reputation / MAX_REPUTATION) * 100}%` }} />
              </div>
              <span className="mono text-[10px]">{v.reputation} bps</span>
            </div>
            <div className="net-caps">
              {Object.entries(v.capabilities).map(([code, level]) => (
                <span key={code} className={`net-cap ${level === 0 ? 'weak' : ''}`}>
                  {CAPABILITY_CODES[Number(code)]}·{CAPABILITY_LEVELS[level]}
                </span>
              ))}
            </div>
            <div className="text-[10px] text-muted">
              <MapPin size={10} className="inline align-[-1px]" /> {v.presence?.city ?? '—'}
              {v.presence && v.presence.expiresAt < now ? ' (presence STALE)' : ''} ·{' '}
              {v.presence ? `${(v.presence.latE7 / 1e7).toFixed(2)}°, ${(v.presence.lonE7 / 1e7).toFixed(2)}°` : 'n/a'}
            </div>
            <div className="net-stats mono text-[10px]">
              <span>stake {v.stakeSol.toLocaleString()} SOL</span>
              <span>tasks {v.tasksCompleted}</span>
              <span>acc {(v.accuracyBps / 100).toFixed(1)}%</span>
            </div>
          </article>
        ))}
      </div>

      <footer className="st-foot">
        <p className="text-[10px] text-muted">
          Deterministic: the same task + slot always reroutes to the same winner. The six gates mirror{' '}
          <span className="font-mono">routing.rs</span> exactly (including stale-presence and DECLARED-is-too-
          weak capability rules); the sha256 draws are represented by a sync FNV-1a with identical bps
          semantics — flagged because it is DEMO, not on-chain entropy. When the random gate rejects a
          winner, advance the slot: that is precisely the <span className="font-mono">NotRouteWinner</span>{' '}
          retry path. Live validator data awaits the <Link to="/status">/validators API (planned)</Link>.
        </p>
      </footer>
    </div>
  )
}
