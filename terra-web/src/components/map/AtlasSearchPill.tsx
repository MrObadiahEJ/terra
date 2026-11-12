import { useMemo, useRef, useState } from 'react'
import { Loader2, Search, X } from 'lucide-react'
import { searchLocal, searchPlaces, type SearchResult } from '../../lib/search'
import { useLocale } from '../../lib/locale'

interface Props {
  onPick: (result: SearchResult) => void
  /** Extra class applied to the form (layout hook). */
  className?: string
}

/**
 * Google-Earth-style search pill: local places/coordinates resolve instantly,
 * anything else asks the online geocoder on Enter. Kept separate from the
 * sidebar search so the atlas always has one reachable search field on mobile.
 */
export default function AtlasSearchPill({ onPick, className }: Props) {
  const { t } = useLocale()
  const inputRef = useRef<HTMLInputElement>(null)
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<SearchResult[]>([])
  const [busy, setBusy] = useState(false)
  const [open, setOpen] = useState(false)

  const localResults = useMemo(() => searchLocal(query, 5), [query])

  const pick = (result: SearchResult) => {
    onPick(result)
    setQuery(result.label)
    setResults([])
    setOpen(false)
    inputRef.current?.blur()
  }

  const submit = async () => {
    if (localResults.length > 0) {
      pick(localResults[0])
      return
    }
    const q = query.trim()
    if (!q) return
    setBusy(true)
    setOpen(true)
    try {
      const remote = await searchPlaces(q)
      setResults(remote.slice(0, 6))
      if (remote.length > 0) pick(remote[0])
    } finally {
      setBusy(false)
    }
  }

  const rows = localResults.length > 0 ? localResults : results

  return (
    <form
      className={`atlas-search-pill${className ? ` ${className}` : ''}`}
      role="search"
      onSubmit={(event) => {
        event.preventDefault()
        void submit()
      }}
    >
      <Search size={18} aria-hidden="true" />
      <input
        ref={inputRef}
        value={query}
        onChange={(event) => {
          setQuery(event.target.value)
          setResults([])
          setOpen(true)
        }}
        onFocus={() => setOpen(true)}
        onKeyDown={(event) => {
          if (event.key === 'Escape') {
            setQuery('')
            setResults([])
            setOpen(false)
            inputRef.current?.blur()
          }
        }}
        placeholder={t('searchPlaceholder')}
        aria-label={t('searchTitle')}
        autoComplete="off"
        spellCheck={false}
      />
      {busy && <Loader2 size={16} className="atlas-search-spinner" aria-hidden="true" />}
      {query && !busy && (
        <button
          type="button"
          className="atlas-search-clear"
          aria-label={t('clearAction')}
          onClick={() => {
            setQuery('')
            setResults([])
            inputRef.current?.focus()
          }}
        >
          <X size={15} />
        </button>
      )}
      {open && query.trim() && (rows.length > 0 || busy || results.length === 0) && (
        <div className="atlas-search-pill-results" role="listbox">
          {rows.map((result) => (
            <button key={result.id} type="button" role="option" onClick={() => pick(result)}>
              <span>{result.label}</span>
              <small>
                {result.source === 'coords'
                  ? t('searchCoords')
                  : localResults.length > 0
                    ? t('searchNearby')
                    : t('searchOnline')}
              </small>
            </button>
          ))}
          {rows.length === 0 && !busy && (
            <button type="button" onClick={() => void submit()}>
              <span>{query.trim()}</span>
              <small>{t('searchOnline')} ↵</small>
            </button>
          )}
        </div>
      )}
    </form>
  )
}
