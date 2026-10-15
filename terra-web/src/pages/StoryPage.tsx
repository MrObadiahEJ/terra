// /story/:storyId — replay a saved story (RFC-014 §4).
//
// The URL is a reproducible scene specification: the same link resolves the
// same story document and timeline position. ?step=N jumps to a stop,
// ?time=T seeks to an exact second (wins over step). Replay reproduces the
// same logical journey and camera keyframes — not necessarily identical
// pixels across GPUs and screen sizes.

import { useCallback, useEffect, useRef, useState } from 'react'
import { Link, useParams, useSearchParams } from 'react-router-dom'
import { StoryPlayer, type StoryPlayerHandle } from '../components/story/StoryPlayer'
import { getStory } from '../story/api'
import { DEMO_STORIES, isDemoStoryId } from '../story/demo'
import { parseStoryLink } from '../story/url'
import type { StoryDefinition, StoryWithDefinition } from '../story/types'
import { useLocale } from '../lib/locale'
import type { TranslationKey } from '../lib/locale'
import {
  ELEVATION_SOURCE_KEYS,
  GEOMETRY_SOURCE_KEYS,
  SPATIAL_DIMENSION_KEYS,
  codeKey,
} from '../lib/spatial'
import { ArrowLeft, Download, Video, Image as ImageIcon, X } from 'lucide-react'
import {
  exportStoryVideo,
  exportStoryStill,
  isExportCancelled,
  ASPECT_PRESETS,
  type ExportProgress,
} from '../story/export'

/** Compact a long hex/ref for display. */
function shortRef(ref: string): string {
  return ref.length > 16 ? `${ref.slice(0, 10)}…${ref.slice(-4)}` : ref
}

export default function StoryPage() {
  const { storyId = '' } = useParams()
  const [search] = useSearchParams()
  const { t } = useLocale()
  const playerRef = useRef<StoryPlayerHandle | null>(null)
  const abortRef = useRef<AbortController | null>(null)
  const [story, setStory] = useState<StoryDefinition | null>(null)
  const [meta, setMeta] = useState<StoryWithDefinition | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)
  const [exporting, setExporting] = useState<ExportProgress | null>(null)
  const urlTimer = useRef(0)

  // Deep link: time wins over step (parseStoryLink already prefers time).
  const linkState = parseStoryLink(search.toString())

  useEffect(() => {
    let cancelled = false
    // Deferred so the effect never sets state synchronously (eslint-safe).
    queueMicrotask(() => {
      if (cancelled) return
      setLoading(true)
      setError(null)
    })
    const embedded = isDemoStoryId(storyId) ? DEMO_STORIES[storyId]?.() : undefined
    if (embedded) {
      queueMicrotask(() => {
        if (cancelled) return
        setStory(embedded)
        setMeta(null)
        setLoading(false)
      })
      return () => {
        cancelled = true
      }
    }
    getStory(storyId)
      .then((row) => {
        if (cancelled) return
        setStory(row.definition)
        setMeta(row)
      })
      .catch((e: unknown) => {
        if (!cancelled) setError(e instanceof Error ? e.message : String(e))
      })
      .finally(() => {
        if (!cancelled) setLoading(false)
      })
    return () => {
      cancelled = true
    }
  }, [storyId])

  const initialTimeS = (() => {
    if (!story) return 0
    if (linkState.time !== undefined) return Math.min(linkState.time, story.duration_s)
    if (linkState.step !== undefined) {
      const stop = story.stops[Math.min(linkState.step, story.stops.length - 1)]
      return stop?.time_s ?? 0
    }
    return 0
  })()

  // Keep the address bar in sync with the playing position (debounced) so a
  // copied URL captures the current moment without spamming history.
  const onTimeChange = useCallback(
    (time: number) => {
      window.clearTimeout(urlTimer.current)
      urlTimer.current = window.setTimeout(() => {
        const params = new URLSearchParams(window.location.search)
        params.set('time', time.toFixed(1))
        window.history.replaceState(null, '', `${window.location.pathname}?${params}`)
      }, 500)
    },
    [],
  )

  const runExport = async (kind: 'video' | 'still', aspect: keyof typeof ASPECT_PRESETS) => {
    const player = playerRef.current
    if (!player || !story) return
    const controller = new AbortController()
    abortRef.current = controller
    setExporting({ phase: 'rendering', progress: 0 })
    try {
      if (kind === 'still') {
        await exportStoryStill({ definition: story, player, aspect, signal: controller.signal })
        setExporting({ phase: 'done', progress: 1 })
      } else {
        await exportStoryVideo({
          definition: story,
          player,
          aspect,
          signal: controller.signal,
          onProgress: (p) => setExporting(p),
        })
        setExporting({ phase: 'done', progress: 1 })
      }
    } catch (e) {
      if (isExportCancelled(e)) {
        setExporting({ phase: 'cancelled', progress: 0 })
      } else {
        setExporting({
          phase: 'failed',
          progress: 0,
          error: e instanceof Error ? e.message : String(e),
        })
      }
    } finally {
      abortRef.current = null
    }
  }

  const cancelExport = () => abortRef.current?.abort()

  if (loading) {
    return <div className="p-6 text-sm text-muted">{t('loadingEllipsis')}</div>
  }
  if (error || !story) {
    return (
      <div className="p-6 text-sm space-y-3">
        <p className="text-red-500">{error ?? t('storyMissing')}</p>
        <Link className="btn btn-secondary inline-flex gap-1" to="/stories">
          <ArrowLeft size={14} /> {t('storyBackToList')}
        </Link>
      </div>
    )
  }

  const provenanceStops = story.stops.filter((s) => s.provenance)

  return (
    <div className="flex flex-col h-full min-h-0">
      <div className="story-header">
        <div className="min-w-0">
          <div className="flex items-center gap-2 flex-wrap">
            <Link className="btn btn-ghost p-1" to="/stories" title={t('storyBackToList')}>
              <ArrowLeft size={14} />
            </Link>
            <h1 className="font-semibold truncate">{story.title}</h1>
            <span className="story-type-badge">{t(`storyType${story.story_type === 'journey' ? 'Journey' : story.story_type === 'history' ? 'History' : 'Discovery'}` as never)}</span>
            {isDemoStoryId(story.id) && <span className="story-demo-badge">{t('storyDemoBadge')}</span>}
          </div>
          {story.description && (
            <p className="text-[12px] text-muted mt-0.5">{story.description}</p>
          )}
        </div>
        <div className="story-export">
          <span className="text-[11px] text-muted hidden sm:inline">{t('storyExport')}</span>
          {(Object.keys(ASPECT_PRESETS) as (keyof typeof ASPECT_PRESETS)[]).map((a) => (
            <button
              key={a}
              className="btn btn-secondary px-2 py-1 gap-1 text-[12px]"
              onClick={() => runExport('video', a)}
              disabled={exporting?.phase === 'rendering'}
              title={t('storyExportStart')}
            >
              {a === '16:9' ? <Video size={13} /> : <Download size={13} />}
              {a}
            </button>
          ))}
          <button
            className="btn btn-ghost px-2 py-1 gap-1 text-[12px]"
            onClick={() => runExport('still', '16:9')}
            disabled={exporting?.phase === 'rendering'}
            title={t('storyExportStillTitle')}
          >
            <ImageIcon size={13} />
            PNG
          </button>
          {exporting?.phase === 'rendering' && (
            <button
              className="btn btn-ghost px-2 py-1 gap-1 text-[12px]"
              onClick={cancelExport}
              title={t('storyExportCancel')}
            >
              <X size={13} />
            </button>
          )}
          {exporting && (
            <span className="text-[11px] text-muted" data-testid="story-export-status">
              {exporting.phase === 'rendering'
                ? `${t('storyExporting')} ${Math.round(exporting.progress * 100)}%`
                : exporting.phase === 'done'
                  ? t('storyExportDone')
                  : exporting.phase === 'cancelled'
                    ? t('storyExportCancelled')
                    : `${t('storyExportFail')}: ${exporting.error ?? ''}`}
            </span>
          )}
        </div>
      </div>

      <p className="story-note">{t('storyPlayerNote')}</p>

      <div className="story-stage">
        <StoryPlayer
          ref={playerRef}
          definition={story}
          initialTimeS={initialTimeS}
          onTimeChange={onTimeChange}
        />
      </div>

      {provenanceStops.length > 0 && (
        <section className="story-provenance" aria-label={t('storyProvenanceTitle')}>
          <h2 className="story-prov-title">{t('storyProvenanceTitle')}</h2>
          <p className="story-prov-hint">{t('storyProvenanceHint')}</p>
          <ul className="story-prov-list">
            {provenanceStops.map((s) => {
              const p = s.provenance
              if (!p) return null
              return (
                <li key={s.id} className="story-prov-row">
                  <div className="story-prov-head">
                    <span className="font-medium">{s.label}</span>
                    {s.geometry_version !== undefined && (
                      <span className="story-stop-ver">v{s.geometry_version}</span>
                    )}
                    <span
                      className={`story-prov-state ${p.verified ? 'is-verified' : 'is-unverified'}`}
                    >
                      {p.verified ? t('storyProvVerified') : t('storyProvUnverified')}
                    </span>
                  </div>
                  <div className="story-prov-meta">
                    <span>
                      {t('storyProvSource')}:{' '}
                      {t(codeKey(GEOMETRY_SOURCE_KEYS, p.source) as TranslationKey)}
                    </span>
                    <span>
                      {t('storyProvDimension')}:{' '}
                      {t(codeKey(SPATIAL_DIMENSION_KEYS, p.dimension) as TranslationKey)}
                    </span>
                    <span>
                      {t('storyProvElevation')}:{' '}
                      {t(codeKey(ELEVATION_SOURCE_KEYS, p.elevation_source) as TranslationKey)}
                    </span>
                    <span>{t('storyProvAttestations', { n: p.attest_count })}</span>
                  </div>
                  {(p.verified_by || p.evidence_manifest || p.geometry_hash) && (
                    <div className="story-prov-refs">
                      {p.verified_by && (
                        <span className="font-mono">
                          {t('storyProvVerifiedBy')}: {shortRef(p.verified_by)}
                        </span>
                      )}
                      {p.evidence_manifest && (
                        <span className="font-mono">
                          {t('storyProvEvidence')}: {shortRef(p.evidence_manifest)}
                        </span>
                      )}
                      {p.geometry_hash && (
                        <span className="font-mono">
                          {t('storyProvHash')}: {shortRef(p.geometry_hash)}
                        </span>
                      )}
                    </div>
                  )}
                </li>
              )
            })}
          </ul>
        </section>
      )}

      {meta && (
        <p className="story-meta">
          {t('storyMetaLine', {
            stops: String(meta.stop_count),
            duration: meta.duration_s.toFixed(0),
          })}
        </p>
      )}
    </div>
  )
}
