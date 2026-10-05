import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { Radio } from 'lucide-react'
import { api, type ActivityResponse } from '../lib/api'
import { useActivityStore, type ActivityItem } from '../lib/activityStore'
import { useLocale, type TranslationKey } from '../lib/locale'

// B4 live activity feed — merges:
//   * demo-engine events pushed while /demo scenarios play (source: 'demo')
//   * real off-chain API rows polled from GET /api/v1/activity (source: 'api')
// Polling pauses on hidden tabs; API errors degrade to the demo-only view.
// On-chain Solana tx indexing is a named gap (see /status).

type Translate = (key: TranslationKey, vars?: Record<string, string | number>) => string

function ago(at: number, t: Translate) {
  const s = Math.max(1, Math.floor((Date.now() - at) / 1000))
  if (s < 60) return t('agoSeconds', { n: s })
  const m = Math.floor(s / 60)
  if (m < 60) return t('agoMinutes', { n: m })
  const h = Math.floor(m / 60)
  if (h < 24) return t('agoHours', { n: h })
  return t('agoDays', { n: Math.floor(h / 24) })
}

export default function ActivityFeed({ pollMs = 5000 }: { pollMs?: number }) {
  const { t } = useLocale()
  const demoItems = useActivityStore((s) => s.items)
  const [apiItems, setApiItems] = useState<ActivityItem[]>([])
  const [apiOffline, setApiOffline] = useState(false)

  useEffect(() => {
    let live = true
    const tick = async () => {
      if (document.hidden) return
      try {
        const r: ActivityResponse = await api.listActivity(40)
        if (!live) return
        setApiOffline(false)
        setApiItems(
          r.items.map((x) => ({
            id: x.id,
            at: x.at,
            kind: x.kind,
            summary: x.summary,
            source: 'api' as const,
          })),
        )
      } catch {
        if (live) setApiOffline(true)
      }
    }
    void tick()
    const timer = setInterval(() => void tick(), pollMs)
    return () => {
      live = false
      clearInterval(timer)
    }
  }, [pollMs])

  const merged = [...demoItems, ...apiItems]
    .sort((a, b) => b.at - a.at)
    .slice(0, 60)

  return (
    <section className="af">
      <div className="af-head">
        <span className="af-title">
          <Radio size={13} /> {t('liveActivity')}
        </span>
        <span className="af-src">
          {apiOffline ? (
            <span className="af-offline">{t('apiOfflineDemoOnly')}</span>
          ) : (
            t('apiPolledEvery', { seconds: pollMs / 1000 })
          )}
        </span>
      </div>
      <div className="af-list">
        {merged.length === 0 && (
          <div className="af-empty">
            {t('activityEmptyA')}
            <Link to="/demo">{t('activityEmptyB')}</Link>
            {t('activityEmptyC')}
          </div>
        )}
        {merged.map((it) => (
          <div key={it.id} className="af-item">
            <span
              className={`af-dot af-${it.source}`}
              title={it.source === 'demo' ? t('demoEngineSimulated') : t('offchainApiRealRow')}
            />
            <div className="af-body">
              <div className="af-kind">
                {it.kind}
                <span className="af-sum">{it.summary}</span>
              </div>
              <div className="af-meta">
                <span>{ago(it.at, t)}</span>
                {it.source === 'demo' ? <span>demo</span> : <span>api</span>}
                {it.link && <Link to={it.link}>{t('openExternal')}</Link>}
              </div>
            </div>
          </div>
        ))}
      </div>
    </section>
  )
}
