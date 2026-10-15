// /stories — story library + minimal creator (RFC-014).
//
// Demo stories always work offline; saved stories come from the API. The
// creator turns ordered stops into a camera track with the same deterministic
// builder the engine uses — no hidden animation magic.

import { useCallback, useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { Film, History, Map as MapIcon, Plus, Trash2, Compass } from 'lucide-react'
import { listStories, createStory, deleteStory } from '../story/api'
import { demoHistory, demoJourney } from '../story/demo'
import { buildTrackFromStops, bearingDeg, suggestLegSeconds, haversineM } from '../story/timeline'
import type { StoryDefinition, StoryRow, StoryStop } from '../story/types'
import { useLocale } from '../lib/locale'
import type { TranslationKey } from '../lib/locale'

const TYPE_ICONS = { journey: Film, history: History, discovery: Compass } as const
const TYPE_KEYS: Record<string, TranslationKey> = {
  journey: 'storyTypeJourney',
  history: 'storyTypeHistory',
  discovery: 'storyTypeDiscovery',
}

/** Preset creator points around the atlas demo focus (Basel). */
const PRESET_POINTS: Array<{ label: string; lon: number; lat: number }> = [
  { label: 'Gate', lon: 9.515, lat: 47.141 },
  { label: 'Bridge', lon: 9.5205, lat: 47.1438 },
  { label: 'Square', lon: 9.525, lat: 47.14 },
  { label: 'Park', lon: 9.512, lat: 47.145 },
]

export default function StoriesPage() {
  const { t } = useLocale()
  const [rows, setRows] = useState<StoryRow[]>([])
  const [apiError, setApiError] = useState<string | null>(null)
  const [title, setTitle] = useState('')
  const [picked, setPicked] = useState<number[]>([0, 1])
  const [saving, setSaving] = useState(false)
  const [notice, setNotice] = useState<string | null>(null)

  const refresh = useCallback(() => {
    listStories()
      .then((r) => {
        setRows(r)
        setApiError(null)
      })
      .catch((e: unknown) => setApiError(e instanceof Error ? e.message : String(e)))
  }, [])

  useEffect(refresh, [refresh])

  const togglePoint = (idx: number) => {
    setPicked((p) => (p.includes(idx) ? p.filter((x) => x !== idx) : [...p, idx].sort()))
  }

  const saveJourney = async () => {
    if (picked.length < 2) return
    setSaving(true)
    setNotice(null)
    try {
      const ordered = picked.map((i) => PRESET_POINTS[i])
      let clock = 0
      const stops: StoryStop[] = ordered.map((p, i) => {
        const kind = i === 0 ? 'start' : i === ordered.length - 1 ? 'arrival' : 'waypoint'
        if (i > 0) {
          const prev = ordered[i - 1]
          clock += suggestLegSeconds([prev.lon, prev.lat], [p.lon, p.lat])
        }
        return {
          id: `stop-${i}`,
          label: p.label,
          kind,
          time_s: Number(clock.toFixed(1)),
          lon: p.lon,
          lat: p.lat,
          height_m: 450,
          heading_deg:
            i < ordered.length - 1
              ? bearingDeg([p.lon, p.lat], [ordered[i + 1].lon, ordered[i + 1].lat])
              : undefined,
          pitch_deg: -30,
        }
      })
      const duration = Math.max(clock + 3, 8)
      const id = `story-${Date.now().toString(36)}`
      const def: StoryDefinition = {
        schema_version: 1,
        id,
        story_type: 'journey',
        title: title.trim() || 'Untitled journey',
        stops,
        camera_track: buildTrackFromStops(stops),
        routes: ordered.slice(1).map((p, i) => {
          const prev = ordered[i]
          return {
            id: `leg-${i}`,
            method: 'great-circle' as const,
            coordinates: [
              [prev.lon, prev.lat],
              [p.lon, p.lat],
            ] as [number, number][],
            length_m: haversineM([prev.lon, prev.lat], [p.lon, p.lat]),
            label: 'Illustrative transition (aerial)',
          }
        }),
        captions: [],
        duration_s: Number(duration.toFixed(1)),
      }
      await createStory(def)
      setTitle('')
      setNotice(t('storyCreated'))
      refresh()
    } catch (e) {
      setNotice(e instanceof Error ? e.message : String(e))
    } finally {
      setSaving(false)
    }
  }

  const remove = async (id: string) => {
    try {
      await deleteStory(id)
      refresh()
    } catch (e) {
      setNotice(e instanceof Error ? e.message : String(e))
    }
  }

  const demos: StoryDefinition[] = [demoJourney(), demoHistory()]

  return (
    <div className="p-4 max-w-3xl mx-auto space-y-6 text-sm">
      <div>
        <h1 className="text-lg font-semibold">{t('storyListTitle')}</h1>
        <p className="text-[12px] text-muted">{t('storyListHint')}</p>
      </div>

      <section className="space-y-2">
        <h2 className="font-semibold">{t('storyDemos')}</h2>
        <div className="grid gap-2 sm:grid-cols-2">
          {demos.map((d) => {
            const Icon = TYPE_ICONS[d.story_type] ?? Film
            return (
              <Link key={d.id} to={`/story/${d.id}`} className="story-card">
                <div className="flex items-center gap-2">
                  <Icon size={15} />
                  <span className="font-medium truncate">{d.title}</span>
                </div>
                <div className="story-card-meta">
                  {t(TYPE_KEYS[d.story_type] ?? 'storyTypeJourney')} · {d.stops.length}{' '}
                  {t('storyStopsUnit')} · {d.duration_s.toFixed(0)}s
                </div>
                {d.description && (
                  <div className="text-[11px] text-muted mt-1">{d.description}</div>
                )}
              </Link>
            )
          })}
        </div>
      </section>

      <section className="space-y-2">
        <h2 className="font-semibold">{t('storySaved')}</h2>
        {apiError && <p className="text-[12px] text-amber-600">{t('storyApiOffline')}: {apiError}</p>}
        {rows.length === 0 && !apiError && (
          <p className="text-[12px] text-muted">{t('storyEmpty')}</p>
        )}
        <div className="space-y-1.5">
          {rows.map((row) => {
            const Icon = TYPE_ICONS[row.story_type] ?? Film
            return (
              <div key={row.id} className="plan-row">
                <div className="min-w-0 flex items-center gap-2">
                  <Icon size={14} className="shrink-0" />
                  <div className="min-w-0">
                    <Link to={`/story/${row.id}`} className="font-medium truncate hover:underline">
                      {row.title}
                    </Link>
                    <div className="plan-meta">
                      {t(TYPE_KEYS[row.story_type] ?? 'storyTypeJourney')} · {row.stop_count}{' '}
                      {t('storyStopsUnit')} · {row.duration_s.toFixed(0)}s
                    </div>
                  </div>
                </div>
                <button
                  className="btn btn-ghost p-1 shrink-0"
                  onClick={() => remove(row.id)}
                  title={t('storyDelete')}
                >
                  <Trash2 size={13} />
                </button>
              </div>
            )
          })}
        </div>
      </section>

      <section className="story-create space-y-2">
        <h2 className="font-semibold flex items-center gap-1">
          <Plus size={14} /> {t('storyCreateTitle')}
        </h2>
        <p className="text-[12px] text-muted">{t('storyCreateHint')}</p>
        <input
          className="text-input w-full"
          placeholder={t('storyCreateTitlePlaceholder')}
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          maxLength={200}
        />
        <div className="flex flex-wrap gap-1.5">
          {PRESET_POINTS.map((p, i) => (
            <button
              key={p.label}
              className={`btn px-2 py-1 text-[12px] gap-1 ${picked.includes(i) ? 'btn-secondary' : 'btn-ghost'}`}
              onClick={() => togglePoint(i)}
            >
              <MapIcon size={12} />
              {p.label}
            </button>
          ))}
        </div>
        <button
          className="btn btn-secondary inline-flex gap-1"
          onClick={saveJourney}
          disabled={saving || picked.length < 2}
        >
          <Plus size={14} /> {saving ? t('loadingEllipsis') : t('storyCreateSubmit')}
        </button>
        {notice && <p className="text-[12px] text-muted">{notice}</p>}
      </section>

      <p className="text-[11px] text-muted border-t pt-3">{t('storyEngineNote')}</p>
    </div>
  )
}
