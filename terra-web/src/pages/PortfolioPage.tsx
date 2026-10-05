import { useCallback, useEffect, useMemo, useState } from 'react'
import type { FormEvent } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { PublicKey } from '@solana/web3.js'
import {
  ArrowDownToLine,
  ArrowRight,
  BadgeCheck,
  BriefcaseBusiness,
  CircleHelp,
  Copy,
  Fingerprint,
  Landmark,
  LoaderCircle,
  MapPin,
  RefreshCw,
  Shield,
  Wallet,
} from 'lucide-react'
import { api, type IdentityView, type OffChainParcel } from '../lib/api'
import { connection, useWallet } from '../lib/wallet'
import { useAppStore, type OnChainParcelItem } from '../store/appStore'
import { useTxStore } from '../lib/txStore'
import { useLocale } from '../lib/locale'

type IdentityState = 'disconnected' | 'loading' | 'linked' | 'unlinked' | 'error'
type AssetFilter = 'owned' | 'map'

function shortAddress(address: string) {
  return `${address.slice(0, 6)}…${address.slice(-5)}`
}

function formatArea(area: number | null, notMeasured: string) {
  if (area == null) return notMeasured
  if (area >= 1_000_000) return `${(area / 1_000_000).toFixed(2)} km²`
  if (area >= 10_000) return `${(area / 10_000).toFixed(2)} ha`
  return `${Math.round(area).toLocaleString()} m²`
}

function isNoIdentity(error: unknown) {
  return error instanceof Error && /^404:/.test(error.message)
}

export default function PortfolioPage() {
  const { t } = useLocale()
  const navigate = useNavigate()
  const { publicKey, walletName, signMessage } = useWallet()
  const address = publicKey?.toBase58() ?? null
  const parcels = useAppStore((state) => state.parcels)
  const offChainParcels = useAppStore((state) => state.offChainParcels)
  const demoMode = useAppStore((state) => state.demoMode)
  const refreshParcels = useAppStore((state) => state.refreshParcels)
  const refreshOffChain = useAppStore((state) => state.refreshOffChain)
  const selectParcel = useAppStore((state) => state.selectParcel)
  const selectOffChain = useAppStore((state) => state.selectOffChain)
  const transactionCount = useTxStore((state) => state.txs.length)

  const [balance, setBalance] = useState<number | null>(null)
  const [balanceError, setBalanceError] = useState<string | null>(null)
  const [balanceWallet, setBalanceWallet] = useState<string | null>(null)
  const [identity, setIdentity] = useState<IdentityView | null>(null)
  const [identityState, setIdentityState] = useState<IdentityState>('disconnected')
  const [identityWallet, setIdentityWallet] = useState<string | null>(null)
  const [identityError, setIdentityError] = useState<string | null>(null)
  const [credential, setCredential] = useState('')
  const [recovery, setRecovery] = useState('')
  const [binding, setBinding] = useState(false)
  const [bindingError, setBindingError] = useState<string | null>(null)
  const [assetFilter, setAssetFilter] = useState<AssetFilter>('owned')
  const [assetQuery, setAssetQuery] = useState('')
  const [refreshing, setRefreshing] = useState(false)
  const [identityAttempt, setIdentityAttempt] = useState(0)
  const visibleIdentityState: IdentityState = !address
    ? 'disconnected'
    : identityWallet !== address
      ? 'loading'
      : identityState
  const visibleBalance = address && balanceWallet === address ? balance : null
  const visibleBalanceError = address && balanceWallet === address ? balanceError : null

  const myChainParcels = useMemo(
    () => (address ? parcels.filter((parcel) => parcel.holder === address) : []),
    [address, parcels],
  )
  const myMapParcels = useMemo(
    () =>
      address
        ? offChainParcels.filter(
            (parcel) => parcel.holder === address && !parcel.holder.startsWith('demo:'),
          )
        : [],
    [address, offChainParcels],
  )
  const standaloneMapParcels = useMemo(
    () =>
      myMapParcels.filter(
        (mapParcel) =>
          !parcels.some(
            (chainParcel) =>
              chainParcel.holder === mapParcel.holder &&
              chainParcel.account.name === mapParcel.name,
          ),
      ),
    [myMapParcels, parcels],
  )
  const mappedArea = useMemo(
    () => myMapParcels.reduce((sum, parcel) => sum + (parcel.area_m2 ?? 0), 0),
    [myMapParcels],
  )
  const mapRecords = useMemo(() => {
    const source = assetFilter === 'owned' && address
      ? [...standaloneMapParcels]
      : assetFilter === 'owned'
        ? []
        : offChainParcels
    const query = assetQuery.trim().toLocaleLowerCase()
    return source.filter((parcel) =>
      `${parcel.name} ${parcel.holder} ${parcel.status}`.toLocaleLowerCase().includes(query),
    )
  }, [address, assetFilter, assetQuery, offChainParcels, standaloneMapParcels])

  const refresh = useCallback(async () => {
    setRefreshing(true)
    if (address) {
      setBalanceWallet(null)
      setIdentityWallet(null)
      setIdentityAttempt((attempt) => attempt + 1)
    }
    try {
      await Promise.all([refreshOffChain(), ...(address ? [refreshParcels()] : [])])
    } finally {
      setRefreshing(false)
    }
  }, [address, refreshOffChain, refreshParcels])

  useEffect(() => {
    let current = true
    if (!publicKey) return () => { current = false }
    const wallet = publicKey.toBase58()
    void Promise.allSettled([
      connection.getBalance(publicKey),
      api.getIdentityByWallet(wallet),
    ]).then(([balanceResult, identityResult]) => {
      if (!current) return
      setBalanceWallet(wallet)
      if (balanceResult.status === 'fulfilled') {
        setBalance(balanceResult.value / 1_000_000_000)
      } else {
        setBalanceError(balanceResult.reason instanceof Error ? balanceResult.reason.message : t('balanceUnavailable'))
      }
      if (identityResult.status === 'fulfilled') {
        setIdentity(identityResult.value)
        setIdentityState('linked')
      } else if (isNoIdentity(identityResult.reason)) {
        setIdentityState('unlinked')
      } else {
        setIdentityError(
          identityResult.reason instanceof Error ? identityResult.reason.message : t('identityServiceUnavailable'),
        )
        setIdentityState('error')
      }
      setIdentityWallet(wallet)
    })
    return () => { current = false }
  }, [identityAttempt, publicKey, t])

  useEffect(() => {
    void refreshOffChain()
  }, [refreshOffChain])

  const bindIdentity = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    if (!publicKey || !signMessage) {
      setBindingError(t('walletCannotSign'))
      return
    }
    if (credential.trim().length < 24) {
      setBindingError(t('phraseMin24'))
      return
    }
    let recoveryKey: PublicKey
    try {
      recoveryKey = new PublicKey(recovery.trim())
    } catch {
      setBindingError(t('validRecoveryAddress'))
      return
    }
    if (recoveryKey.equals(publicKey)) {
      setBindingError(t('recoveryMustDiffer'))
      return
    }

    setBinding(true)
    setBindingError(null)
    try {
      const digest = await crypto.subtle.digest(
        'SHA-256',
        new TextEncoder().encode(credential),
      )
      const identityHash = Array.from(new Uint8Array(digest), (byte) =>
        byte.toString(16).padStart(2, '0'),
      ).join('')
      const result = await api.bindIdentity(
        {
          identity_hash: identityHash,
          owner: publicKey.toBase58(),
          recovery: recoveryKey.toBase58(),
        },
        signMessage,
      )
      setIdentity(result)
      setIdentityState('linked')
      setCredential('')
      setRecovery('')
    } catch (error) {
      setBindingError(error instanceof Error ? error.message : t('identityLinkFailed'))
    } finally {
      setBinding(false)
    }
  }

  const openMapParcel = (parcel: OffChainParcel) => {
    selectOffChain(parcel)
    navigate('/atlas')
  }
  const openChainParcel = (parcel: OnChainParcelItem) => {
    selectParcel(parcel)
    navigate('/atlas')
  }

  const ownedAssetCount = myChainParcels.length + standaloneMapParcels.length

  return (
    <main className="portfolio-page">
      <header className="portfolio-page-head">
        <div>
          <span className="portfolio-eyebrow"><span className="live-dot" /> {t('landOwnerConsole')}</span>
          <h1>{t('portfolioTitle')}</h1>
          <p>{t('portfolioSubtitle')}</p>
        </div>
        <button className="btn btn-secondary portfolio-refresh" onClick={() => void refresh()} disabled={refreshing}>
          {refreshing ? <LoaderCircle size={14} className="animate-spin" /> : <RefreshCw size={14} />}
          {t('refreshPortfolio')}
        </button>
      </header>

<section className="portfolio-overview" aria-label={t('portfolioOverview')}>
        <article className="portfolio-balance-card">
<div className="portfolio-card-label"><Wallet size={14} /> {t('walletBalance')}</div>
          <strong>{visibleBalance == null ? (address ? '—' : t('connectWallet')) : visibleBalance.toFixed(4)}</strong>
          <span>{visibleBalance == null ? (visibleBalanceError ?? (address ? t('readingDevnet') : t('balanceAppearsHere'))) : 'SOL · Solana devnet'}</span>
        </article>
        <article className="portfolio-metric-card">
<div className="portfolio-card-label"><Landmark size={14} /> {t('pfOwnedLandAssets')}</div>
          <strong>{address ? ownedAssetCount : '—'}</strong>
          <span>{address ? t('holdingsSplit', { chain: myChainParcels.length, mapped: standaloneMapParcels.length }) : t('connectHoldings')}</span>
        </article>
        <article className="portfolio-metric-card">
<div className="portfolio-card-label"><MapPin size={14} /> {t('mappedArea')}</div>
          <strong>{address ? formatArea(mappedArea, t('areaNotMeasured')) : '—'}</strong>
          <span>{address ? t('measuredFromGeometry') : t('areaLinkedWallet')}</span>
        </article>
        <article className="portfolio-metric-card">
<div className="portfolio-card-label"><BadgeCheck size={14} /> {t('identityStat')}</div>
          <strong>{visibleIdentityState === 'linked' ? t('identityLinked') : visibleIdentityState === 'loading' ? t('identityChecking') : visibleIdentityState === 'unlinked' ? t('identityNotLinked') : '—'}</strong>
          <span>{identityState === 'linked' ? t('walletIdentityCommitment') : t('privateIdentityRecovery')}</span>
        </article>
      </section>

      <section className="portfolio-columns">
        <article className="portfolio-card identity-card">
          <div className="portfolio-section-head">
            <div className="portfolio-section-icon"><Fingerprint size={17} /></div>
            <div>
              <span className="portfolio-eyebrow">{t('ssiEyebrow')}</span>
              <h2>{t('identityRecoveryHeading')}</h2>
            </div>
          </div>
          {!address ? (
            <div className="portfolio-empty">
              <p>{t('identityEmptyHint')}</p>
              <span>{t('identityTiedHint')}</span>
            </div>
          ) : visibleIdentityState === 'loading' ? (
            <div className="portfolio-inline-status"><LoaderCircle size={16} className="animate-spin" /> {t('lookingUpWallet')}</div>
          ) : visibleIdentityState === 'linked' && identity ? (
            <div className="identity-summary">
              <div className="identity-linked-badge"><BadgeCheck size={14} /> {t('identityLinkedToWallet')}</div>
              <dl>
                <div><dt>{t('ownerWallet')}</dt><dd title={identity.owner}>{shortAddress(identity.owner)}</dd></div>
                <div><dt>{t('recoveryWallet')}</dt><dd title={identity.recovery}>{shortAddress(identity.recovery)}</dd></div>
                <div><dt>{t('commitmentLabel')}</dt><dd title={identity.identity_hash}>{identity.identity_hash.slice(0, 12)}…{identity.identity_hash.slice(-8)}</dd></div>
                <div><dt>{t('linkedParcelsLabel')}</dt><dd>{identity.parcel_count}</dd></div>
                <div><dt>{t('labelCreated')}</dt><dd>{new Date(identity.created_at).toLocaleDateString()}</dd></div>
              </dl>
              <p className="identity-privacy-note"><Shield size={13} /> {t('identityPrivacyNote')}</p>
            </div>
          ) : visibleIdentityState === 'error' ? (
            <div className="portfolio-callout portfolio-callout-error">
              <CircleHelp size={15} />
              <span>{identityError || t('identityServiceFailed')}</span>
              <button className="btn btn-ghost" onClick={() => {
                setIdentityWallet(null)
                setIdentityAttempt((n) => n + 1)
              }}>{t('retryWord')}</button>
            </div>
          ) : (
            <>
<p className="portfolio-copy">{t('identityFormCopy')}</p>
              <form className="identity-form" onSubmit={(event) => void bindIdentity(event)}>
                <label>
                  {t('privateIdentityPhrase')}
                  <input
                    className="text-input"
                    type="password"
                    autoComplete="new-password"
                    value={credential}
                    onChange={(event) => setCredential(event.target.value)}
                    placeholder={t('phrasePlaceholder')}
                    minLength={24}
                    required
                  />
                </label>
                <label>
                  {t('recoveryWalletAddress')}
                  <input
                    className="text-input"
                    value={recovery}
                    onChange={(event) => setRecovery(event.target.value)}
                    placeholder={t('solanaAddressPlaceholder')}
                    autoComplete="off"
                    required
                  />
                </label>
                <div className="identity-privacy-note">
                  <Shield size={14} />
                  <span>{t('identityFormPrivacy')}</span>
                </div>
                {bindingError && <div className="portfolio-callout portfolio-callout-error">{bindingError}</div>}
                <button className="btn btn-primary identity-submit" type="submit" disabled={binding || !signMessage}>
                  {binding ? <><LoaderCircle size={14} className="animate-spin" /> {t('signingAndLinking')}</> : <><Fingerprint size={14} /> {t('createIdentityLink')}</>}
                </button>
                {!signMessage && <span className="text-xs text-muted">{t('walletNoSignMessage')}</span>}
              </form>
            </>
          )}
          {identityError && visibleIdentityState !== 'error' && <p className="portfolio-inline-error">{identityError}</p>}
        </article>

        <article className="portfolio-card wallet-card">
          <div className="portfolio-section-head">
            <div className="portfolio-section-icon"><Wallet size={17} /></div>
            <div>
              <span className="portfolio-eyebrow">{t('connectedSigner')}</span>
              <h2>{t('walletHeading')}</h2>
            </div>
          </div>
          {address ? (
            <div className="wallet-summary">
              <div className="wallet-profile">
                <span className="wallet-avatar">{(walletName || 'W').slice(0, 1).toUpperCase()}</span>
                <div><strong>{walletName || t('solanaWallet')}</strong><span>{t('connectedOnDevnet')}</span></div>
                <BadgeCheck size={16} className="wallet-verified-icon" />
              </div>
              <button
                className="wallet-address-copy"
                onClick={() => void navigator.clipboard.writeText(address)}
                title={t('copyWalletAddress')}
                aria-label={t('copyWalletAddressFull')}
              >
                <span>{shortAddress(address)}</span><Copy size={13} />
              </button>
              <div className="wallet-security-note">
                <Shield size={15} />
                <span>{t('walletSecurityNote')}</span>
              </div>
              <a className="portfolio-explorer-link" href={`https://explorer.solana.com/address/${address}?cluster=devnet`} target="_blank" rel="noreferrer">
                {t('viewWalletExplorer')} <ArrowRight size={13} />
              </a>
            </div>
          ) : (
            <div className="portfolio-empty">
              <p>{t('walletNotConnected')}</p>
              <span>{t('walletConnectHint')}</span>
            </div>
          )}
          <div className="portfolio-activity-link">
            <span><ArrowDownToLine size={14} /> {t('recentTransactions')} <b>{transactionCount}</b></span>
            <Link to="/transactions">{t('viewActivity')} <ArrowRight size={13} /></Link>
          </div>
        </article>
      </section>

      <section className="portfolio-card portfolio-assets">
        <div className="portfolio-section-head portfolio-assets-head">
          <div className="portfolio-section-icon"><BriefcaseBusiness size={17} /></div>
          <div className="portfolio-assets-title">
            <span className="portfolio-eyebrow">{t('landRegistryEyebrow')}</span>
            <h2>{t('assetsHeading')}</h2>
          </div>
          <div className="land-viewer-seg" role="group" aria-label={t('assetListFilter')}>
            <button className={`land-viewer-segbtn ${assetFilter === 'owned' ? 'on' : ''}`} onClick={() => setAssetFilter('owned')}>{t('myAssets')}</button>
            <button className={`land-viewer-segbtn ${assetFilter === 'map' ? 'on' : ''}`} onClick={() => setAssetFilter('map')}>{t('mapRecords')}</button>
          </div>
        </div>
        <div className="portfolio-asset-toolbar">
          <label className="parcel-search portfolio-asset-search">
            <MapPin size={15} />
            <input value={assetQuery} onChange={(event) => setAssetQuery(event.target.value)} placeholder={t('searchLandAssets')} aria-label={t('searchLandAssets')} />
          </label>
          <Link className="btn btn-primary" to="/atlas"><MapPin size={14} /> {t('openLandAtlas')}</Link>
        </div>
        {!address && assetFilter === 'owned' ? (
          <div className="portfolio-empty asset-empty">
            <p>{t('connectSeeHoldings')}</p>
            <span>{t('explorePilotHint')}</span>
            <Link to="/atlas">{t('exploreDemoParcels')} <ArrowRight size={13} /></Link>
          </div>
        ) : assetFilter === 'owned' && myChainParcels.length === 0 && mapRecords.length === 0 ? (
          <div className="portfolio-empty asset-empty">
            <p>{t('noLinkedParcels')}</p>
            <span>{t('drawBoundaryHint')}</span>
            <Link to="/atlas">{t('registerOrExplore')} <ArrowRight size={13} /></Link>
          </div>
        ) : (
          <div className="portfolio-asset-list">
            {myChainParcels.map((parcel) => (
              <button key={parcel.address} className="portfolio-asset-row" onClick={() => openChainParcel(parcel)}>
                <span className="asset-row-icon"><Landmark size={16} /></span>
                <span className="asset-row-main"><strong>{parcel.account.name}</strong><span>{shortAddress(parcel.address)} · {t('onchainOwnership')}</span></span>
                <span className="asset-row-status">{t('verifiedOwner')}</span><ArrowRight size={14} className="asset-row-arrow" />
              </button>
            ))}
            {mapRecords.map((parcel) => (
              <button key={parcel.id} className="portfolio-asset-row" onClick={() => openMapParcel(parcel)}>
                <span className="asset-row-icon"><MapPin size={16} /></span>
                <span className="asset-row-main"><strong>{parcel.name}</strong><span>{formatArea(parcel.area_m2, t('areaNotMeasured'))} · {parcel.holder === address ? t('linkedToThisWallet') : shortAddress(parcel.holder)}</span></span>
                <span className={`asset-row-status ${parcel.holder === address ? 'owned' : ''} ${demoMode ? 'demo' : ''}`}>{demoMode ? t('demoBadge') : parcel.holder === address ? parcel.status : t('mapRecord')}</span>
                <ArrowRight size={14} className="asset-row-arrow" />
              </button>
            ))}
            {assetFilter === 'map' && mapRecords.length === 0 && (
              <div className="portfolio-empty asset-empty"><p>{t('noMapRecordsMatch')}</p></div>
            )}
          </div>
        )}
        {demoMode && <div className="portfolio-demo-note"><Shield size={13} /> {t('demoParcelsNote')}</div>}
      </section>
    </main>
  )
}
