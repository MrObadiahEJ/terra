import { describe, it, expect } from 'vitest'
import { parseGeoJSON } from './api'

describe('api — parseGeoJSON helper', () => {
  it('returns null for null/undefined input', () => {
    expect(parseGeoJSON(null)).toBeNull()
    expect(parseGeoJSON(undefined)).toBeNull()
  })

  it('parses valid GeoJSON JSON into the payload', () => {
    const fc = { type: 'FeatureCollection', features: [{ type: 'Feature' }] }
    expect(parseGeoJSON<unknown>(JSON.stringify(fc))).toEqual(fc)
  })

  it('returns null for malformed JSON', () => {
    expect(parseGeoJSON('{not-json')).toBeNull()
  })
})
