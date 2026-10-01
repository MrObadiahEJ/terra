import { useState } from 'react'
import { MapPin, Shuffle } from 'lucide-react'
import {
  AVAILABILITIES,
  CAPABILITY_ANY,
  CAPABILITY_CODES,
  CAPABILITY_LEVELS,
  MAX_REPUTATION,
  PRESENCE_TTL_SECS,
  TIERS,
  drawBps,
  evaluateEligibility,
  passesRandomGate,
  routeSeed,
  selectWinner,
  type TaskRequirement,
} from '../../../lib/routingSim'
import { DEMO_BLOCKHASH, DEMO_INITIAL_SLOT } from '../../../lib/validatorDemo'
import { Broadcast, ClockBar, DemoBanner, Field, VerdictCard } from './shared'
import { SIM_START, err, pushGate, type Gate, type Verdict } from './helpers'

const CENTER = { latE7: 4_051_000, lonE7: 9_700_000, label: 'Buea, CM' } // routing.rs demo center

function candidateVerdict(
  req: TaskRequirement,
  avail: number,
  tier: number,
  rep: number,
  home: string,
  bindingOk: boolean,
  capLevel: number,
  present: boolean,
  distKm: number,
  now: number,
): Verdict {
  const presence = present
    ? {
        latE7: CENTER.latE7 + Math.round((distKm * 1000 * 1e7) / 111_320),
        lonE7: CENTER.lonE7,
        city: 'demo-city',
        expiresAt: SIM_START + PRESENCE_TTL_SECS,
      }
    : null
  const el = evaluateEligibility(
    req,
    {
      tier,
      reputation: rep,
      availability: avail,
      jurisdiction: home,
      capabilities: req.capabilityCode === CAPABILITY_ANY ? {} : { [req.capabilityCode]: capLevel },
      presence,
    },
    bindingOk,
    now,
  )
  const gates: Gate[] = []
  pushGate(gates, 'passes_availability', el.availability.pass, el.availability.detail)
  pushGate(gates, 'passes_tier', el.tier.pass, el.tier.detail)
  pushGate(gates, 'passes_reputation', el.reputation.pass, el.reputation.detail)
  pushGate(gates, 'passes_capability', el.capability.pass, el.capability.detail)
  pushGate(gates, 'passes_jurisdiction', el.jurisdiction.pass, el.jurisdiction.detail)
  pushGate(gates, 'passes_geo', el.geo.pass, el.geo.detail)
  return {
    ok: el.eligible,
    headline: el.eligible
      ? 'Eligible — enters the competitor set for draw_bps'
      : 'Filtered out — route_task rejects before the random gate',
    gates,
  }
}

export default function RoutingPanel() {
  const [now, setNow] = useState(SIM_START)
  const [minTier, setMinTier] = useState(2)
  const [minRep, setMinRep] = useState(5_000)
  const [reqCode, setReqCode] = useState<number>(CAPABILITY_ANY)
  const [reqJuris, setReqJuris] = useState<string>('CM')
  const [radiusKm, setRadiusKm] = useState(50)
  const [avail, setAvail] = useState(0) // ONLINE
  const [tier, setTier] = useState(2) // ESTABLISHED
  const [rep, setRep] = useState(7_000)
  const [home, setHome] = useState('NG')
  const [bindingOk, setBindingOk] = useState(true)
  const [capLevel, setCapLevel] = useState(2) // VERIFIED
  const [present, setPresent] = useState(true)
  const [distKm, setDistKm] = useState(12)
  const [blockhash, setBlockhash] = useState(DEMO_BLOCKHASH)
  const [slot, setSlot] = useState(DEMO_INITIAL_SLOT)

  const req: TaskRequirement = {
    id: 'demo-task-7',
    label: 'Boundary survey task',
    description: 'DEMO requirement record',
    minTier,
    minReputation: minRep,
    capabilityCode: reqCode,
    jurisdiction: reqJuris === 'none' ? null : reqJuris,
    radiusM: radiusKm * 1000,
    centerLatE7: CENTER.latE7,
    centerLonE7: CENTER.lonE7,
    centerLabel: CENTER.label,
  }
  const verdict = candidateVerdict(req, avail, tier, rep, home, bindingOk, capLevel, present, distKm, now)

  // Deterministic winner among the competitor set (DEMO candidate + 2 pre-qualified rivals).
  const rivals = ['V-rex-9k2', 'V-sol-4p8']
  const competitors = verdict.ok ? ['TerraDemo-V1', ...rivals] : rivals
  const seed = routeSeed(req.id, blockhash, slot)
  const draws = competitors.map((id) => ({ id, draw: drawBps(seed, id) }))
  const winner = selectWinner(seed, competitors)
  const myDraw = drawBps(seed, 'TerraDemo-V1')
  const myRandomPass = verdict.ok ? passesRandomGate(myDraw, competitors.length) : false
  const routed = verdict.ok && winner === 'TerraDemo-V1' && myRandomPass
  const lastGate = verdict.ok
    ? myRandomPass
      ? { ok: true, summary: `ValidatorRouted — draw ${myDraw} < 10000/${competitors.length}, winner ${winner}` }
      : { ok: false, summary: `Rejected — ${err('NotRouteWinner')} — draw ${myDraw} ≥ 10000/${competitors.length}` }
    : { ok: false, summary: `Filtered — ${err('ValidatorNotEligible')} — a route gate failed above` }

  return (
    <div className="lab-body">
      <DemoBanner source="routing.rs + cross_border.rs (RFC-012 Phase 10)" />

      <div className="lab-grid mt-3">
        <div className="lab-card space-y-2">
          <h3 className="text-[13px] font-semibold flex items-center gap-1.5">
            <MapPin size={14} /> Task requirement → validator profile
          </h3>
          <div className="pg-fields">
            <Field label={`min_tier = ${TIERS[minTier]}`}>
              <select className="select-input" value={minTier} onChange={(e) => setMinTier(Number(e.target.value))}>
                {TIERS.map((t, i) => (
                  <option key={t} value={i}>{`${i} — ${t}`}</option>
                ))}
              </select>
            </Field>
            <Field label={`min_reputation = ${minRep} bps (MAX ${MAX_REPUTATION})`}>
              <input
                className="select-input"
                type="number"
                min={0}
                max={MAX_REPUTATION}
                value={minRep}
                onChange={(e) => setMinRep(Math.max(0, Math.min(MAX_REPUTATION, Number(e.target.value) || 0)))}
              />
            </Field>
            <Field label="capability required">
              <select className="select-input" value={reqCode} onChange={(e) => setReqCode(Number(e.target.value))}>
                <option value={CAPABILITY_ANY}>255 — CAPABILITY_ANY</option>
                {CAPABILITY_CODES.map((c, i) => (
                  <option key={c} value={i}>{`${i} — ${c}`}</option>
                ))}
              </select>
            </Field>
            <Field label="requirement.jurisdiction">
              <select className="select-input" value={reqJuris} onChange={(e) => setReqJuris(e.target.value)}>
                <option value="none">[0,0] — none</option>
                {['CM', 'NG', 'GH', 'KE'].map((c) => (
                  <option key={c} value={c}>{c}</option>
                ))}
              </select>
            </Field>
            <Field label={`radius = ${radiusKm} km (0 = no geo constraint)`}>
              <input
                className="select-input"
                type="number"
                min={0}
                value={radiusKm}
                onChange={(e) => setRadiusKm(Math.max(0, Number(e.target.value) || 0))}
              />
            </Field>
            <Field label="candidate availability">
              <select className="select-input" value={avail} onChange={(e) => setAvail(Number(e.target.value))}>
                {AVAILABILITIES.map((a, i) => (
                  <option key={a} value={i}>{`${i} — ${a}`}</option>
                ))}
              </select>
            </Field>
            <Field label="candidate tier">
              <select className="select-input" value={tier} onChange={(e) => setTier(Number(e.target.value))}>
                {TIERS.map((t, i) => (
                  <option key={t} value={i}>{`${i} — ${t}`}</option>
                ))}
              </select>
            </Field>
            <Field label={`candidate reputation = ${rep}`}>
              <input
                className="select-input"
                type="number"
                min={0}
                max={MAX_REPUTATION}
                value={rep}
                onChange={(e) => setRep(Math.max(0, Math.min(MAX_REPUTATION, Number(e.target.value) || 0)))}
              />
            </Field>
            <Field label="candidate home jurisdiction">
              <select className="select-input" value={home} onChange={(e) => setHome(e.target.value)}>
                <option value="">[0,0] — undeclared</option>
                {['CM', 'NG', 'GH', 'KE'].map((c) => (
                  <option key={c} value={c}>{c}</option>
                ))}
              </select>
            </Field>
            <Field label="CrossBorderBinding for home→required">
              <select
                className="select-input"
                value={bindingOk ? 'ok' : 'bad'}
                onChange={(e) => setBindingOk(e.target.value === 'ok')}
              >
                <option value="ok">ACTIVE binding present</option>
                <option value="bad">no / revoked binding</option>
              </select>
            </Field>
            <Field label={`candidate ${reqCode === CAPABILITY_ANY ? 'capability' : CAPABILITY_CODES[reqCode]} level`}>
              <select className="select-input" value={capLevel} onChange={(e) => setCapLevel(Number(e.target.value))}>
                {CAPABILITY_LEVELS.map((l, i) => (
                  <option key={l} value={i}>{`${i} — ${l}`}</option>
                ))}
              </select>
            </Field>
            <Field label="presence record">
              <select
                className="select-input"
                value={present ? 'fresh' : 'none'}
                onChange={(e) => setPresent(e.target.value === 'fresh')}
              >
                <option value="fresh">fresh (within 24h TTL)</option>
                <option value="none">absent</option>
              </select>
            </Field>
            <Field label={`distance from ${CENTER.label} = ${distKm} km`}>
              <input
                className="select-input"
                type="number"
                min={0}
                max={20000}
                value={distKm}
                onChange={(e) => setDistKm(Math.max(0, Number(e.target.value) || 0))}
              />
            </Field>
          </div>
        </div>

        <div className="lab-card space-y-2">
          <h3 className="text-[13px] font-semibold flex items-center gap-1.5">
            <Shuffle size={14} /> Deterministic random gate (route_seed → draw_bps → select_winner)
          </h3>
          <div className="pg-fields">
            <Field label="blockhash">
              <input
                className="select-input"
                value={blockhash}
                onChange={(e) => setBlockhash(e.target.value)}
              />
            </Field>
            <Field label="slot">
              <input
                className="select-input"
                type="number"
                value={slot}
                onChange={(e) => setSlot(Number(e.target.value) || 0)}
              />
            </Field>
          </div>
          <button
            className="btn btn-secondary px-2 py-1 text-[11px]"
            onClick={() => {
              setBlockhash(`TerraDemoBlockhash${(slot * 2654435761) % 10 ** 10}`)
              setSlot(slot + 1 + (slot % 7))
            }}
          >
            new blockhash + slot
          </button>
          <table className="pg-table">
            <thead>
              <tr>
                <th>competitor</th>
                <th>draw_bps</th>
                <th>random gate</th>
              </tr>
            </thead>
            <tbody>
              {draws.map((d) => (
                <tr key={d.id} className={d.id === 'TerraDemo-V1' ? 'pg-row-mine' : ''}>
                  <td className="font-mono text-[11px]">{d.id}</td>
                  <td className="font-mono text-[11px]">{d.draw}</td>
                  <td className="text-[11px]">
                    {passesRandomGate(d.draw, competitors.length)
                      ? `pass (< ${Math.floor(10_000 / competitors.length)})`
                      : `fail (≥ ${Math.floor(10_000 / competitors.length)})`}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          <p className="text-[11px] text-muted">
            seed = <span className="font-mono">{seed}</span> · winner ={' '}
            <span className="font-mono">{winner ?? '—'}</span> · task jurisdiction{' '}
            {req.jurisdiction === null ? '[0,0] any' : req.jurisdiction}
          </p>
        </div>
      </div>

      <div className="mt-3">
        <VerdictCard title="route_task — candidate filter gates (routing.rs, soft-fail order)" verdict={verdict} />
      </div>

      <div className="mt-3">
        <Broadcast
          ix="route_task"
          ok={lastGate.ok}
          summary={
            lastGate.ok
              ? lastGate.summary
              : `${lastGate.summary} — ${
                  verdict.ok
                    ? `draw ${myDraw} / competitors ${competitors.length}`
                    : verdict.gates.find((g) => g.state === 'fail')?.label
                }`
          }
          note={routed ? 'Routed: all six filter gates + random gate passed.' : 'See gate list — first failing gate decides.'}
        />
      </div>
      <p className="text-[10px] text-muted mt-2">
        Demo chain view: failed broadcasts land as{' '}
        <span className="font-mono">{lastGate.ok ? 'confirmed route record' : lastGate.summary.split(' — ').slice(0, 2).join(' — ')}</span>{' '}
        on <a href="/transactions">/transactions</a>. Winner set includes 2 pre-qualified rivals so the 10000/n
        admission gate is always exercised with n ≥ 3.
      </p>
      <ClockBar now={now} onAdvance={(s) => setNow(now + s)} onReset={() => setNow(SIM_START)} />
    </div>
  )
}
