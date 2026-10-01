import { useMemo, useState, type FormEvent } from 'react'
import { Activity, AlertTriangle, ArrowUpRight, Check, CircleHelp, Code2, Database, LockKeyhole, Play, RefreshCw, ShieldCheck } from 'lucide-react'
import { useWallet } from '../lib/wallet'
import { api } from '../lib/api'
import { BACKEND_GROUPS, BACKEND_OPERATIONS, type BackendOperation } from '../lib/backendConsole'
import { useLocale } from '../lib/locale'

type JsonObject = Record<string, unknown>
type ResultState = { operation: BackendOperation; value: unknown; at: number } | null

function isRecord(value: unknown): value is JsonObject {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function parseObjectInput(input: string, label: string): Record<string, string> {
  if (!input.trim()) return {}
  const value: unknown = JSON.parse(input)
  if (!isRecord(value)) throw new Error(`${label} must be a JSON object`)
  return Object.fromEntries(
    Object.entries(value).map(([key, entry]) => [
      key,
      typeof entry === 'string' ? entry : JSON.stringify(entry),
    ]),
  )
}

function operationPath(operation: BackendOperation, params: Record<string, string>) {
  return operation.path.replace(/:([a-zA-Z0-9_]+)/g, (_token, name: string) => {
    const value = params[name]?.trim()
    if (!value) throw new Error(`Enter a value for "${name}"`)
    if (value === '.' || value === '..' || value.includes('/')) {
      throw new Error(`"${name}" must be a single route segment`)
    }
    return encodeURIComponent(value)
  })
}

function recordTitle(value: unknown, index: number) {
  if (!isRecord(value)) return `Record ${index + 1}`
  for (const key of ['name', 'id', 'pubkey', 'address', 'parcel_id', 'zone_id', 'status']) {
    const field = value[key]
    if (typeof field === 'string' && field.trim()) return `${key}: ${field}`
  }
  return `Record ${index + 1}`
}

function responseItems(value: unknown) {
  return Array.isArray(value) ? value : null
}

export default function ProtocolConsolePage() {
  const { publicKey, signMessage } = useWallet()
  const { t } = useLocale()
  const [group, setGroup] = useState(BACKEND_GROUPS[0])
  const [operationId, setOperationId] = useState(BACKEND_OPERATIONS[0].id)
  const [pathParams, setPathParams] = useState<Record<string, string>>({})
  const [queryInput, setQueryInput] = useState(BACKEND_OPERATIONS[0].query ?? '{}')
  const [bodyInput, setBodyInput] = useState(BACKEND_OPERATIONS[0].body ?? '')
  const [confirmation, setConfirmation] = useState('')
  const [armed, setArmed] = useState(false)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [result, setResult] = useState<ResultState>(null)
  const [history, setHistory] = useState<ResultState[]>([])

  const groupOperations = useMemo(
    () => BACKEND_OPERATIONS.filter((operation) => operation.group === group),
    [group],
  )
  const selected = BACKEND_OPERATIONS.find((operation) => operation.id === operationId) ?? groupOperations[0] ?? BACKEND_OPERATIONS[0]
  const parameterNames = useMemo(
    () => [...selected.path.matchAll(/:([a-zA-Z0-9_]+)/g)].map((match) => match[1]),
    [selected],
  )
  const requiresConfirmation = selected.method !== 'GET' && selected.mutates !== false
  const requiredPhrase = requiresConfirmation ? selected.sensitive ? 'EXECUTE' : 'APPLY' : ''

  const chooseGroup = (nextGroup: string) => {
    setGroup(nextGroup)
    const firstOperation = BACKEND_OPERATIONS.find((operation) => operation.group === nextGroup)
    if (firstOperation) chooseOperation(firstOperation)
  }

  const chooseOperation = (operation: BackendOperation) => {
    setOperationId(operation.id)
    setPathParams({})
    setQueryInput(operation.query ?? '{}')
    setBodyInput(operation.body ?? '')
    setConfirmation('')
    setArmed(false)
    setError(null)
  }

  const runOperation = async (event: FormEvent) => {
    event.preventDefault()
    setError(null)
    setResult(null)

    if (selected.disabledReason) {
      setError(selected.disabledReason)
      return
    }
    if (requiresConfirmation && !publicKey) {
      setError('Connect a wallet before submitting an API-side write request.')
      return
    }
    if (selected.id === 'identity-bind' && !signMessage) {
      setError('This endpoint requires a wallet that supports message signing. Use the signed identity workflow on the Portfolio page.')
      return
    }
    if (requiresConfirmation && (!armed || confirmation !== requiredPhrase)) {
      setError(`Review the impact, acknowledge the notice, and type ${requiredPhrase} to continue.`)
      return
    }

    let path: string
    let query: Record<string, string>
    let body: unknown
    try {
      path = operationPath(selected, pathParams)
      query = parseObjectInput(queryInput, 'Query parameters')
      body = selected.method === 'GET' || !bodyInput.trim() ? undefined : JSON.parse(bodyInput)
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Invalid request input')
      return
    }

    setBusy(true)
    try {
      let value: unknown
      if (path === '/health') {
        value = await api.serviceHealth()
      } else if (selected.id === 'identity-bind' && isRecord(body) && signMessage) {
        if ('national_id' in body || 'phone' in body || 'display_name' in body) {
          throw new Error('For privacy, use the Portfolio identity workflow for optional personal details.')
        }
        const identityHash = body.identity_hash
        const owner = body.owner
        const recovery = body.recovery
        if (typeof identityHash !== 'string' || typeof owner !== 'string' || typeof recovery !== 'string') {
          throw new Error('Identity binding requires string fields: identity_hash, owner, and recovery')
        }
        value = await api.bindIdentity({ identity_hash: identityHash, owner, recovery }, signMessage)
      } else {
        value = await api.consoleRequest(path, selected.method, body, query)
      }
      const entry = { operation: selected, value, at: Date.now() }
      setResult(entry)
      setHistory((current) => [entry, ...current].slice(0, 8))
      setConfirmation('')
      setArmed(false)
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : 'Backend request failed')
    } finally {
      setBusy(false)
    }
  }

  const items = result ? responseItems(result.value) : null
  const renderedItems = items?.slice(0, 40) ?? []

  return (
    <main className="backend-console">
      <header className="backend-console-head">
        <div>
          <span className="backend-kicker"><Activity size={13} /> TERRA API · {BACKEND_GROUPS.length} {t('consoleServices').toUpperCase()}</span>
          <h1>{t('consoleTitle')}</h1>
          <p>{t('consoleDescription')}</p>
        </div>
        <span className="backend-api-badge"><span /> {t('consoleApiBadge')}</span>
      </header>

      <section className="backend-notice" aria-label="API operation safeguards">
        <span className="backend-notice-icon"><AlertTriangle size={17} /></span>
        <div>
          <strong>{t('consoleWarningTitle')}</strong>
          <p>{t('consoleWarningBody')}</p>
        </div>
      </section>

      <div className="backend-console-grid">
        <aside className="backend-service-rail">
          <div className="backend-panel-label"><Database size={13} /> {t('consoleServices').toUpperCase()}</div>
          {BACKEND_GROUPS.map((name) => {
            const count = BACKEND_OPERATIONS.filter((operation) => operation.group === name).length
            return (
              <button
                key={name}
                className={`backend-service-button${group === name ? ' active' : ''}`}
                onClick={() => chooseGroup(name)}
              >
                <span>{name}</span><small>{count}</small>
              </button>
            )
          })}
          <div className="backend-connection">
            <span className={publicKey ? 'connected' : ''}><i /></span>
            <div><strong>{publicKey ? t('consoleWalletConnected') : t('consoleReadOnlyMode')}</strong><small>{publicKey ? `${publicKey.toBase58().slice(0, 5)}…${publicKey.toBase58().slice(-4)}` : t('consoleConnectUnlock')}</small></div>
          </div>
        </aside>

        <section className="backend-console-main">
          <div className="backend-console-toolbar">
            <div>
              <span className="backend-panel-label"><Code2 size={13} /> {t('consoleWorkbench').toUpperCase()}</span>
              <h2>{group}</h2>
            </div>
            <label className="backend-operation-select">
              <span>{t('consoleOperation')}</span>
              <select
                className="select-input"
                value={selected.id}
                onChange={(event) => {
                  const operation = BACKEND_OPERATIONS.find((candidate) => candidate.id === event.currentTarget.value)
                  if (operation) chooseOperation(operation)
                }}
              >
                {groupOperations.map((operation) => (
                  <option key={operation.id} value={operation.id}>
                    {operation.method} · {operation.path}
                  </option>
                ))}
              </select>
            </label>
          </div>

          <form className="backend-request-card" onSubmit={(event) => void runOperation(event)}>
            <div className="backend-route-line">
              <span className={`backend-method ${selected.method.toLowerCase()}`}>{selected.method}</span>
              <code>{selected.path}</code>
              {selected.sensitive && <span className="backend-sensitive"><LockKeyhole size={11} /> Sensitive</span>}
            </div>
            <p className="backend-operation-summary">{selected.summary}</p>
            {selected.disabledReason && <div className="backend-disabled-note"><LockKeyhole size={13} />{selected.disabledReason}</div>}

            {parameterNames.length > 0 && (
              <div className="backend-params">
                {parameterNames.map((name) => (
                  <label key={name}>
                    <span>{name.replaceAll('_', ' ')}</span>
                    <input
                      className="text-input"
                      value={pathParams[name] ?? ''}
                      onChange={(event) => setPathParams((current) => ({ ...current, [name]: event.target.value }))}
                      autoComplete="off"
                      placeholder={`Enter ${name}`}
                    />
                  </label>
                ))}
              </div>
            )}

            <label className="backend-json-field">
              <span>{t('consoleQuery')}</span>
              <textarea
                className="backend-json-input compact"
                value={queryInput}
                onChange={(event) => setQueryInput(event.target.value)}
                spellCheck={false}
                placeholder="{}"
              />
              {selected.path.startsWith('/geo/') || selected.path === '/spatial/parcels/near' || selected.path === '/fusion/roads' || selected.path === '/fusion/pois'
                ? <small>Spatial query examples use the Soa pilot region and are only a starting point; they do not imply parcel ownership.</small>
                : null}
            </label>

            {requiresConfirmation && !selected.disabledReason && (
              <>
                <label className="backend-json-field">
                  <span>{t('consoleBody')}</span>
                  <textarea
                    className="backend-json-input"
                    value={bodyInput}
                    onChange={(event) => setBodyInput(event.target.value)}
                    spellCheck={false}
                    placeholder="{}"
                  />
                </label>
                <div className={`backend-confirm ${selected.sensitive ? 'sensitive' : ''}`}>
                  <label className="backend-acknowledge">
                    <input type="checkbox" checked={armed} onChange={(event) => setArmed(event.target.checked)} />
                    <span>{selected.sensitive ? t('consoleSensitiveAck') : t('consoleWriteAck')}</span>
                  </label>
                  <label className="backend-confirm-input">
                    <span>{t('consoleTypeToEnable')} <code>{requiredPhrase}</code></span>
                    <input
                      className="text-input"
                      value={confirmation}
                      onChange={(event) => setConfirmation(event.target.value)}
                      autoComplete="off"
                    />
                  </label>
                </div>
              </>
            )}

            <div className="backend-request-footer">
              {selected.disabledReason
                ? <span className="backend-warn-inline"><LockKeyhole size={13} /> {t('consoleDisabled')}</span>
                : requiresConfirmation && !publicKey
                ? <span className="backend-warn-inline"><CircleHelp size={13} /> {t('consoleConnectToWrite')}</span>
                : <span className="backend-readonly-note"><ShieldCheck size={13} /> {requiresConfirmation ? `${t('consoleWriteConfirm')} · ${selected.sensitive ? t('consoleSensitiveAck') : t('consoleWriteAck')}` : t('consoleReadOnly')}</span>}
              <button
                type="submit"
                className="btn btn-primary backend-run"
                disabled={busy || Boolean(selected.disabledReason) || (selected.id === 'identity-bind' && !signMessage) || (requiresConfirmation && (!publicKey || !armed || confirmation !== requiredPhrase))}
              >
                {busy ? <RefreshCw size={14} className="animate-spin" /> : <Play size={14} />}
                {busy ? t('consoleSending') : t('consoleRun')}
              </button>
            </div>
          </form>

          {error && <div className="backend-result error"><AlertTriangle size={14} /><span>{error}</span></div>}

          {result && (
            <section className="backend-response">
              <header>
                <div><span className="backend-success"><Check size={13} /> {t('consoleResponse').toUpperCase()}</span><h3>{result.operation.summary}</h3></div>
                <time>{new Intl.DateTimeFormat(undefined, { hour: '2-digit', minute: '2-digit', second: '2-digit' }).format(result.at)}</time>
              </header>
              {items ? (
                <div className="backend-record-list">
                  <div className="backend-record-count">{items.length} {t('consoleRecords')}{items.length > renderedItems.length ? ` · showing ${renderedItems.length}` : ''}</div>
                  {renderedItems.map((item, index) => (
                    <details className="backend-record" key={`${recordTitle(item, index)}-${index}`}>
                      <summary><span>{recordTitle(item, index)}</span><ArrowUpRight size={13} /></summary>
                      <pre>{JSON.stringify(item, null, 2)}</pre>
                    </details>
                  ))}
                  {items.length === 0 && <p className="backend-empty">{t('consoleNoRecords')}</p>}
                </div>
              ) : (
                <pre className="backend-response-json">{JSON.stringify(result.value, null, 2)}</pre>
              )}
            </section>
          )}

          {!result && !error && (
            <div className="backend-idle">
              <span><Database size={18} /></span>
              <strong>{t('consoleIdleTitle')}</strong>
              <p>{t('consoleIdleBody')}</p>
            </div>
          )}
        </section>

        <aside className="backend-history">
          <div className="backend-panel-label"><RefreshCw size={13} /> {t('consoleRecent').toUpperCase()}</div>
          {history.length === 0 ? (
            <p className="backend-history-empty">{t('consoleNoHistory')}</p>
          ) : history.map((entry, index) => entry && (
            <button
              className="backend-history-item"
              key={`${entry.operation.id}-${entry.at}-${index}`}
              onClick={() => {
                setGroup(entry.operation.group)
                setOperationId(entry.operation.id)
                setResult(entry)
                setError(null)
              }}
            >
              <span className={`backend-method ${entry.operation.method.toLowerCase()}`}>{entry.operation.method}</span>
              <span><strong>{entry.operation.group}</strong><small>{entry.operation.summary}</small></span>
            </button>
          ))}
          <p className="backend-history-foot">{t('consoleMemory')}</p>
        </aside>
      </div>
    </main>
  )
}
