import { describe, it, expect } from 'vitest'
import {
  STATUS_LEVELS,
  STATUS_META,
  STATUS_MODULES,
  STATUS_TRACKS,
  TRACK_META,
  statusCounts,
} from './protocolStatus'

describe('protocolStatus — status bookkeeping', () => {
  it('exposes the four status levels and three tracks', () => {
    expect(STATUS_LEVELS).toEqual(['implemented', 'partial', 'experimental', 'planned'])
    expect(STATUS_TRACKS).toEqual(['protocol', 'infra', 'frontend'])
  })

  it('meta tables cover every status and track', () => {
    for (const level of STATUS_LEVELS) {
      expect(STATUS_META[level]).toBeDefined()
      expect(typeof STATUS_META[level].labelKey).toBe('string')
      expect(typeof STATUS_META[level].badge).toBe('string')
    }
    for (const track of STATUS_TRACKS) {
      expect(TRACK_META[track]).toBeDefined()
      expect(typeof TRACK_META[track].labelKey).toBe('string')
      expect(typeof TRACK_META[track].hintKey).toBe('string')
    }
  })

  it('statusCounts aggregates the default module table without loss', () => {
    const counts = statusCounts()
    const sum = Object.values(counts).reduce((a, b) => a + b, 0)
    expect(sum).toBe(STATUS_MODULES.length)
    for (const level of STATUS_LEVELS) {
      expect(counts[level]).toBe(STATUS_MODULES.filter((m) => m.status === level).length)
    }
  })

  it('statusCounts tallies caller-supplied modules', () => {
    const modules = [
      { id: 'a', name: 'A', track: 'protocol', status: 'implemented', evidence: '', verified: [], gaps: [] },
      { id: 'b', name: 'B', track: 'protocol', status: 'partial', evidence: '', verified: [], gaps: [] },
      { id: 'c', name: 'C', track: 'protocol', status: 'planned', evidence: '', verified: [], gaps: [] },
    ] as unknown as typeof STATUS_MODULES
    expect(statusCounts(modules)).toEqual({
      implemented: 1,
      partial: 1,
      experimental: 0,
      planned: 1,
    })
  })
})
