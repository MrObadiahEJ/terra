import { useEffect, useMemo, useState, type CSSProperties } from 'react'
import { Link } from 'react-router-dom'
import {
  ArrowDownRight,
  ArrowRight,
  ArrowUpRight,
  Activity,
  Compass,
  Fingerprint,
  Layers3,
  MapPin,
  Radio,
  ShieldCheck,
  Sparkles,
  Wallet,
} from 'lucide-react'
import { api } from '../lib/api'
import { useActivityStore, type ActivityItem } from '../lib/activityStore'
import { useLocale } from '../lib/locale'
import { useWallet } from '../lib/wallet'
import { useAppStore } from '../store/appStore'

type FeedState = 'checking' | 'online' | 'offline'

const CUBE_LAYOUT: [number, number][] = [
  [260, 30],
  [188, 66],
  [332, 66],
  [116, 102],
  [260, 102],
  [404, 102],
  [188, 138],
  [332, 138],
]

function cubePaths(x: number, y: number) {
  const w = 36
  const h = 18
  const depth = 40
  return {
    top: `M${x},${y} L${x + w},${y + h} L${x},${y + 2 * h} L${x - w},${y + h} Z`,
    left: `M${x - w},${y + h} L${x},${y + 2 * h} L${x},${y + 2 * h + depth} L${x - w},${y + h + depth} Z`,
    right: `M${x + w},${y + h} L${x},${y + 2 * h} L${x},${y + 2 * h + depth} L${x + w},${y + h + depth} Z`,
  }
}

function formatTime(at: number, locale: string) {
  return new Intl.DateTimeFormat(locale, { hour: '2-digit', minute: '2-digit', second: '2-digit' }).format(at)
}

export default function WelcomePage() {
  const { locale, t } = useLocale()
  const { publicKey, walletName } = useWallet()
  const parcels = useAppStore((state) => state.parcels)
  const loadingParcels = useAppStore((state) => state.loadingParcels)
  const parcelsError = useAppStore((state) => state.parcelsError)
  const offChainParcels = useAppStore((state) => state.offChainParcels)
  const loadingOffChain = useAppStore((state) => state.loadingOffChain)
  const refreshOffChain = useAppStore((state) => state.refreshOffChain)
  const ownedParcels = useMemo(
    () => publicKey ? parcels.filter((parcel) => parcel.holder === publicKey.toBase58()) : [],
    [parcels, publicKey],
  )
  const demoItems = useActivityStore((state) => state.items)
  const [apiItems, setApiItems] = useState<ActivityItem[]>([])
  const [feedState, setFeedState] = useState<FeedState>('checking')
  const cycleParcels = useMemo(() => offChainParcels.slice(0, 6), [offChainParcels])
  const [cycleIndex, setCycleIndex] = useState(0)
  const [cyclePaused, setCyclePaused] = useState(false)

  useEffect(() => {
    if (offChainParcels.length === 0 && !loadingOffChain) void refreshOffChain()
  }, [offChainParcels.length, loadingOffChain, refreshOffChain])

  useEffect(() => {
    if (cycleParcels.length < 2 || cyclePaused) return
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return
    const timer = window.setInterval(() => {
      if (document.hidden) return
      setCycleIndex((index) => (index + 1) % cycleParcels.length)
    }, 3500)
    return () => window.clearInterval(timer)
  }, [cycleParcels.length, cyclePaused])

  const activeIndex = cycleParcels.length ? cycleIndex % cycleParcels.length : 0
  const activeParcel = cycleParcels[activeIndex] ?? null

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

          <div className="welcome-art" aria-label="Terra land asset preview">
            <div className="welcome-art-top">
              <span><span className="welcome-live-dot" /> {t('walletOverview')}</span>
              <span className={`welcome-art-tag ${publicKey ? 'connected' : ''}`}>
                {publicKey ? t('walletConnected') : t('walletNotConnected')}
              </span>
            </div>
            <div className="welcome-asset-card">
              <div className="welcome-asset-meta">
                <span>{t('registeredLand')}</span>
                <span className="welcome-asset-network"><i /> SOLANA · DEVNET</span>
              </div>
              <div className="welcome-asset-main">
                <div>
                  <span className="welcome-asset-label">{t('ownedLandAssets')}</span>
                  <strong>{publicKey && !loadingParcels && !parcelsError ? ownedParcels.length : '—'}</strong>
                  <span className="welcome-asset-caption">
                    {!publicKey ? t('connectToViewHoldings') : loadingParcels ? t('loadingHoldings') : parcelsError ? t('holdingsUnavailable') : t('verifiedOnChain')}
                  </span>
                </div>
                <div className="welcome-land-mark" aria-hidden="true">
                  <svg viewBox="0 0 120 120">
                    <path d="m18 34 45-18 39 23-7 48-45 20-39-24 7-49Z" />
                    <path d="m18 34 44 23 40-18M62 57l-12 50M41 25l-2 45 11 37M85 28l-6 45 16 14" />
                    <circle cx="62" cy="57" r="3" />
                    <circle cx="18" cy="34" r="2" />
                    <circle cx="102" cy="39" r="2" />
                  </svg>
                </div>
              </div>
              <div className="welcome-wallet-row">
                <span className="welcome-wallet-icon"><Wallet size={14} /></span>
                <span>
                  <strong>{publicKey ? walletName || t('walletConnected') : t('selfCustody')}</strong>
                  <small>{publicKey ? `${publicKey.toBase58().slice(0, 6)}…${publicKey.toBase58().slice(-5)}` : t('walletControls')}</small>
                </span>
                <Link to="/portfolio" aria-label={t('openPortfolio')}><ArrowUpRight size={15} /></Link>
              </div>
            </div>
            <div
              className="welcome-art-preview"
              onMouseEnter={() => setCyclePaused(true)}
              onMouseLeave={() => setCyclePaused(false)}
            >
              <span className="welcome-preview-kicker">{t('spatialPreview')}</span>
              {activeParcel && (
                <div className="welcome-cycle-head">
                  <strong key={activeParcel.id}>{activeParcel.name}</strong>
                  <span>
                    {Math.round(activeParcel.area_m2 ?? 0).toLocaleString(locale === 'zh' ? 'zh-CN' : locale)} m² · {activeParcel.status}
                  </span>
                </div>
              )}
              <svg
                key={activeParcel?.id ?? 'static'}
                className="welcome-map-art welcome-cubes"
                viewBox="0 0 560 230"
                role="img"
                aria-label={activeParcel ? `${activeParcel.name} isometric land block` : 'Isometric land block preview'}
              >
                <ellipse className="welcome-cube-ring" cx="260" cy="120" rx="176" ry="92" />
                <ellipse className="welcome-cube-shadow" cx="260" cy="216" rx="132" ry="11" />
                {CUBE_LAYOUT.map(([cx, cy], index) => {
                  const faces = cubePaths(cx, cy)
                  return (
                    <g className="welcome-cube" style={{ '--i': index } as CSSProperties} key={`${cx}-${cy}`}>
                      <g className={index === 0 ? 'welcome-cube-bob' : undefined}>
                        <path className="cube-face cube-left" d={faces.left} />
                        <path className="cube-face cube-right" d={faces.right} />
                        <path className="cube-face cube-top" d={faces.top} />
                      </g>
                    </g>
                  )
                })}
                <circle className="welcome-parcel-node" cx="84" cy="120" r="4" />
                <circle className="welcome-parcel-node" cx="436" cy="120" r="4" />
                <circle className="welcome-parcel-node" cx="260" cy="28" r="4" />
              </svg>
              <div className="welcome-art-footer">
                <span><Layers3 size={13} /> {t('spatialRecord')}</span>
                {activeParcel ? (
                  <Link className="welcome-cycle-open" to={`/atlas?parcel=${activeParcel.id}`}>
                    {t('openAtlas')} <ArrowUpRight size={11} />
                  </Link>
                ) : (
                  <span className="welcome-preview-note">{t('illustrativeGeometry')}</span>
                )}
              </div>
              {activeParcel && cycleParcels.length > 1 && (
                <div className="welcome-cycle-dots">
                  {cycleParcels.map((parcel, index) => (
                    <button
                      key={parcel.id}
                      type="button"
                      className={index === activeIndex ? 'on' : ''}
                      aria-label={parcel.name}
                      aria-pressed={index === activeIndex}
                      onClick={() => setCycleIndex(index)}
                    />
                  ))}
                </div>
              )}
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
            <span className="welcome-stat-icon"><MapPin size={16} /></span>
            <div><strong>{publicKey && !loadingParcels && !parcelsError ? ownedParcels.length : '—'}</strong><span>{t('ownedLandAssets')}</span></div>
            <small>{!publicKey ? t('connectToViewHoldings') : loadingParcels ? t('loadingHoldings') : parcelsError ? t('holdingsUnavailable') : t('verifiedOnChain')}</small>
          </article>
          <article className="welcome-stat">
            <span className="welcome-stat-icon"><Compass size={16} /></span>
            <div><strong>2D <i>/</i> 3D</strong><span>{t('landContext')}</span></div>
            <small>{t('switchAtlas')}</small>
          </article>
        </section>

        <section className="welcome-lower">
          <div className="welcome-holdings">
            <div className="welcome-section-head">
              <div><span className="welcome-section-kicker">{t('walletOverview')}</span><h2>{t('yourLandAssets')}</h2></div>
              <Link to="/portfolio" className="welcome-section-action">{t('openPortfolio')} <ArrowUpRight size={13} /></Link>
            </div>
            {publicKey && !loadingParcels && !parcelsError && ownedParcels.length > 0 ? (
              <div className="welcome-holdings-list">
                {ownedParcels.slice(0, 3).map((parcel) => (
                  <article className="welcome-holding-row" key={parcel.address}>
                    <span className="welcome-holding-icon"><MapPin size={15} /></span>
                    <span className="welcome-holding-copy">
                      <strong>{parcel.account.name}</strong>
                      <small>{t('onChainRecord')} · {parcel.id.slice(0, 8)}…</small>
                    </span>
                    <span className="welcome-holding-status"><ShieldCheck size={13} /> {t('verified')}</span>
                    <ArrowRight size={14} className="welcome-holding-arrow" />
                  </article>
                ))}
              </div>
            ) : (
              <div className="welcome-holdings-empty">
                <span className="welcome-empty-icon"><Wallet size={17} /></span>
                <div>
                  <strong>{!publicKey ? t('walletNotConnected') : loadingParcels ? t('loadingHoldings') : parcelsError ? t('holdingsUnavailable') : t('noOwnedLandYet')}</strong>
                  <p>{!publicKey ? t('connectToViewHoldings') : loadingParcels ? t('loadingHoldingsDescription') : parcelsError ? parcelsError : t('noOwnedLandDescription')}</p>
                </div>
                <Link to={publicKey ? '/atlas' : '/portfolio'} aria-label={publicKey ? t('openAtlas') : t('openPortfolio')}>
                  <ArrowUpRight size={15} />
                </Link>
              </div>
            )}
          </div>
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
