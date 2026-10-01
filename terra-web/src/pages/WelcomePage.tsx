import { useEffect, useMemo, useState } from 'react'
import { Link } from 'react-router-dom'
import {
  ArrowDownRight,
  ArrowRight,
  ArrowUpRight,
  Activity,
  Compass,
  Fingerprint,
  Layers3,
  Radio,
  ShieldCheck,
  Sparkles,
} from 'lucide-react'
import { api } from '../lib/api'
import { useActivityStore, type ActivityItem } from '../lib/activityStore'
import { useLocale } from '../lib/locale'

type FeedState = 'checking' | 'online' | 'offline'

function formatTime(at: number, locale: string) {
  return new Intl.DateTimeFormat(locale, { hour: '2-digit', minute: '2-digit', second: '2-digit' }).format(at)
}

export default function WelcomePage() {
  const { locale, t } = useLocale()
  const demoItems = useActivityStore((state) => state.items)
  const [apiItems, setApiItems] = useState<ActivityItem[]>([])
  const [feedState, setFeedState] = useState<FeedState>('checking')

  useEffect(() => {
    let live = true
    const refresh = async () => {
      if (document.hidden) return
      try {
        const response = await api.listActivity(40)
        if (!live) return
        setApiItems(response.items.map((item) => ({
          id: item.id,
          at: item.at,
          kind: item.kind,
          summary: item.summary,
          source: 'api',
        })))
        setFeedState('online')
      } catch {
        if (live) setFeedState('offline')
      }
    }
    void refresh()
    const timer = window.setInterval(() => void refresh(), 5000)
    return () => {
      live = false
      window.clearInterval(timer)
    }
  }, [])

  const recent = useMemo(
    () => [...demoItems, ...apiItems].sort((a, b) => b.at - a.at).slice(0, 6),
    [apiItems, demoItems],
  )

  return (
    <main className="welcome-page">
      <div className="welcome-shell">
        <section className="welcome-hero">
          <div className="welcome-copy">
            <span className="welcome-eyebrow"><Sparkles size={13} /> {t('welcomeEyebrow')}</span>
            <h1>{t('welcomeTitleA')}<br /><span>{t('welcomeTitleB')}</span></h1>
            <p>{t('welcomeIntro')}</p>
            <div className="welcome-actions">
              <Link className="welcome-primary" to="/atlas"><Compass size={16} /> {t('openAtlas')} <ArrowUpRight size={14} /></Link>
              <Link className="welcome-secondary" to="/demo"><Activity size={15} /> {t('watchDemo')}</Link>
            </div>
            <div className="welcome-assurance">
              <ShieldCheck size={14} /> {t('walletFirst')} <span /> {t('reviewableActivity')} <span /> 2D + 3D
            </div>
          </div>

          <div className="welcome-art" aria-label="Illustration of a parcel review workflow">
            <div className="welcome-art-top">
              <span><span className="welcome-live-dot" /> NETWORK PREVIEW</span>
              <span className="welcome-art-tag">{t('simulation')}</span>
            </div>
            <svg className="welcome-map-art" viewBox="0 0 560 300" role="img" aria-label="Parcel boundaries connected to review nodes">
              <defs>
                <linearGradient id="land-fill" x1="0" y1="0" x2="1" y2="1">
                  <stop offset="0" stopColor="currentColor" stopOpacity=".28" />
                  <stop offset="1" stopColor="currentColor" stopOpacity=".03" />
                </linearGradient>
                <pattern id="map-grid" width="38" height="38" patternUnits="userSpaceOnUse">
                  <path d="M38 0H0V38" fill="none" stroke="currentColor" strokeOpacity=".08" strokeWidth="1" />
                </pattern>
              </defs>
              <rect width="560" height="300" fill="url(#map-grid)" />
              <path className="welcome-road" d="M-20 238 C72 194 106 257 187 209 S307 206 360 153 477 122 586 48" />
              <path className="welcome-road secondary" d="M37 10 C112 82 146 94 239 101 S374 84 515 167" />
              <path className="welcome-parcel" d="M137 110 234 75 296 124 279 205 182 221 121 172Z" />
              <path className="welcome-parcel secondary" d="m296 124 92-26 59 47-23 81-145-21z" />
              <path className="welcome-route" d="M168 163 C224 133 264 164 331 158 S402 147 458 116" />
              <circle className="welcome-node pulse" cx="168" cy="163" r="7" />
              <circle className="welcome-node" cx="331" cy="158" r="6" />
              <circle className="welcome-node pulse delay" cx="458" cy="116" r="7" />
              <g className="welcome-map-label" transform="translate(144 245)">
                <rect width="130" height="31" rx="8" />
                <text x="13" y="20">PARCEL · 0x8F2…</text>
              </g>
              <g className="welcome-map-label floating" transform="translate(347 43)">
                <rect width="160" height="40" rx="9" />
                <circle cx="16" cy="20" r="4" />
                <text x="29" y="24">3 REVIEWERS ACTIVE</text>
              </g>
            </svg>
            <div className="welcome-art-footer">
              <span><Layers3 size={13} /> {t('spatialRecord')}</span>
              <ArrowRight size={14} />
              <span><Fingerprint size={13} /> {t('identityChecks')}</span>
              <ArrowRight size={14} />
              <span><ShieldCheck size={13} /> {t('reviewOutcome')}</span>
            </div>
          </div>
        </section>

        <section className="welcome-summary" aria-label="Platform overview">
          <article className="welcome-stat">
            <span className="welcome-stat-icon"><Radio size={16} /></span>
            <div><strong>{feedState === 'checking' ? '…' : apiItems.length}</strong><span>{t('recentApiRecords')}</span></div>
            <small>{feedState === 'online' ? t('latestWindow') : feedState === 'offline' ? t('apiOffline') : t('connectingFeed')}</small>
          </article>
          <article className="welcome-stat">
            <span className="welcome-stat-icon"><Compass size={16} /></span>
            <div><strong>2D <i>/</i> 3D</strong><span>{t('landContext')}</span></div>
            <small>{t('switchAtlas')}</small>
          </article>
          <article className="welcome-stat">
            <span className="welcome-stat-icon"><Fingerprint size={16} /></span>
            <div><strong>{t('selfCustody')}</strong><span>{t('walletLinked')}</span></div>
            <small>{t('walletControls')}</small>
          </article>
        </section>

        <section className="welcome-lower">
          <div className="welcome-activity">
            <div className="welcome-section-head">
              <div><span className="welcome-section-kicker">{t('networkPulse')}</span><h2>{t('recentActivity')}</h2></div>
              <span className={`welcome-status ${feedState}`}><i />{feedState === 'online' ? t('apiConnected') : feedState === 'offline' ? t('apiOffline') : t('connecting')}</span>
            </div>
            {recent.length ? (
              <div className="welcome-feed">
                {recent.map((item, index) => (
                  <article className="welcome-feed-row" key={`${item.source}-${item.id}`}>
                    <span className={`welcome-feed-dot ${item.source}`} />
                    <div className="welcome-feed-copy">
                      <div><strong>{item.kind.replaceAll('_', ' ')}</strong><span className={`welcome-source ${item.source}`}>{item.source === 'api' ? 'API record' : 'DEMO'}</span></div>
                      <p>{item.summary}</p>
                    </div>
                    <time>{formatTime(item.at, locale === 'zh' ? 'zh-CN' : locale)}</time>
                    {index < recent.length - 1 && <span className="welcome-feed-line" />}
                  </article>
                ))}
              </div>
            ) : (
              <div className="welcome-empty">
                <Activity size={18} />
                <p>{feedState === 'offline' ? t('offlineActivity') : t('noActivity')}</p>
                <Link to="/demo">{t('playDemo')} <ArrowRight size={13} /></Link>
              </div>
            )}
            <Link className="welcome-all-activity" to="/transactions">{t('activityExplorer')} <ArrowRight size={14} /></Link>
          </div>

          <aside className="welcome-next">
            <span className="welcome-section-kicker">{t('startHere')}</span>
            <h2>{t('yourLand')}</h2>
            <p>{t('welcomeNext')}</p>
            <Link to="/portfolio" className="welcome-next-link"><span className="welcome-next-icon"><Fingerprint size={17} /></span><span><strong>{t('setupPortfolio')}</strong><small>{t('portfolioDesc')}</small></span><ArrowUpRight size={15} /></Link>
            <Link to="/network" className="welcome-next-link"><span className="welcome-next-icon"><ShieldCheck size={17} /></span><span><strong>{t('browseNetwork')}</strong><small>{t('networkDesc')}</small></span><ArrowUpRight size={15} /></Link>
            <div className="welcome-note"><ArrowDownRight size={14} /> {t('demoNotice')}</div>
          </aside>
        </section>
      </div>
    </main>
  )
}
