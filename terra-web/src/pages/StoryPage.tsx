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
import { ArrowLeft, Download, Video } from 'lucide-react'
import { exportStoryVideo, ASPECT_PRESETS, type ExportProgress } from '../story/export'

export default function StoryPage() {
  const { storyId = '' } = useParams()
  const [search] = useSearchParams()
  const { t } = useLocale()
  const playerRef = useRef<StoryPlayerHandle | null>(null)
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

  const startExport = async (aspect: keyof typeof ASPECT_PRESETS) => {
    const player = playerRef.current
    if (!player || !story) return
    setExporting({ phase: 'rendering', progress: 0 })
    try {
      await exportStoryVideo({
        definition: story,
        player,
        aspect,
        onProgress: (p) => setExporting(p),
      })
      setExporting({ phase: 'done', progress: 1 })
    } catch (e) {
      setExporting({
        phase: 'failed',
        progress: 0,
        error: e instanceof Error ? e.message : String(e),
      })
    }
  }

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
              onClick={() => startExport(a)}
              disabled={exporting?.phase === 'rendering'}
              title={t('storyExportStart')}
            >
              {a === '16:9' ? <Video size={13} /> : <Download size={13} />}
              {a}
            </button>
          ))}
          {exporting && (
            <span className="text-[11px] text-muted" data-testid="story-export-status">
              {exporting.phase === 'rendering'
                ? `${t('storyExporting')} ${Math.round(exporting.progress * 100)}%`
                : exporting.phase === 'done'
                  ? t('storyExportDone')
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
