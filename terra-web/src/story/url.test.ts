import { describe, it, expect } from 'vitest'
import { parseStoryLink, buildStoryLink } from './url'

describe('parseStoryLink', () => {
  it('parses step and time', () => {
    expect(parseStoryLink('?step=3').step).toBe(3)
    expect(parseStoryLink('?time=18.5').time).toBe(18.5)
    const both = parseStoryLink('?step=3&time=18.5')
    expect(both.step).toBe(3)
    expect(both.time).toBe(18.5)
  })

  it('ignores absent, negative and non-numeric values', () => {
    expect(parseStoryLink('').step).toBeUndefined()
    expect(parseStoryLink('?time=-4').time).toBeUndefined()
    expect(parseStoryLink('?step=abc').step).toBeUndefined()
  })
})

describe('buildStoryLink', () => {
  it('omits empty state', () => {
    expect(buildStoryLink('demo-journey')).toBe('/story/demo-journey')
  })

  it('round-trips time and step', () => {
    const t = buildStoryLink('demo-journey', { time: 18.5 })
    expect(t).toBe('/story/demo-journey?time=18.5')
    expect(parseStoryLink(t.split('?')[1]).time).toBe(18.5)

    const s = buildStoryLink('demo-journey', { step: 2 })
    expect(s).toBe('/story/demo-journey?step=2')
    expect(parseStoryLink(s.split('?')[1]).step).toBe(2)
  })

  it('gives time precedence over step', () => {
    const l = buildStoryLink('demo-journey', { step: 2, time: 9 })
    expect(parseStoryLink(l.split('?')[1]).time).toBe(9)
  })

  it('encodes the id', () => {
    expect(buildStoryLink('a/b')).toBe('/story/a%2Fb')
  })
})
