import { useState } from 'react'
import { Globe2, RotateCcw } from 'lucide-react'

// --- country helpers (cross_border.rs: [u8;2] ISO-3166-1 alpha-2 bytes) -----

type Country = 'none' | 'CM' | 'NG' | 'GH' | 'KE'
type BindingState = 'none' | 'ACTIVE' | 'SUSPENDED' | 'REVOKED' | 'EXPIRED'

const COUNTRY_LABEL: Record<Country, string> = {
  none: 'undeclared [0,0]',
  CM: 'CM — Cameroon',
  NG: 'NG — Nigeria',
  GH: 'GH — Ghana',
  KE: 'KE — Kenya',
}

const BINDING_LABEL: Record<BindingState, string> = {
  none: 'no binding presented',
  ACTIVE: 'ACTIVE (unexpired)',
  SUSPENDED: 'SUSPENDED',
  REVOKED: 'REVOKED',
  EXPIRED: 'ACTIVE but expired',
}

const bytesOf = (c: Country): [number, number] =>
  c === 'none' ? [0, 0] : [c.charCodeAt(0), c.charCodeAt(1)]

const hexBytes = (b: [number, number]) =>
  `[${b[0]}, ${b[1]}] = 0x${b[0].toString(16).padStart(2, '0')}${b[1].toString(16).padStart(2, '0')}`

const pairKey = (a: [number, number], b: [number, number]): string => {
  const [min, max] = a <= b ? [a, b] : [b, a]
  return `${String.fromCharCode(min[0], min[1])}↔${String.fromCharCode(max[0], max[1])}`
}

interface Gate {
  label: string
  state: 'pass' | 'fail' | 'skip'
  detail: string
}

interface Verdict {
  ok: boolean
  headline: string
  gates: Gate[]
}

/** routing.rs::passes_jurisdiction + binding_slot_ok — soft-fail candidate filter. */
function routeVerdict(req: Country, home: Country, binding: BindingState): Verdict {
  const required = bytesOf(req)
  const homeB = bytesOf(home)
  const gates: Gate[] = []

  const needJuris = required[0] !== 0 || required[1] !== 0
  if (!needJuris) {
    gates.push({
      label: 'requirement.jurisdiction',
      state: 'pass',
      detail: '[0,0] — no constraint, binding slot ignored',
    })
    return { ok: true, headline: 'Eligible — candidate routed', gates }
  }
  gates.push({
    label: 'requirement.jurisdiction',
    state: 'pass',
    detail: hexBytes(required),
  })

  if (homeB[0] === 0 && homeB[1] === 0) {
    gates.push({
      label: 'passes_jurisdiction (profile)',
      state: 'fail',
      detail: 'profile.jurisdiction = [0,0] → false (undeclared home jurisdiction)',
    })
    return { ok: false, headline: 'Not eligible — filtered out of route_task', gates }
  }

  if (homeB === required || (homeB[0] === required[0] && homeB[1] === required[1])) {
    gates.push({
      label: 'passes_jurisdiction (same country)',
      state: 'pass',
      detail: `${hexBytes(homeB)} matches requirement → no binding needed (slot is filler)`,
    })
    gates.push({ label: 'binding_slot_ok', state: 'skip', detail: 'candidate is in-country' })
    return { ok: true, headline: 'Eligible — candidate routed', gates }
  }

  gates.push({
    label: 'passes_jurisdiction (foreign)',
    state: 'fail',
    detail: `${hexBytes(homeB)} ≠ ${hexBytes(required)} → needs binding_ok`,
  })

  if (binding === 'none') {
    gates.push({
      label: 'binding_slot_ok',
      state: 'fail',
      detail: 'no CrossBorderBinding account for the ordered pair → soft-fail',
    })
    return { ok: false, headline: 'Not eligible — filtered out of route_task', gates }
  }

  gates.push({
    label: 'binding pair',
    state: 'pass',
    detail: `ordered pair ${pairKey(homeB, required)} (lexicographic min,max)`,
  })

  if (binding === 'ACTIVE') {
    gates.push({
      label: 'binding.status / expiry',
      state: 'pass',
      detail: 'ACTIVE and expires_at > now → binding_slot_ok = true',
    })
    return { ok: true, headline: 'Eligible — candidate routed', gates }
  }

  gates.push({
    label: 'binding.status / expiry',
    state: 'fail',
    detail:
      binding === 'EXPIRED'
        ? 'binding_is_expired(expires_at, now) → true → soft-fail'
        : `binding.status = ${binding} ≠ ACTIVE → soft-fail`,
  })
  return { ok: false, headline: 'Not eligible — filtered out of route_task', gates }
}

/** cross_border.rs::record_cross_border_verification — hard errors, in order. */
function spanVerdict(req: Country, home: Country, binding: BindingState): Verdict {
  const required = bytesOf(req)
  const homeB = bytesOf(home)
  const gates: Gate[] = []

  const undeclared = (homeB[0] === 0 && homeB[1] === 0) || (required[0] === 0 && required[1] === 0)
  if (undeclared) {
    gates.push({
      label: 'home / required declared',
      state: 'fail',
      detail: 'UndeclaredJurisdiction (6230): home or required is [0,0]',
    })
    return { ok: false, headline: 'Rejected — UndeclaredJurisdiction (6230)', gates }
  }
  gates.push({ label: 'home / required declared', state: 'pass', detail: `${hexBytes(homeB)} vs ${hexBytes(required)}` })

  const same = homeB[0] === required[0] && homeB[1] === required[1]
  if (same) {
    gates.push({
      label: 'home ≠ required',
      state: 'fail',
      detail: 'SameJurisdiction (6227): span must cross a border',
    })
    return { ok: false, headline: 'Rejected — SameJurisdiction (6227)', gates }
  }
  gates.push({ label: 'home ≠ required', state: 'pass', detail: 'cross-border span' })

  if (binding === 'none') {
    gates.push({
      label: 'binding PDA key',
      state: 'fail',
      detail: `JurisdictionMismatch (6228): expected ["cross_border_binding", min, max] for ${pairKey(homeB, required)}`,
    })
    return { ok: false, headline: 'Rejected — JurisdictionMismatch (6228)', gates }
  }
  gates.push({
    label: 'binding PDA key',
    state: 'pass',
    detail: `["cross_border_binding", …] for ${pairKey(homeB, required)}`,
  })

  if (binding !== 'ACTIVE' && binding !== 'EXPIRED') {
    gates.push({
      label: 'binding.status == ACTIVE',
      state: 'fail',
      detail: `BindingRevoked (6089): binding.status = ${binding} (any non-ACTIVE status maps here)`,
    })
    return { ok: false, headline: 'Rejected — BindingRevoked (6089)', gates }
  }
  gates.push({ label: 'binding.status == ACTIVE', state: 'pass', detail: binding === 'EXPIRED' ? 'ACTIVE' : 'ACTIVE' })

  if (binding === 'EXPIRED') {
    gates.push({
      label: 'expires_at > now',
      state: 'fail',
      detail: 'BindingExpired (6090): binding_is_expired(expires_at, now)',
    })
    return { ok: false, headline: 'Rejected — BindingExpired (6090)', gates }
  }
  gates.push({ label: 'expires_at > now', state: 'pass', detail: 'not expired' })

  return {
    ok: true,
    headline: 'Recorded — CrossBorderVerificationRecorded',
    gates,
  }
}

const PRESETS: { label: string; req: Country; home: Country; binding: BindingState }[] = [
  { label: 'Home-country validator', req: 'CM', home: 'CM', binding: 'none' },
  { label: 'Valid cross-border', req: 'CM', home: 'NG', binding: 'ACTIVE' },
  { label: 'Suspended binding', req: 'CM', home: 'NG', binding: 'SUSPENDED' },
  { label: 'Expired binding', req: 'CM', home: 'NG', binding: 'EXPIRED' },
]

function VerdictCard({ title, verdict }: { title: string; verdict: Verdict }) {
  return (
    <div className="lab-card">
      <div className="flex items-center justify-between mb-2 gap-2">
        <h3 className="text-[13px] font-semibold">{title}</h3>
        <span className={`lab-badge ${verdict.ok ? 'lab-badge-ok' : 'lab-badge-err'}`}>
          {verdict.ok ? 'PASS' : 'FAIL'}
        </span>
      </div>
      <p className={`text-[12px] font-medium mb-2 ${verdict.ok ? 'text-emerald-700' : 'text-red-700'}`}>
        {verdict.headline}
      </p>
      <ol className="lab-gates">
        {verdict.gates.map((g, i) => (
          <li key={i} className={`gate-${g.state}`}>
            <span className="gate-mark">
              {g.state === 'pass' ? '✓' : g.state === 'fail' ? '✗' : '–'}
            </span>
            <span className="font-mono text-[11px]">{g.label}</span>
            <span className="text-[11px] text-muted break-all">{g.detail}</span>
          </li>
        ))}
      </ol>
    </div>
  )
}

export default function CrossBorderGate() {
  const [req, setReq] = useState<Country>('CM')
  const [home, setHome] = useState('NG' as Country)
  const [binding, setBinding] = useState<BindingState>('ACTIVE')

  const route = routeVerdict(req, home, binding)
  const span = spanVerdict(req, home, binding)

  const apply = (p: (typeof PRESETS)[number]) => {
    setReq(p.req)
    setHome(p.home)
    setBinding(p.binding)
  }

  return (
    <div className="lab-body">
      <div className="lab-card space-y-2 mb-3">
        <div className="flex items-center justify-between flex-wrap gap-2">
          <h3 className="text-[13px] font-semibold flex items-center gap-1.5">
            <Globe2 size={14} /> Inputs (RFC-012 Phase 10)
          </h3>
          <button className="btn btn-ghost px-2 py-1" onClick={() => apply(PRESETS[1])}>
            <RotateCcw size={12} /> Reset
          </button>
        </div>
        <div className="grid grid-cols-2 gap-x-3 gap-y-1" style={{ gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))' }}>
          <label className="text-[11px] text-muted">
            Task requirement — jurisdiction
            <select className="select-input" value={req} onChange={(e) => setReq(e.target.value as Country)}>
              {(Object.keys(COUNTRY_LABEL) as Country[]).map((c) => (
                <option key={c} value={c}>
                  {COUNTRY_LABEL[c]}
                </option>
              ))}
            </select>
          </label>
          <label className="text-[11px] text-muted">
            Validator profile — home jurisdiction
            <select className="select-input" value={home} onChange={(e) => setHome(e.target.value as Country)}>
              {(Object.keys(COUNTRY_LABEL) as Country[]).map((c) => (
                <option key={c} value={c}>
                  {COUNTRY_LABEL[c]}
                </option>
              ))}
            </select>
          </label>
          <label className="text-[11px] text-muted">
            CrossBorderBinding for the ordered pair
            <select
              className="select-input"
              value={binding}
              onChange={(e) => setBinding(e.target.value as BindingState)}
            >
              {(Object.keys(BINDING_LABEL) as BindingState[]).map((b) => (
                <option key={b} value={b}>
                  {BINDING_LABEL[b]}
                </option>
              ))}
            </select>
          </label>
        </div>
        <div className="flex flex-wrap gap-1 pt-1">
          <span className="text-[11px] text-muted self-center mr-1">Presets:</span>
          {PRESETS.map((p) => (
            <button key={p.label} className="btn btn-secondary px-2 py-1 text-[11px]" onClick={() => apply(p)}>
              {p.label}
            </button>
          ))}
        </div>
        <p className="text-[10px] text-muted">
          PDA seed: <span className="font-mono">["cross_border_binding", min, max]</span> — pairs are ordered
          lexicographically, so CM↔NG is one binding no matter who travels. Expires_at = 0 means never expires.
        </p>
      </div>

      <div className="lab-grid">
        <VerdictCard title="route_task — candidate eligibility (soft-fail filter)" verdict={route} />
        <VerdictCard title="record_cross_border_verification — auditable span record" verdict={span} />
      </div>
    </div>
  )
}
