export type GraphicsTier = 'low' | 'medium' | 'high' | 'ultra'
export type TierPreference = 'auto' | GraphicsTier

export interface TierSettings {
  terrain: boolean
  imageryMaxLevel: number
  requestRenderMode: boolean
  resolutionScale: number
  msaaSamples: number
  labelMinLod: number
  buildings: boolean
  buildingCap: number
}

export const TIER_SETTINGS: Record<GraphicsTier, TierSettings> = {
  low: {
    terrain: false,
    imageryMaxLevel: 18,
    requestRenderMode: true,
    resolutionScale: 0.75,
    msaaSamples: 1,
    labelMinLod: 5,
    buildings: false,
    buildingCap: 0,
  },
  medium: {
    terrain: true,
    imageryMaxLevel: 19,
    requestRenderMode: true,
    resolutionScale: 1,
    msaaSamples: 1,
    labelMinLod: 4,
    buildings: true,
    buildingCap: 600,
  },
  high: {
    terrain: true,
    imageryMaxLevel: 20,
    requestRenderMode: false,
    resolutionScale: 1,
    msaaSamples: 4,
    labelMinLod: 4,
    buildings: true,
    buildingCap: 1200,
  },
  ultra: {
    terrain: true,
    imageryMaxLevel: 20,
    requestRenderMode: false,
    resolutionScale: 1.5,
    msaaSamples: 4,
    labelMinLod: 4,
    buildings: true,
    buildingCap: 2000,
  },
}

const TIER_RANK: Record<GraphicsTier, number> = { low: 0, medium: 1, high: 2, ultra: 3 }
const PREF_KEY = 'terra-atlas-graphics'

export function tierFromRank(rank: number): GraphicsTier {
  if (rank <= 0) return 'low'
  if (rank === 1) return 'medium'
  if (rank === 2) return 'high'
  return 'ultra'
}

export function stepDownTier(tier: GraphicsTier): GraphicsTier {
  return tierFromRank(TIER_RANK[tier] - 1)
}

export function loadTierPreference(): TierPreference {
  try {
    const saved = localStorage.getItem(PREF_KEY)
    if (saved === 'auto' || saved === 'low' || saved === 'medium' || saved === 'high' || saved === 'ultra') {
      return saved
    }
  } catch {
    /* private mode */
  }
  return 'auto'
}

export function saveTierPreference(pref: TierPreference): void {
  try {
    localStorage.setItem(PREF_KEY, pref)
  } catch {
    /* private mode */
  }
}

function readGpuString(): string | null {
  try {
    const canvas = document.createElement('canvas')
    const gl = (canvas.getContext('webgl2') ?? canvas.getContext('webgl')) as WebGLRenderingContext | null
    if (!gl) return null
    let value: string | null = null
    const dbg = gl.getExtension('WEBGL_debug_renderer_info')
    value = String(
      dbg ? gl.getParameter(dbg.UNMASKED_RENDERER_WEBGL) : gl.getParameter(gl.RENDERER),
    )
    gl.getExtension('WEBGL_lose_context')?.loseContext()
    return value
  } catch {
    return null
  }
}

function isSoftwareRenderer(gpu: string | null): boolean {
  if (!gpu) return false
  return /swiftshader|llvmpipe|software|microsoft basic render|softwarerasterizer|google swangle/i.test(
    gpu,
  )
}

/** Highest tier "auto" may pick on this machine. Never "ultra" — that stays manual. */
export function detectHardwareCap(): GraphicsTier {
  const gpu = readGpuString()
  if (gpu === null || isSoftwareRenderer(gpu)) return 'low'
  const nav = navigator as Navigator & { deviceMemory?: number }
  const mem = nav.deviceMemory
  const cores = navigator.hardwareConcurrency
  if ((mem !== undefined && mem <= 2) || (cores !== undefined && cores <= 2)) return 'low'
  if ((mem !== undefined && mem <= 4) || (cores !== undefined && cores <= 4)) return 'medium'
  return 'high'
}

interface FpsWatchOptions {
  onFps?: (fps: number | null) => void
  onDowngrade?: () => void
  allowDowngrade?: boolean
  minFps?: number
  windowMs?: number
  sustainMs?: number
  graceMs?: number
}

const FRAME_TARGET = 1000 / 60

export class FpsWatchdog {
  private samples: number[] = []
  private unregister: (() => void) | null = null
  private reportTimer: number | null = null
  private lowSince: number | null = null
  private graceUntil = 0
  private lastFrame = 0
  private opts: FpsWatchOptions

  constructor(opts: FpsWatchOptions = {}) {
    this.opts = {
      minFps: 30,
      windowMs: 2500,
      sustainMs: 3000,
      graceMs: 6000,
      allowDowngrade: true,
      ...opts,
    }
  }

  start(register: (cb: () => void) => () => void): void {
    this.stop()
    this.samples = []
    this.lowSince = null
    this.graceUntil = performance.now() + (this.opts.graceMs ?? 6000)
    const onFrame = () => {
      const now = performance.now()
      if (now - this.lastFrame < FRAME_TARGET * 0.5) return
      this.lastFrame = now
      this.samples.push(now)
    }
    this.unregister = register(onFrame)
    this.reportTimer = window.setInterval(() => this.report(), 1000)
  }

  stop(): void {
    if (this.reportTimer !== null) {
      window.clearInterval(this.reportTimer)
      this.reportTimer = null
    }
    this.unregister?.()
    this.unregister = null
    this.samples = []
    this.lowSince = null
  }

  private report(): void {
    const now = performance.now()
    const cutoff = now - (this.opts.windowMs ?? 2500)
    while (this.samples.length > 0 && this.samples[0] < cutoff) this.samples.shift()

    if (this.samples.length < 2) {
      this.opts.onFps?.(null)
      this.lowSince = null
      return
    }

    const span = this.samples[this.samples.length - 1] - this.samples[0]
    const fps = span > 0 ? Math.round(((this.samples.length - 1) / span) * 1000) : 0
    this.opts.onFps?.(fps)

    if (!this.opts.allowDowngrade || now < this.graceUntil) {
      if (fps >= (this.opts.minFps ?? 30)) this.lowSince = null
      return
    }
    if (fps < (this.opts.minFps ?? 30)) {
      if (this.lowSince === null) {
        this.lowSince = now
        return
      }
      if (now - this.lowSince >= (this.opts.sustainMs ?? 3000)) {
        this.lowSince = null
        this.samples = []
        this.graceUntil = now + 5000
        this.opts.onDowngrade?.()
      }
    } else {
      this.lowSince = null
    }
  }
}
