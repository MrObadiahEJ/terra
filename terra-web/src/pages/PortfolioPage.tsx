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

type IdentityState = 'disconnected' | 'loading' | 'linked' | 'unlinked' | 'error'
type AssetFilter = 'owned' | 'map'

function shortAddress(address: string) {
  return `${address.slice(0, 6)}…${address.slice(-5)}`
}

function formatArea(area: number | null) {
  if (area == null) return 'Area not measured'
  if (area >= 1_000_000) return `${(area / 1_000_000).toFixed(2)} km²`
  if (area >= 10_000) return `${(area / 10_000).toFixed(2)} ha`
  return `${Math.round(area).toLocaleString()} m²`
}

function isNoIdentity(error: unknown) {
  return error instanceof Error && /^404:/.test(error.message)
}

export default function PortfolioPage() {
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
        setBalanceError(balanceResult.reason instanceof Error ? balanceResult.reason.message : 'Balance unavailable')
      }
      if (identityResult.status === 'fulfilled') {
        setIdentity(identityResult.value)
        setIdentityState('linked')
      } else if (isNoIdentity(identityResult.reason)) {
        setIdentityState('unlinked')
      } else {
        setIdentityError(
          identityResult.reason instanceof Error ? identityResult.reason.message : 'Identity service unavailable',
        )
        setIdentityState('error')
      }
      setIdentityWallet(wallet)
    })
    return () => { current = false }
  }, [identityAttempt, publicKey])

  useEffect(() => {
    void refreshOffChain()
  }, [refreshOffChain])

  const bindIdentity = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    if (!publicKey || !signMessage) {
      setBindingError('This wallet cannot sign identity requests. Choose a wallet that supports message signing.')
      return
    }
    if (credential.trim().length < 24) {
      setBindingError('Use a private, high-entropy phrase with at least 24 characters.')
      return
    }
    let recoveryKey: PublicKey
    try {
      recoveryKey = new PublicKey(recovery.trim())
    } catch {
      setBindingError('Enter a valid Solana recovery wallet address.')
      return
    }
    if (recoveryKey.equals(publicKey)) {
      setBindingError('The recovery wallet must be different from the connected wallet.')
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
      setBindingError(error instanceof Error ? error.message : 'Identity link failed')
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
          <span className="portfolio-eyebrow"><span className="live-dot" /> LAND OWNER CONSOLE</span>
          <h1>Identity &amp; portfolio</h1>
          <p>Your wallet, identity commitment and land records in one secure workspace.</p>
        </div>
        <button className="btn btn-secondary portfolio-refresh" onClick={() => void refresh()} disabled={refreshing}>
          {refreshing ? <LoaderCircle size={14} className="animate-spin" /> : <RefreshCw size={14} />}
          Refresh portfolio
        </button>
      </header>

      <section className="portfolio-overview" aria-label="Portfolio overview">
        <article className="portfolio-balance-card">
          <div className="portfolio-card-label"><Wallet size={14} /> Wallet balance</div>
          <strong>{visibleBalance == null ? (address ? '—' : 'Connect wallet') : visibleBalance.toFixed(4)}</strong>
          <span>{visibleBalance == null ? (visibleBalanceError ?? (address ? 'Reading Solana devnet' : 'Your wallet balance appears here')) : 'SOL · Solana devnet'}</span>
        </article>
        <article className="portfolio-metric-card">
          <div className="portfolio-card-label"><Landmark size={14} /> Owned land assets</div>
          <strong>{address ? ownedAssetCount : '—'}</strong>
          <span>{address ? `${myChainParcels.length} on-chain · ${standaloneMapParcels.length} mapped` : 'Connect to view holdings'}</span>
        </article>
        <article className="portfolio-metric-card">
          <div className="portfolio-card-label"><MapPin size={14} /> Mapped area</div>
          <strong>{address ? formatArea(mappedArea) : '—'}</strong>
          <span>{address ? 'Measured from linked parcel geometry' : 'Area linked to your wallet'}</span>
        </article>
        <article className="portfolio-metric-card">
          <div className="portfolio-card-label"><BadgeCheck size={14} /> Identity</div>
          <strong>{visibleIdentityState === 'linked' ? 'Linked' : visibleIdentityState === 'loading' ? 'Checking' : visibleIdentityState === 'unlinked' ? 'Not linked' : '—'}</strong>
          <span>{identityState === 'linked' ? 'Wallet identity commitment' : 'Private identity & recovery'}</span>
        </article>
      </section>

      <section className="portfolio-columns">
        <article className="portfolio-card identity-card">
          <div className="portfolio-section-head">
            <div className="portfolio-section-icon"><Fingerprint size={17} /></div>
            <div>
              <span className="portfolio-eyebrow">SELF-SOVEREIGN IDENTITY</span>
              <h2>Identity &amp; recovery</h2>
            </div>
          </div>
          {!address ? (
            <div className="portfolio-empty">
              <p>Connect a wallet to check for an existing identity or set up a recovery wallet.</p>
              <span>Your identity commitment is tied to the wallet that signs it.</span>
            </div>
          ) : visibleIdentityState === 'loading' ? (
            <div className="portfolio-inline-status"><LoaderCircle size={16} className="animate-spin" /> Looking up this wallet…</div>
          ) : visibleIdentityState === 'linked' && identity ? (
            <div className="identity-summary">
              <div className="identity-linked-badge"><BadgeCheck size={14} /> Identity linked to this wallet</div>
              <dl>
                <div><dt>Owner wallet</dt><dd title={identity.owner}>{shortAddress(identity.owner)}</dd></div>
                <div><dt>Recovery wallet</dt><dd title={identity.recovery}>{shortAddress(identity.recovery)}</dd></div>
                <div><dt>Commitment</dt><dd title={identity.identity_hash}>{identity.identity_hash.slice(0, 12)}…{identity.identity_hash.slice(-8)}</dd></div>
                <div><dt>Linked parcels</dt><dd>{identity.parcel_count}</dd></div>
                <div><dt>Created</dt><dd>{new Date(identity.created_at).toLocaleDateString()}</dd></div>
              </dl>
              <p className="identity-privacy-note"><Shield size={13} /> Credential text is not retained here. Keep your recovery wallet secure.</p>
            </div>
          ) : visibleIdentityState === 'error' ? (
            <div className="portfolio-callout portfolio-callout-error">
              <CircleHelp size={15} />
              <span>{identityError || 'Could not check the identity service.'}</span>
              <button className="btn btn-ghost" onClick={() => {
                setIdentityWallet(null)
                setIdentityAttempt((n) => n + 1)
              }}>Retry</button>
            </div>
          ) : (
            <>
              <p className="portfolio-copy">Create a private identity commitment and attach a recovery wallet. The connected wallet signs the request.</p>
              <form className="identity-form" onSubmit={(event) => void bindIdentity(event)}>
                <label>
                  Private identity phrase
                  <input
                    className="text-input"
                    type="password"
                    autoComplete="new-password"
                    value={credential}
                    onChange={(event) => setCredential(event.target.value)}
                    placeholder="Use a unique, high-entropy phrase"
                    minLength={24}
                    required
                  />
                </label>
                <label>
                  Recovery wallet address
                  <input
                    className="text-input"
                    value={recovery}
                    onChange={(event) => setRecovery(event.target.value)}
                    placeholder="Solana public address"
                    autoComplete="off"
                    required
                  />
                </label>
                <div className="identity-privacy-note">
                  <Shield size={14} />
                  <span>Only a SHA-256 commitment and wallet addresses are sent. Never use a national ID, password, or low-entropy secret: hashes of predictable values can be guessed.</span>
                </div>
                {bindingError && <div className="portfolio-callout portfolio-callout-error">{bindingError}</div>}
                <button className="btn btn-primary identity-submit" type="submit" disabled={binding || !signMessage}>
                  {binding ? <><LoaderCircle size={14} className="animate-spin" /> Signing &amp; linking…</> : <><Fingerprint size={14} /> Create identity link</>}
                </button>
                {!signMessage && <span className="text-xs text-muted">This wallet does not support signing messages.</span>}
              </form>
            </>
          )}
          {identityError && visibleIdentityState !== 'error' && <p className="portfolio-inline-error">{identityError}</p>}
        </article>

        <article className="portfolio-card wallet-card">
          <div className="portfolio-section-head">
            <div className="portfolio-section-icon"><Wallet size={17} /></div>
            <div>
              <span className="portfolio-eyebrow">CONNECTED SIGNER</span>
              <h2>Wallet</h2>
            </div>
          </div>
          {address ? (
            <div className="wallet-summary">
              <div className="wallet-profile">
                <span className="wallet-avatar">{(walletName || 'W').slice(0, 1).toUpperCase()}</span>
                <div><strong>{walletName || 'Solana wallet'}</strong><span>Connected on Solana devnet</span></div>
                <BadgeCheck size={16} className="wallet-verified-icon" />
              </div>
              <button
                className="wallet-address-copy"
                onClick={() => void navigator.clipboard.writeText(address)}
                title="Copy wallet address"
                aria-label="Copy full wallet address"
              >
                <span>{shortAddress(address)}</span><Copy size={13} />
              </button>
              <div className="wallet-security-note">
                <Shield size={15} />
                <span>Transactions require approval in your wallet. Terra never has access to your private keys.</span>
              </div>
              <a className="portfolio-explorer-link" href={`https://explorer.solana.com/address/${address}?cluster=devnet`} target="_blank" rel="noreferrer">
                View wallet on Solana Explorer <ArrowRight size={13} />
              </a>
            </div>
          ) : (
            <div className="portfolio-empty">
              <p>No wallet connected</p>
              <span>Connect Phantom, Solflare, Backpack or another Wallet Standard wallet from the top bar.</span>
            </div>
          )}
          <div className="portfolio-activity-link">
            <span><ArrowDownToLine size={14} /> Recent transactions <b>{transactionCount}</b></span>
            <Link to="/transactions">View activity <ArrowRight size={13} /></Link>
          </div>
        </article>
      </section>

      <section className="portfolio-card portfolio-assets">
        <div className="portfolio-section-head portfolio-assets-head">
          <div className="portfolio-section-icon"><BriefcaseBusiness size={17} /></div>
          <div className="portfolio-assets-title">
            <span className="portfolio-eyebrow">LAND REGISTRY</span>
            <h2>Assets</h2>
          </div>
          <div className="land-viewer-seg" role="group" aria-label="Asset list filter">
            <button className={`land-viewer-segbtn ${assetFilter === 'owned' ? 'on' : ''}`} onClick={() => setAssetFilter('owned')}>My assets</button>
            <button className={`land-viewer-segbtn ${assetFilter === 'map' ? 'on' : ''}`} onClick={() => setAssetFilter('map')}>Map records</button>
          </div>
        </div>
        <div className="portfolio-asset-toolbar">
          <label className="parcel-search portfolio-asset-search">
            <MapPin size={15} />
            <input value={assetQuery} onChange={(event) => setAssetQuery(event.target.value)} placeholder="Search land assets" aria-label="Search land assets" />
          </label>
          <Link className="btn btn-primary" to="/atlas"><MapPin size={14} /> Open land atlas</Link>
        </div>
        {!address && assetFilter === 'owned' ? (
          <div className="portfolio-empty asset-empty">
            <p>Connect a wallet to see verified holdings.</p>
            <span>Explore the pilot area to preview sample land parcels.</span>
            <Link to="/atlas">Explore demo parcels <ArrowRight size={13} /></Link>
          </div>
        ) : assetFilter === 'owned' && myChainParcels.length === 0 && mapRecords.length === 0 ? (
          <div className="portfolio-empty asset-empty">
            <p>No parcels are linked to this wallet yet.</p>
            <span>Draw a boundary and register a parcel, or browse map records.</span>
            <Link to="/atlas">Register or explore land <ArrowRight size={13} /></Link>
          </div>
        ) : (
          <div className="portfolio-asset-list">
            {myChainParcels.map((parcel) => (
              <button key={parcel.address} className="portfolio-asset-row" onClick={() => openChainParcel(parcel)}>
                <span className="asset-row-icon"><Landmark size={16} /></span>
                <span className="asset-row-main"><strong>{parcel.account.name}</strong><span>{shortAddress(parcel.address)} · on-chain ownership</span></span>
                <span className="asset-row-status">Verified owner</span><ArrowRight size={14} className="asset-row-arrow" />
              </button>
            ))}
            {mapRecords.map((parcel) => (
              <button key={parcel.id} className="portfolio-asset-row" onClick={() => openMapParcel(parcel)}>
                <span className="asset-row-icon"><MapPin size={16} /></span>
                <span className="asset-row-main"><strong>{parcel.name}</strong><span>{formatArea(parcel.area_m2)} · {parcel.holder === address ? 'linked to this wallet' : shortAddress(parcel.holder)}</span></span>
                <span className={`asset-row-status ${parcel.holder === address ? 'owned' : ''} ${demoMode ? 'demo' : ''}`}>{demoMode ? 'Demo' : parcel.holder === address ? parcel.status : 'Map record'}</span>
                <ArrowRight size={14} className="asset-row-arrow" />
              </button>
            ))}
            {assetFilter === 'map' && mapRecords.length === 0 && (
              <div className="portfolio-empty asset-empty"><p>No map records match this search.</p></div>
            )}
          </div>
        )}
        {demoMode && <div className="portfolio-demo-note"><Shield size={13} /> Demo parcels are illustrative records, not wallet-owned land or chain state.</div>}
      </section>
    </main>
  )
}
