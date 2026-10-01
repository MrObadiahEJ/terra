// B4 activity feed store — session-local demo-engine events.
//
// The ActivityFeed component merges these (source: 'demo') with real off-chain
// API rows polled from GET /api/v1/activity (source: 'api'). Ids are run-scoped
// (`scenario:seed:runId:seq`) so StrictMode double-effects and replays never
// duplicate a run's entries, while every new run shows up as fresh activity.
import { create } from 'zustand'

export interface ActivityItem {
  id: string
  /** epoch ms */
  at: number
  kind: string
  summary: string
  source: 'demo' | 'api'
  link?: string
}

const MAX_ITEMS = 200

interface ActivityState {
  items: ActivityItem[]
  push: (item: ActivityItem) => void
}

export const useActivityStore = create<ActivityState>((set) => ({
  items: [],
  push: (item) =>
    set((s) =>
      s.items.some((x) => x.id === item.id) ? s : { items: [item, ...s.items].slice(0, MAX_ITEMS) },
    ),
}))
