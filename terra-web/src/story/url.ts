// URL codec for story deep links (RFC-014 §4).
//
//   /story/:id              → play from the beginning
//   /story/:id?step=3       → jump to stop 3
//   /story/:id?time=18.5    → seek to 18.5s (wins over step when both given)
//
// Links are reproducible scene specifications: the same URL resolves the same
// story document and the same logical timeline position on every device.

export interface StoryLinkState {
  step?: number
  time?: number
}

export function parseStoryLink(search: string): StoryLinkState {
  const params = new URLSearchParams(search)
  const out: StoryLinkState = {}
  const stepRaw = params.get('step')
  if (stepRaw !== null) {
    const n = Number.parseInt(stepRaw, 10)
    if (Number.isFinite(n) && n >= 0) out.step = n
  }
  const timeRaw = params.get('time')
  if (timeRaw !== null) {
    const n = Number.parseFloat(timeRaw)
    if (Number.isFinite(n) && n >= 0) out.time = n
  }
  return out
}

/** Serialize link state; omits empty fields. `time` takes precedence over `step`. */
export function buildStoryLink(id: string, state: StoryLinkState = {}): string {
  const params = new URLSearchParams()
  if (state.time !== undefined && state.time >= 0) {
    params.set('time', state.time.toFixed(1))
  } else if (state.step !== undefined && state.step >= 0) {
    params.set('step', String(state.step))
  }
  const qs = params.toString()
  return qs ? `/story/${encodeURIComponent(id)}?${qs}` : `/story/${encodeURIComponent(id)}`
}
