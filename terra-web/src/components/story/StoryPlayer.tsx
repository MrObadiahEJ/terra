// Story player (RFC-014): deterministic camera timeline over a minimal
// Cesium mount. Camera state is a pure function of story time — playback,
// scrubbing and deep-link seeks all go through evaluateCamera(), never
// through frame-count animation. Route polylines and camera tracks stay
// separate entities: a great-circle transition is drawn dashed and labelled
// "illustrative", never as a drivable path.

import {
  forwardRef,
  useCallback,
  useEffect,
  useImperativeHandle,
  useMemo,
  useRef,
  useState,
} from 'react'
import * as Cesium from 'cesium'
import 'cesium/Build/Cesium/Widgets/widgets.css'
import { useLocale } from '../../lib/locale'
import type { TranslationKey } from '../../lib/locale'
import { GEOMETRY_SOURCE_KEYS, codeKey } from '../../lib/spatial'
import { Pause, Play, SkipBack, SkipForward, Link2, Gauge } from 'lucide-react'
import {
  activeCaptions,
  activeStop,
  evaluateCamera,
  stopIndexAt,
  type CameraState,
} from '../../story/timeline'
import { buildStoryLink } from '../../story/url'
import type { StoryDefinition } from '../../story/types'

export interface StoryPlayerHandle {
  setTime(t: number): void
  getTime(): number
  isPlaying(): boolean
  getViewer(): Cesium.Viewer | null
}

interface Props {
  definition: StoryDefinition
  initialTimeS?: number
  /** Reports story time changes (scrub, playback, step). */
  onTimeChange?: (t: number) => void
  /** Bumped to resize the canvas (video export switches aspect ratios). */
  sizeKey?: number
}

const SPEEDS = [0.5, 1, 2] as const

function applyCamera(viewer: Cesium.Viewer, cam: CameraState): void {
  viewer.camera.setView({
    destination: Cesium.Cartesian3.fromDegrees(cam.lon, cam.lat, cam.heightM),
    orientation: {
      heading: Cesium.Math.toRadians(cam.headingDeg),
      pitch: Cesium.Math.toRadians(cam.pitchDeg),
      roll: Cesium.Math.toRadians(cam.rollDeg),
    },
  })
}

export const StoryPlayer = forwardRef<StoryPlayerHandle, Props>(function StoryPlayer(
  { definition, initialTimeS = 0, onTimeChange, sizeKey },
  ref,
) {
  const { t } = useLocale()
  const containerRef = useRef<HTMLDivElement | null>(null)
  const viewerRef = useRef<Cesium.Viewer | null>(null)
  const [webglError, setWebglError] = useState<string | null>(null)
  const [timeS, setTimeS] = useState(() =>
    Math.min(Math.max(initialTimeS, 0), definition.duration_s),
  )
  const [playing, setPlaying] = useState(false)
  const [speedIdx, setSpeedIdx] = useState(1)
  const [copied, setCopied] = useState(false)

  const timeRef = useRef(timeS)
  const playingRef = useRef(false)
  const speedRef = useRef(SPEEDS[speedIdx])
  const rafRef = useRef(0)
  const wallAnchorRef = useRef({ wall: 0, story: 0 })

  timeRef.current = timeS
  playingRef.current = playing
  speedRef.current = SPEEDS[speedIdx]

  const seek = useCallback(
    (next: number) => {
      const clamped = Math.min(Math.max(next, 0), definition.duration_s)
      timeRef.current = clamped
      setTimeS(clamped)
      onTimeChange?.(clamped)
      const viewer = viewerRef.current
      if (viewer) applyCamera(viewer, evaluateCamera(definition.camera_track, clamped))
    },
    [definition, onTimeChange],
  )

  useImperativeHandle(
    ref,
    (): StoryPlayerHandle => ({
      setTime: seek,
      getTime: () => timeRef.current,
      isPlaying: () => playingRef.current,
      getViewer: () => viewerRef.current,
    }),
    [seek],
  )

  // ---- minimal Cesium mount ------------------------------------------------
  useEffect(() => {
    const container = containerRef.current
    if (!container) return
    let viewer: Cesium.Viewer
    try {
      viewer = new Cesium.Viewer(container, {
        baseLayerPicker: false,
        geocoder: false,
        homeButton: false,
        sceneModePicker: false,
        navigationHelpButton: false,
        animation: false,
        timeline: false,
        fullscreenButton: false,
        infoBox: false,
        selectionIndicator: false,
        baseLayer: new Cesium.ImageryLayer(
          new Cesium.UrlTemplateImageryProvider({
            url: 'https://tile.openstreetmap.org/{z}/{x}/{y}.png',
            maximumLevel: 18,
            credit: '© OpenStreetMap contributors',
          }),
        ),
        terrain: undefined,
      })
      viewer.scene.globe.enableLighting = true
      viewer.scene.screenSpaceCameraController.minimumZoomDistance = 1
    } catch (err) {
      container.replaceChildren()
      const msg = err instanceof Error ? err.message : String(err)
      queueMicrotask(() => setWebglError(msg))
      return
    }
    viewerRef.current = viewer
    if (import.meta.env.DEV) {
      ;(window as unknown as { __terraViewer?: Cesium.Viewer }).__terraViewer = viewer
    }
    applyCamera(viewer, evaluateCamera(definition.camera_track, timeRef.current))

    // Stop markers.
    for (const s of definition.stops) {
      viewer.entities.add({
        id: `story-stop-${s.id}`,
        position: Cesium.Cartesian3.fromDegrees(s.lon, s.lat, s.height_m),
        point: {
          pixelSize: 9,
          color: s.kind === 'arrival' ? Cesium.Color.ORANGE : Cesium.Color.fromCssColorString('#2dd4bf'),
          outlineColor: Cesium.Color.BLACK,
          outlineWidth: 2,
          disableDepthTestDistance: Number.POSITIVE_INFINITY,
        },
        label: {
          text: s.label,
          font: '12px sans-serif',
          pixelOffset: new Cesium.Cartesian2(0, -18),
          outlineWidth: 3,
          style: Cesium.LabelStyle.FILL_AND_OUTLINE,
          disableDepthTestDistance: Number.POSITIVE_INFINITY,
        },
      })
    }

    // Navigation paths — dashed + unlabelled-in-3D for illustrative legs.
    for (const r of definition.routes) {
      const positions = r.coordinates.map(([lon, lat]: [number, number]) =>
        Cesium.Cartesian3.fromDegrees(lon, lat, 0),
      )
      const illustrative = r.method === 'great-circle'
      viewer.entities.add({
        id: `story-route-${r.id}`,
        polyline: {
          positions,
          width: illustrative ? 3 : 4,
          material: illustrative
            ? new Cesium.PolylineDashMaterialProperty({
                color: Cesium.Color.fromCssColorString('#f59e0b'),
              })
            : Cesium.Color.fromCssColorString('#38bdf8'),
          clampToGround: !illustrative,
        },
      })
    }

    return () => {
      viewer.destroy()
      viewerRef.current = null
    }
    // Rebuild only when the story identity changes; time updates go through seek().
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [definition.id])

  // ---- deterministic playback loop ----------------------------------------
  useEffect(() => {
    if (!playing) return
    wallAnchorRef.current = { wall: performance.now(), story: timeRef.current }
    const tick = () => {
      const { wall, story } = wallAnchorRef.current
      const elapsed = ((performance.now() - wall) / 1000) * speedRef.current
      const next = story + elapsed
      if (next >= definition.duration_s) {
        seek(definition.duration_s)
        setPlaying(false)
        return
      }
      seek(next)
      rafRef.current = requestAnimationFrame(tick)
    }
    rafRef.current = requestAnimationFrame(tick)
    return () => cancelAnimationFrame(rafRef.current)
  }, [playing, definition.duration_s, seek])

  // While playing, the timeline owns the camera; free-look only when paused.
  useEffect(() => {
    const viewer = viewerRef.current
    if (!viewer) return
    const ctrl = viewer.scene.screenSpaceCameraController
    ctrl.enableRotate = !playing
    ctrl.enableTranslate = !playing
    ctrl.enableZoom = !playing
    ctrl.enableTilt = !playing
  }, [playing])

  // Caption overlay state is derived — no extra effects (eslint-safe).
  const captions = activeCaptions(definition.captions, timeS)
  const currentStop = activeStop(definition.stops, timeS)
  const stepIdx = stopIndexAt(definition.stops, timeS)

  const togglePlay = () => {
    if (!playing && timeRef.current >= definition.duration_s - 1e-3) seek(0)
    setPlaying((p) => !p)
  }

  const copyLink = async () => {
    const path = buildStoryLink(definition.id, { time: timeRef.current })
    const url = `${window.location.origin}${path}`
    try {
      await navigator.clipboard.writeText(url)
      setCopied(true)
      window.setTimeout(() => setCopied(false), 1500)
    } catch {
      window.prompt(t('storyCopyLinkPrompt'), url)
    }
  }

  const fmt = (s: number) => `${Math.floor(s / 60)}:${(s % 60).toFixed(1).padStart(4, '0')}`

  const captionText = useMemo(
    () => captions.map((c) => c.text).join(' · '),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [timeS],
  )

  return (
    <div className="story-player">
      <div
        ref={containerRef}
        className="story-player-canvas"
        data-testid="story-canvas"
        style={webglError ? { display: 'none' } : undefined}
      />
      {webglError && (
        <div className="story-player-fallback" role="status">
          <p>{t('storyWebglFallback')}</p>
          <p className="text-[12px] opacity-70">{webglError}</p>
        </div>
      )}

      {captionText && <div className="story-caption">{captionText}</div>}
      {currentStop && (
        <div className="story-stop-badge">
          <span className="story-stop-kind">{currentStop.kind}</span>
          <span>{currentStop.label}</span>
          {currentStop.geometry_version !== undefined && (
            <span className="story-stop-ver">v{currentStop.geometry_version}</span>
          )}
          {currentStop.provenance && (
            <span
              className={`story-stop-prov ${currentStop.provenance.verified ? 'is-verified' : 'is-unverified'}`}
            >
              {t(codeKey(GEOMETRY_SOURCE_KEYS, currentStop.provenance.source) as TranslationKey)}
              {' · '}
              {currentStop.provenance.verified
                ? t('storyProvVerified')
                : t('storyProvUnverified')}
              {' · '}
              {t('storyProvAttestations', { n: currentStop.provenance.attest_count })}
            </span>
          )}
        </div>
      )}

      <div className="story-transport">
        <button
          className="btn btn-ghost p-1"
          onClick={() => seek((definition.stops[Math.max(0, stepIdx - 1)]?.time_s) ?? 0)}
          disabled={stepIdx <= 0}
          title={t('storyPrevStop')}
        >
          <SkipBack size={15} />
        </button>
        <button
          className="btn btn-secondary px-2 py-1 gap-1"
          onClick={togglePlay}
          data-testid="story-play"
        >
          {playing ? <Pause size={14} /> : <Play size={14} />}
          {playing ? t('storyPause') : t('storyPlay')}
        </button>
        <button
          className="btn btn-ghost p-1"
          onClick={() => seek(definition.stops[Math.min(definition.stops.length - 1, stepIdx + 1)]?.time_s ?? definition.duration_s)}
          disabled={stepIdx >= definition.stops.length - 1}
          title={t('storyNextStop')}
        >
          <SkipForward size={15} />
        </button>

        <input
          type="range"
          min={0}
          max={definition.duration_s}
          step={0.1}
          value={timeS}
          onChange={(e) => seek(Number(e.target.value))}
          className="story-scrub flex-1"
          aria-label={t('storyScrub')}
          data-testid="story-scrub"
        />
        <span className="story-time" data-testid="story-time">
          {fmt(timeS)} / {fmt(definition.duration_s)}
        </span>

        <button
          className="btn btn-ghost p-1"
          onClick={() => setSpeedIdx((i) => (i + 1) % SPEEDS.length)}
          title={t('storySpeed')}
        >
          <Gauge size={14} />
          <span className="text-[11px] ml-0.5">{speedRef.current}×</span>
        </button>
        <button className="btn btn-ghost p-1" onClick={copyLink} title={t('storyShare')}>
          <Link2 size={14} />
          <span className="text-[11px] ml-0.5">{copied ? t('storyCopied') : t('storyShare')}</span>
        </button>
      </div>

      {/* sizeKey exists so the export flow can resize the host without remounting. */}
      <span hidden data-size-key={sizeKey ?? 0} />
    </div>
  )
})
