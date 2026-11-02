import { describe, it, expect, beforeEach } from 'vitest'
import { useActivityStore, type ActivityItem } from './activityStore'

const item = (id: string, source: 'demo' | 'api' = 'demo'): ActivityItem => ({
  id,
  at: 0,
  kind: 'demo_event',
  summary: `summary-${id}`,
  source,
})

describe('activityStore — session-local demo activity feed', () => {
  beforeEach(() => {
    useActivityStore.setState({ items: [] })
  })

  it('starts empty', () => {
    expect(useActivityStore.getState().items).toEqual([])
  })

  it('push adds a new item', () => {
    useActivityStore.getState().push(item('a1'))
    const items = useActivityStore.getState().items
    expect(items).toHaveLength(1)
    expect(items[0].id).toBe('a1')
  })

  it('keeps newest-first ordering', () => {
    const s = useActivityStore.getState()
    s.push(item('a1'))
    s.push(item('a2'))
    s.push(item('a3'))
    const ids = useActivityStore.getState().items.map((x) => x.id)
    expect(ids).toEqual(['a3', 'a2', 'a1'])
  })

  it('dedups repeated pushes of the same run-scoped id', () => {
    const s = useActivityStore.getState()
    s.push(item('a1'))
    s.push(item('a1'))
    expect(useActivityStore.getState().items).toHaveLength(1)
  })

  it('caps the feed at 200 items, dropping the oldest', () => {
    const s = useActivityStore.getState()
    for (let i = 0; i < 205; i++) s.push(item(`a${i}`))
    const items = useActivityStore.getState().items
    expect(items).toHaveLength(200)
    expect(items[0].id).toBe('a204') // newest kept
    expect(items[199].id).toBe('a5') // oldest of the pushed window dropped (a0..a4)
  })

  it('preserves the source label across pushes', () => {
    const s = useActivityStore.getState()
    s.push(item('d1', 'demo'))
    s.push(item('b1', 'api'))
    expect(useActivityStore.getState().items.map((x) => x.source)).toEqual(['api', 'demo'])
  })
})
