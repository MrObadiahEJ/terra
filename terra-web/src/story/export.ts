// Story → video export (RFC-014 §6), v0: client-side deterministic capture.
//
// The exporter drives the SAME story timeline the player uses — it never
// builds a separate animation. Frames are captured from the live Cesium
// canvas via captureStream + MediaRecorder at a fixed pace, so a 24s story
// takes ~24s wall-clock to export (no server renderer, no giant frame
// buffers — bounded memory on small machines). Captions play in the link
// player; they are not burned into v0 videos (honest limitation, stated in
// the UI).

import type { StoryDefinition } from './types'
import type { StoryPlayerHandle } from '../components/story/StoryPlayer'
import { createExport, updateExport } from './api'

export const ASPECT_PRESETS = {
  '16:9': { width: 1280, height: 720 },
  '9:16': { width: 720, height: 1280 },
  '1:1': { width: 720, height: 720 },
} as const

export type ExportAspectKey = keyof typeof ASPECT_PRESETS

export interface ExportProgress {
  phase: 'rendering' | 'done' | 'failed'
  progress: number
  error?: string
}

/** v0 safety cap: exports are short social clips, not feature films. */
const MAX_EXPORT_SECONDS = 60
const FPS = 24

function pickRecorderMime(): string {
  const candidates = [
    'video/webm;codecs=vp9',
    'video/webm;codecs=vp8',
    'video/webm',
  ]
  for (const m of candidates) {
    if (typeof MediaRecorder !== 'undefined' && MediaRecorder.isTypeSupported(m)) return m
  }
  throw new Error('MediaRecorder webm capture is not supported in this browser')
}

function waitFrame(): Promise<void> {
  return new Promise((resolve) => requestAnimationFrame(() => resolve()))
}

export async function exportStoryVideo(opts: {
  definition: StoryDefinition
  player: StoryPlayerHandle
  aspect: ExportAspectKey
  onProgress?: (p: ExportProgress) => void
}): Promise<void> {
  const { definition, player, aspect, onProgress } = opts
  if (definition.duration_s > MAX_EXPORT_SECONDS) {
    throw new Error(`story longer than ${MAX_EXPORT_SECONDS}s — shorten it before export`)
  }
  const viewer = player.getViewer()
  const canvas = viewer?.scene.canvas
  if (!viewer || !canvas) {
    throw new Error('3D viewer unavailable — export needs WebGL')
  }
  if (typeof MediaRecorder === 'undefined') {
    throw new Error('MediaRecorder is unavailable in this browser')
  }
  const mimeType = pickRecorderMime()
  const preset = ASPECT_PRESETS[aspect]

  // `receiptId` is null when the story is a local demo (not in the database),
  // in which case there is no ledger row to update.
  let receiptId: string | null = null
  try {
    const receipt = await createExport(definition.id, {
      aspect,
      width: preset.width,
      height: preset.height,
      fps: FPS,
      duration_s: definition.duration_s,
    })
    receiptId = receipt.id
  } catch {
    // No ledger row to update — local/demo stories stay receipt-less.
  }

  player.setTime(0)
  const host = canvas.parentElement
  const prevStyle = host ? { width: host.style.width, height: host.style.height } : null
  if (host) {
    host.style.width = `${preset.width}px`
    host.style.height = `${preset.height}px`
  }
  viewer.resize()

  const stream = canvas.captureStream(FPS)
  const chunks: Blob[] = []
  const recorder = new MediaRecorder(stream, { mimeType, videoBitsPerSecond: 6_000_000 })
  recorder.ondataavailable = (e) => {
    if (e.data.size > 0) chunks.push(e.data)
  }

  const finished = new Promise<void>((resolve, reject) => {
    recorder.onstop = () => resolve()
    recorder.onerror = (e) =>
      reject(new Error((e as ErrorEvent).message ?? 'recorder error'))
  })

  recorder.start(250)
  const startWall = performance.now()
  try {
    // Deterministic pacing: advance story time by 1/FPS steps, waiting for
    // the wall clock so the recorder sees each frame.
    const totalFrames = Math.ceil(definition.duration_s * FPS)
    for (let frame = 0; frame <= totalFrames; frame++) {
      const t = Math.min(frame / FPS, definition.duration_s)
      player.setTime(t)
      viewer.scene.requestRender()
      await waitFrame()
      const targetWall = startWall + (frame / FPS) * 1000
      const delay = targetWall - performance.now()
      if (delay > 0) await new Promise((r) => setTimeout(r, delay))
      onProgress?.({ phase: 'rendering', progress: Math.min(1, t / definition.duration_s) })
    }
  } finally {
    if (recorder.state !== 'inactive') recorder.stop()
    if (host && prevStyle) {
      host.style.width = prevStyle.width
      host.style.height = prevStyle.height
    }
    viewer.resize()
  }

  await finished
  const blob = new Blob(chunks, { type: 'video/webm' })
  if (blob.size === 0) throw new Error('recorder produced an empty video')

  // Download locally — v0 has no server-side artifact store for videos.
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `${definition.id}-${aspect.replace(':', 'x')}.webm`
  document.body.appendChild(a)
  a.click()
  a.remove()
  window.setTimeout(() => URL.revokeObjectURL(url), 10_000)

  if (receiptId) {
    await updateExport(receiptId, { status: 'done', progress: 1 }).catch(() => undefined)
  }
  onProgress?.({ phase: 'done', progress: 1 })
}
