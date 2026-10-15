// Story → video export (RFC-014 §6). Phase 03 refinements over the v0
// client-side capture:
//
//  - Captions and the story title are burned into the video, so the artifact
//    stands alone (v0 left captions to the link player only).
//  - Frames are composited on an offscreen canvas that the recorder captures,
//    keeping the recorder fed even while the Cesium canvas re-renders.
//  - The export receipt is always resolved — done, failed or cancelled — so
//    the ledger never leaves a row stuck in `running`.
//  - Cancellation via AbortSignal, and a PNG still of the current frame that
//    works even where MediaRecorder is unavailable.
//
// The exporter drives the SAME deterministic timeline the player uses; a 24 s
// story still takes about 24 s wall-clock to export (no server renderer, no
// giant frame buffers — bounded memory on small machines).

import type { CaptionCue, StoryDefinition } from './types'
import type { StoryPlayerHandle } from '../components/story/StoryPlayer'
import { activeCaptions, activeStop } from './timeline'
import { createExport, updateExport } from './api'

export const ASPECT_PRESETS = {
  '16:9': { width: 1280, height: 720 },
  '9:16': { width: 720, height: 1280 },
  '1:1': { width: 720, height: 720 },
} as const

export type ExportAspectKey = keyof typeof ASPECT_PRESETS

export interface ExportProgress {
  phase: 'rendering' | 'done' | 'failed' | 'cancelled'
  progress: number
  error?: string
}

/** Throw to distinguish a user cancellation from a genuine failure. */
export class ExportCancelledError extends Error {
  constructor() {
    super('export cancelled')
    this.name = 'ExportCancelledError'
  }
}

export function isExportCancelled(e: unknown): boolean {
  return e instanceof ExportCancelledError
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

/** Draw the source canvas into `w×h`, letterboxed so aspect is preserved. */
function drawContain(
  ctx: CanvasRenderingContext2D,
  src: HTMLCanvasElement,
  w: number,
  h: number,
): void {
  ctx.fillStyle = '#000'
  ctx.fillRect(0, 0, w, h)
  const sw = src.width || 1
  const sh = src.height || 1
  const scale = Math.min(w / sw, h / sh)
  const dw = sw * scale
  const dh = sh * scale
  ctx.drawImage(src, (w - dw) / 2, (h - dh) / 2, dw, dh)
}

function roundRect(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  w: number,
  h: number,
  r: number,
): void {
  ctx.beginPath()
  ctx.moveTo(x + r, y)
  ctx.arcTo(x + w, y, x + w, y + h, r)
  ctx.arcTo(x + w, y + h, x, y + h, r)
  ctx.arcTo(x, y + h, x, y, r)
  ctx.arcTo(x, y, x + w, y, r)
  ctx.closePath()
}

/** Greedy word wrap at a max pixel width. */
function wrapText(ctx: CanvasRenderingContext2D, text: string, maxWidth: number): string[] {
  const words = text.split(/\s+/).filter(Boolean)
  const lines: string[] = []
  let line = ''
  for (const word of words) {
    const candidate = line ? `${line} ${word}` : word
    if (ctx.measureText(candidate).width <= maxWidth || line === '') {
      line = candidate
    } else {
      lines.push(line)
      line = word
    }
  }
  if (line) lines.push(line)
  return lines
}

function drawCaptions(
  ctx: CanvasRenderingContext2D,
  cues: CaptionCue[],
  w: number,
  h: number,
  unit: number,
): void {
  if (cues.length === 0) return
  const text = cues.map((c) => c.text).join(' · ')
  const font = `${Math.round(20 * unit)}px -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif`
  ctx.font = font
  ctx.textBaseline = 'middle'
  const maxTextWidth = w * 0.82
  const lines = wrapText(ctx, text, maxTextWidth)
  const lineHeight = Math.round(28 * unit)
  const padX = Math.round(14 * unit)
  const padY = Math.round(10 * unit)
  const blockW = Math.min(
    maxTextWidth + padX * 2,
    Math.max(...lines.map((l) => ctx.measureText(l).width)) + padX * 2,
  )
  const blockH = lines.length * lineHeight + padY * 2
  const x = (w - blockW) / 2
  const y = h - blockH - Math.round(40 * unit)

  ctx.fillStyle = 'rgba(2, 6, 23, 0.72)'
  roundRect(ctx, x, y, blockW, blockH, Math.round(10 * unit))
  ctx.fill()

  ctx.fillStyle = '#e5eefc'
  ctx.textAlign = 'center'
  const cx = w / 2
  lines.forEach((l, i) => {
    ctx.fillText(l, cx, y + padY + lineHeight * (i + 0.5))
  })
  ctx.textAlign = 'left'
}

function drawTitle(
  ctx: CanvasRenderingContext2D,
  title: string,
  w: number,
  unit: number,
): void {
  const font = `${Math.round(15 * unit)}px -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif`
  ctx.font = font
  ctx.textBaseline = 'middle'
  const padX = Math.round(10 * unit)
  const blockH = Math.round(30 * unit)
  const textW = Math.min(ctx.measureText(title).width, w * 0.7)
  const margin = Math.round(14 * unit)
  ctx.fillStyle = 'rgba(2, 6, 23, 0.6)'
  roundRect(ctx, margin, margin, textW + padX * 2, blockH, blockH / 2)
  ctx.fill()
  ctx.fillStyle = '#e5eefc'
  ctx.fillText(title, margin + padX, margin + blockH / 2, w * 0.7)
}

/** Composite one story frame: the live canvas plus burned-in overlays. */
function renderStoryFrame(
  ctx: CanvasRenderingContext2D,
  source: HTMLCanvasElement,
  definition: StoryDefinition,
  t: number,
  preset: { width: number; height: number },
  unit: number,
): void {
  drawContain(ctx, source, preset.width, preset.height)
  drawCaptions(ctx, activeCaptions(definition.captions, t), preset.width, preset.height, unit)
  drawTitle(ctx, definition.title, preset.width, unit)
}

function downloadBlob(blob: Blob, filename: string): void {
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  document.body.appendChild(a)
  a.click()
  a.remove()
  window.setTimeout(() => URL.revokeObjectURL(url), 10_000)
}

async function resolveReceipt(
  receiptId: string | null,
  status: 'done' | 'failed' | 'cancelled',
  error?: string,
): Promise<void> {
  if (!receiptId) return
  await updateExport(receiptId, {
    status,
    progress: status === 'done' ? 1 : 0,
    error: status === 'failed' ? error : undefined,
  }).catch(() => undefined)
}

/** `receiptId` is null when the story is a local demo (not in the database). */
async function openReceipt(
  definition: StoryDefinition,
  aspect: ExportAspectKey,
): Promise<string | null> {
  try {
    const receipt = await createExport(definition.id, {
      aspect,
      width: ASPECT_PRESETS[aspect].width,
      height: ASPECT_PRESETS[aspect].height,
      fps: FPS,
      duration_s: definition.duration_s,
    })
    return receipt.id
  } catch {
    return null
  }
}

export async function exportStoryVideo(opts: {
  definition: StoryDefinition
  player: StoryPlayerHandle
  aspect: ExportAspectKey
  onProgress?: (p: ExportProgress) => void
  signal?: AbortSignal
}): Promise<void> {
  const { definition, player, aspect, onProgress, signal } = opts
  if (definition.duration_s > MAX_EXPORT_SECONDS) {
    throw new Error(`story longer than ${MAX_EXPORT_SECONDS}s — shorten it before export`)
  }
  if (signal?.aborted) throw new ExportCancelledError()
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
  const unit = Math.min(preset.width, preset.height) / 720

  const receiptId = await openReceipt(definition, aspect)

  player.setTime(0)
  const host = canvas.parentElement
  const prevStyle = host ? { width: host.style.width, height: host.style.height } : null
  if (host) {
    host.style.width = `${preset.width}px`
    host.style.height = `${preset.height}px`
  }
  viewer.resize()

  // Offscreen compositor: the recorder captures this canvas, not Cesium's, so
  // burned-in captions and the title are part of every frame.
  const comp = document.createElement('canvas')
  comp.width = preset.width
  comp.height = preset.height
  const ctx = comp.getContext('2d')
  if (!ctx) {
    if (host && prevStyle) {
      host.style.width = prevStyle.width
      host.style.height = prevStyle.height
    }
    viewer.resize()
    await resolveReceipt(receiptId, 'failed', '2D compositor unavailable')
    throw new Error('2D canvas context unavailable for export')
  }

  const stream = comp.captureStream(FPS)
  const chunks: Blob[] = []
  const recorder = new MediaRecorder(stream, { mimeType, videoBitsPerSecond: 6_000_000 })
  recorder.ondataavailable = (e) => {
    if (e.data.size > 0) chunks.push(e.data)
  }

  let recorderError: Error | null = null
  const finished = new Promise<void>((resolve) => {
    recorder.onstop = () => resolve()
    recorder.onerror = (e) => {
      recorderError = new Error((e as ErrorEvent).message ?? 'recorder error')
      resolve()
    }
  })

  let cancelled = false
  const onAbort = () => {
    cancelled = true
  }
  signal?.addEventListener('abort', onAbort, { once: true })

  recorder.start(250)
  const startWall = performance.now()
  let exportError: unknown = null
  try {
    const totalFrames = Math.ceil(definition.duration_s * FPS)
    for (let frame = 0; frame <= totalFrames; frame++) {
      if (cancelled) throw new ExportCancelledError()
      const t = Math.min(frame / FPS, definition.duration_s)
      player.setTime(t)
      viewer.scene.requestRender()
      await waitFrame()
      renderStoryFrame(ctx, canvas, definition, t, preset, unit)
      const targetWall = startWall + (frame / FPS) * 1000
      const delay = targetWall - performance.now()
      if (delay > 0) await new Promise((r) => setTimeout(r, delay))
      onProgress?.({ phase: 'rendering', progress: Math.min(1, t / definition.duration_s) })
    }
  } catch (e) {
    exportError = e
  } finally {
    if (recorder.state !== 'inactive') recorder.stop()
    if (host && prevStyle) {
      host.style.width = prevStyle.width
      host.style.height = prevStyle.height
    }
    viewer.resize()
    signal?.removeEventListener('abort', onAbort)
  }

  await finished

  if (recorderError && !exportError) exportError = recorderError

  if (exportError) {
    const cancelledErr = isExportCancelled(exportError)
    const message = exportError instanceof Error ? exportError.message : String(exportError)
    await resolveReceipt(receiptId, cancelledErr ? 'cancelled' : 'failed', message)
    onProgress?.({
      phase: cancelledErr ? 'cancelled' : 'failed',
      progress: 0,
      error: cancelledErr ? undefined : message,
    })
    throw exportError
  }

  const blob = new Blob(chunks, { type: 'video/webm' })
  if (blob.size === 0) {
    await resolveReceipt(receiptId, 'failed', 'recorder produced an empty video')
    onProgress?.({ phase: 'failed', progress: 0, error: 'recorder produced an empty video' })
    throw new Error('recorder produced an empty video')
  }

  downloadBlob(blob, `${definition.id}-${aspect.replace(':', 'x')}-${FPS}fps.webm`)
  await resolveReceipt(receiptId, 'done')
  onProgress?.({ phase: 'done', progress: 1 })
}

/**
 * Save a PNG still of the current story frame, with the same burned-in
 * overlays as the video. Works without MediaRecorder — useful where video
 * capture is unsupported.
 */
export async function exportStoryStill(opts: {
  definition: StoryDefinition
  player: StoryPlayerHandle
  aspect: ExportAspectKey
  signal?: AbortSignal
}): Promise<void> {
  const { definition, player, aspect, signal } = opts
  if (signal?.aborted) throw new ExportCancelledError()
  const viewer = player.getViewer()
  const canvas = viewer?.scene.canvas
  if (!viewer || !canvas) {
    throw new Error('3D viewer unavailable — export needs WebGL')
  }
  const preset = ASPECT_PRESETS[aspect]
  const unit = Math.min(preset.width, preset.height) / 720
  const comp = document.createElement('canvas')
  comp.width = preset.width
  comp.height = preset.height
  const ctx = comp.getContext('2d')
  if (!ctx) throw new Error('2D canvas context unavailable for export')

  renderStoryFrame(ctx, canvas, definition, player.getTime(), preset, unit)
  const blob = await new Promise<Blob | null>((resolve) => comp.toBlob(resolve, 'image/png'))
  if (!blob) throw new Error('PNG encoding failed')
  const stop = activeStop(definition.stops, player.getTime())
  const suffix = stop ? `-${stop.id}` : ''
  downloadBlob(blob, `${definition.id}-${aspect.replace(':', 'x')}${suffix}.png`)
}
