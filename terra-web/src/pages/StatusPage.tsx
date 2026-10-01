import { useMemo, useState } from 'react'
import { Link } from 'react-router-dom'
import { terraRegistry } from '../idl/terraRegistry'
import {
  STATUS_LEVELS,
  STATUS_META,
  STATUS_MODULES,
  STATUS_TRACKS,
  TRACK_META,
  statusCounts,
  type StatusLevel,
  type StatusTrack,
} from '../lib/protocolStatus'
import { Activity, CheckCircle2, CircleDashed, GitBranch, Info, XCircle } from 'lucide-react'
import ActivityFeed from '../components/ActivityFeed'

// Protocol status (B10) — honest public progress dashboard.
// Every row cites real source locations; nothing is green because a fn exists.

type TrackFilter = 'all' | StatusTrack
type LevelFilter = 'all' | StatusLevel

function StatChip({ n, label, cls }: { n: number; label: string; cls: string }) {
  return (
    <div className="st-stat">
      <span className={`st-stat-n ${cls}`}>{n}</span>
      <span className="st-stat-l">{label}</span>
    </div>
  )
}

export default function StatusPage() {
  const [track, setTrack] = useState<TrackFilter>('all')
  const [level, setLevel] = useState<LevelFilter>('all')

  const idlInstructions = terraRegistry.instructions?.length ?? 0
  const idlAccounts = terraRegistry.accounts?.length ?? 0
  const idlEvents = terraRegistry.events?.length ?? 0
  const idlTypes = terraRegistry.types?.length ?? 0

  const counts = statusCounts()
  const modules = useMemo(
    () =>
      STATUS_MODULES.filter(
        (m) => (track === 'all' || m.track === track) && (level === 'all' || m.status === level),
      ),
    [track, level],
  )

  return (
    <div className="st-page">
      <header className="land-head">
        <div className="min-w-0">
          <h1 className="land-title">
            <Activity size={18} /> Terra — Protocol status
          </h1>
          <p className="text-[11px] text-muted max-w-3xl">
            Source of truth: the <span className="font-mono">dev</span> branch itself — Anchor handlers, IDL,
            integration tests, API routes and shipped frontend. <strong>IMPLEMENTED</strong> requires
            end-to-end evidence (handler + tests, or a shipped UI); a Rust function merely existing is
            <strong> not</strong> enough.
          </p>
        </div>
        <div className="land-badges">
          <a
            className="btn btn-ghost px-2 py-1 gap-1"
            href="https://github.com/mrobadiahej/terra/tree/dev"
            target="_blank"
            rel="noreferrer"
          >
            <GitBranch size={13} /> dev on GitHub
          </a>
          <Link className="btn btn-secondary px-2 py-1" to="/lab?tab=vault">
            Open the Lab ↗
          </Link>
        </div>
      </header>

      {/* live IDL counters + status tallies */}
      <div className="st-stats">
        <StatChip n={idlInstructions} label="IDL instructions" cls="st-n-ink" />
        <StatChip n={idlAccounts} label="IDL accounts" cls="st-n-ink" />
        <StatChip n={idlEvents} label="IDL events" cls="st-n-ink" />
        <StatChip n={idlTypes} label="IDL types" cls="st-n-ink" />
        <span className="st-sep" />
        <StatChip n={counts.implemented} label="implemented" cls="st-n-ok" />
        <StatChip n={counts.partial} label="partial" cls="st-n-warn" />
        <StatChip n={counts.experimental} label="experimental" cls="st-n-info" />
        <StatChip n={counts.planned} label="planned" cls="st-n-mut" />
      </div>

      <ActivityFeed />

      {/* filters */}
      <div className="st-filters">
        <div className="land-viewer-seg">
          <button
            className={`land-viewer-segbtn ${track === 'all' ? 'on' : ''}`}
            onClick={() => setTrack('all')}
          >
            All
          </button>
          {STATUS_TRACKS.map((t) => (
            <button
              key={t}
              className={`land-viewer-segbtn ${track === t ? 'on' : ''}`}
              onClick={() => setTrack(t)}
              title={TRACK_META[t].hint}
            >
              {TRACK_META[t].label}
            </button>
          ))}
        </div>
        <div className="st-level-chips">
          <button
            className={`st-level-chip ${level === 'all' ? 'on' : ''}`}
            onClick={() => setLevel('all')}
          >
            all levels
          </button>
          {STATUS_LEVELS.map((l) => (
            <button
              key={l}
              className={`st-level-chip ${level === l ? 'on' : ''} lvl-${l}`}
              onClick={() => setLevel(l)}
            >
              {STATUS_META[l].label}
            </button>
          ))}
        </div>
        <span className="flex-1" />
        <span className="text-[10px] text-muted font-mono">
          {modules.length} / {STATUS_MODULES.length} modules
        </span>
      </div>

      {/* legend */}
      <div className="st-legend">
        <span className="text-[10px] text-muted">
          <CheckCircle2 size={11} className="inline align-[-1px] text-emerald-600" /> implemented — proven
          end-to-end in source/tests{'   '}
          <Info size={11} className="inline align-[-1px] text-indigo-600" /> experimental — real code, not
          production-proven{'   '}
          <XCircle size={11} className="inline align-[-1px] text-amber-600" /> partial — named gap remains{'   '}
          <CircleDashed size={11} className="inline align-[-1px] text-slate-400" /> planned — designed, not
          started
        </span>
      </div>

      {/* module cards */}
      <div className="st-grid">
        {modules.map((m) => {
          const meta = STATUS_META[m.status]
          return (
            <article key={m.id} className={`st-card ${m.status}`}>
              <div className="st-card-head">
                <span className="st-card-name">{m.name}</span>
                <span className={`lab-badge ${meta.badge}`}>{meta.label}</span>
              </div>
              <div className="st-card-track">{TRACK_META[m.track].label}</div>
              <code className="st-card-ev" title={m.evidence}>
                {m.evidence}
              </code>
              {m.verified.length > 0 && (
                <ul className="st-list ok">
                  {m.verified.map((v) => (
                    <li key={v}>{v}</li>
                  ))}
                </ul>
              )}
              {m.gaps.length > 0 && (
                <ul className="st-list gap">
                  {m.gaps.map((g) => (
                    <li key={g}>{g}</li>
                  ))}
                </ul>
              )}
            </article>
          )
        })}
        {modules.length === 0 && (
          <p className="text-[12px] text-muted">No modules match the current filter.</p>
        )}
      </div>

      <footer className="st-foot">
        <p className="text-[10px] text-muted">
          Honest by construction: statuses were derived from <span className="font-mono">terra-core</span>{' '}
          modules, <span className="font-mono">tests/integration.rs</span> (339 test fns), the generated
          IDL ({idlInstructions} instructions / {idlAccounts} accounts / {idlEvents} events / {idlTypes}{' '}
          types) and the {21} REST route groups. When a feature is only half-wired (auto vault rotation,
          hardware attestation, production ZK setup), it is listed as PARTIAL or EXPERIMENTAL — not green.
        </p>
      </footer>
    </div>
  )
}
