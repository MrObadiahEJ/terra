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
import { useLocale } from '../lib/locale'

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
  const { t } = useLocale()
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
            <Activity size={18} /> {t('statusTitle')}
          </h1>
          <p className="text-[11px] text-muted max-w-3xl">
            {t('statusIntroA')}<span className="font-mono">dev</span>{t('statusIntroB')}
            <strong>{t('statusImplementedWord')}</strong>{t('statusIntroC')}
            <strong>{t('statusNotWord')}</strong>{t('statusIntroD')}
          </p>
        </div>
        <div className="land-badges">
          <a
            className="btn btn-ghost px-2 py-1 gap-1"
            href="https://github.com/mrobadiahej/terra/tree/dev"
            target="_blank"
            rel="noreferrer"
          >
            <GitBranch size={13} /> {t('devOnGitHub')}
          </a>
          <Link className="btn btn-secondary px-2 py-1" to="/lab?tab=vault">
            {t('openTheLab')}
          </Link>
        </div>
      </header>

      {/* live IDL counters + status tallies */}
      <div className="st-stats">
        <StatChip n={idlInstructions} label={t('idlInstructions')} cls="st-n-ink" />
        <StatChip n={idlAccounts} label={t('idlAccounts')} cls="st-n-ink" />
        <StatChip n={idlEvents} label={t('idlEvents')} cls="st-n-ink" />
        <StatChip n={idlTypes} label={t('idlTypes')} cls="st-n-ink" />
        <span className="st-sep" />
        <StatChip n={counts.implemented} label={t('countImplemented')} cls="st-n-ok" />
        <StatChip n={counts.partial} label={t('countPartial')} cls="st-n-warn" />
        <StatChip n={counts.experimental} label={t('countExperimental')} cls="st-n-info" />
        <StatChip n={counts.planned} label={t('countPlanned')} cls="st-n-mut" />
      </div>

      <ActivityFeed />

      {/* filters */}
      <div className="st-filters">
        <div className="land-viewer-seg">
          <button
            className={`land-viewer-segbtn ${track === 'all' ? 'on' : ''}`}
            onClick={() => setTrack('all')}
          >
            {t('allWord')}
          </button>
          {STATUS_TRACKS.map((trackId) => (
            <button
              key={trackId}
              className={`land-viewer-segbtn ${track === trackId ? 'on' : ''}`}
              onClick={() => setTrack(trackId)}
              title={t(TRACK_META[trackId].hintKey)}
            >
              {t(TRACK_META[trackId].labelKey)}
            </button>
          ))}
        </div>
        <div className="st-level-chips">
          <button
            className={`st-level-chip ${level === 'all' ? 'on' : ''}`}
            onClick={() => setLevel('all')}
          >
            {t('allLevels')}
          </button>
          {STATUS_LEVELS.map((l) => (
            <button
              key={l}
              className={`st-level-chip ${level === l ? 'on' : ''} lvl-${l}`}
              onClick={() => setLevel(l)}
            >
              {t(STATUS_META[l].labelKey)}
            </button>
          ))}
        </div>
        <span className="flex-1" />
        <span className="text-[10px] text-muted font-mono">
          {t('modulesCount', { modules: modules.length, total: STATUS_MODULES.length })}
        </span>
      </div>

      {/* legend */}
      <div className="st-legend">
        <span className="text-[10px] text-muted">
          <CheckCircle2 size={11} className="inline align-[-1px] text-emerald-600" /> {t('legendImplemented')}{'   '}
          <Info size={11} className="inline align-[-1px] text-indigo-600" /> {t('legendExperimental')}{'   '}
          <XCircle size={11} className="inline align-[-1px] text-amber-600" /> {t('legendPartial')}{'   '}
          <CircleDashed size={11} className="inline align-[-1px] text-slate-400" /> {t('legendPlanned')}
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
                <span className={`lab-badge ${meta.badge}`}>{t(meta.labelKey)}</span>
              </div>
              <div className="st-card-track">{t(TRACK_META[m.track].labelKey)}</div>
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
          <p className="text-[12px] text-muted">{t('noModulesMatch')}</p>
        )}
      </div>

      <footer className="st-foot">
        <p className="text-[10px] text-muted">
          {t('statusFooter1')}
          <span className="font-mono">terra-core</span>
          {t('statusFooter2')}
          <span className="font-mono">tests/integration.rs</span>
          {t('statusFooter3', {
            instructions: idlInstructions,
            accounts: idlAccounts,
            events: idlEvents,
            types: idlTypes,
          })}
          {t('stPartial')} / {t('stExperimental')}
          {t('statusFooter4')}
        </p>
      </footer>
    </div>
  )
}
