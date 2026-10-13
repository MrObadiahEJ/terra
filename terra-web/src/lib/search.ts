// Keyless geocoding: decimal-degree coordinates, a small bundled gazetteer,
// and the Photon public geocoder as an optional online fallback. No API keys
// anywhere — matches the keyless-terrain stance of the globe.

export interface SearchResult {
  id: string
  label: string
  longitude: number
  latitude: number
  /** Approximate fly-to height in metres. */
  height: number
  source: 'coords' | 'gazetteer' | 'photon'
}

interface GazetteerEntry {
  name: string
  country?: string
  longitude: number
  latitude: number
}

/** A deliberately tiny offline gazetteer — capital/major cities only. */
const GAZETTEER: GazetteerEntry[] = [
  { name: 'Yaoundé', country: 'Cameroon', longitude: 11.5021, latitude: 3.848 },
  { name: 'Douala', country: 'Cameroon', longitude: 9.7043, latitude: 4.0511 },
  { name: 'Bern', country: 'Switzerland', longitude: 7.4474, latitude: 46.9481 },
  { name: 'Zurich', country: 'Switzerland', longitude: 8.5417, latitude: 47.3769 },
  { name: 'Vaduz', country: 'Liechtenstein', longitude: 9.5215, latitude: 47.141 },
  { name: 'Berlin', country: 'Germany', longitude: 13.405, latitude: 52.52 },
  { name: 'Paris', country: 'France', longitude: 2.3522, latitude: 48.8566 },
  { name: 'London', country: 'United Kingdom', longitude: -0.1276, latitude: 51.5072 },
  { name: 'Madrid', country: 'Spain', longitude: -3.7038, latitude: 40.4168 },
  { name: 'Rome', country: 'Italy', longitude: 12.4964, latitude: 41.9028 },
  { name: 'Amsterdam', country: 'Netherlands', longitude: 4.9041, latitude: 52.3676 },
  { name: 'Brussels', country: 'Belgium', longitude: 4.3517, latitude: 50.8503 },
  { name: 'Vienna', country: 'Austria', longitude: 16.3738, latitude: 48.2082 },
  { name: 'Prague', country: 'Czechia', longitude: 14.4378, latitude: 50.0755 },
  { name: 'Warsaw', country: 'Poland', longitude: 21.0122, latitude: 52.2297 },
  { name: 'Stockholm', country: 'Sweden', longitude: 18.0686, latitude: 59.3293 },
  { name: 'Oslo', country: 'Norway', longitude: 10.7522, latitude: 59.9139 },
  { name: 'Copenhagen', country: 'Denmark', longitude: 12.5683, latitude: 55.6761 },
  { name: 'Helsinki', country: 'Finland', longitude: 24.9384, latitude: 60.1699 },
  { name: 'Athens', country: 'Greece', longitude: 23.7275, latitude: 37.9838 },
  { name: 'Lisbon', country: 'Portugal', longitude: -9.1393, latitude: 38.7223 },
  { name: 'Dublin', country: 'Ireland', longitude: -6.2603, latitude: 53.3498 },
  { name: 'New York', country: 'United States', longitude: -74.006, latitude: 40.7128 },
  { name: 'Los Angeles', country: 'United States', longitude: -118.2437, latitude: 34.0522 },
  { name: 'Chicago', country: 'United States', longitude: -87.6298, latitude: 41.8781 },
  { name: 'Toronto', country: 'Canada', longitude: -79.3832, latitude: 43.6532 },
  { name: 'Mexico City', country: 'Mexico', longitude: -99.1332, latitude: 19.4326 },
  { name: 'São Paulo', country: 'Brazil', longitude: -46.6333, latitude: -23.5505 },
  { name: 'Buenos Aires', country: 'Argentina', longitude: -58.3816, latitude: -34.6037 },
  { name: 'Cairo', country: 'Egypt', longitude: 31.2357, latitude: 30.0444 },
  { name: 'Nairobi', country: 'Kenya', longitude: 36.8219, latitude: -1.2921 },
  { name: 'Lagos', country: 'Nigeria', longitude: 3.3792, latitude: 6.5244 },
  { name: 'Johannesburg', country: 'South Africa', longitude: 28.0473, latitude: -26.2041 },
  { name: 'Dubai', country: 'United Arab Emirates', longitude: 55.2708, latitude: 25.2048 },
  { name: 'Mumbai', country: 'India', longitude: 72.8777, latitude: 19.076 },
  { name: 'Delhi', country: 'India', longitude: 77.209, latitude: 28.6139 },
  { name: 'Singapore', country: 'Singapore', longitude: 103.8198, latitude: 1.3521 },
  { name: 'Tokyo', country: 'Japan', longitude: 139.6917, latitude: 35.6895 },
  { name: 'Seoul', country: 'South Korea', longitude: 126.978, latitude: 37.5665 },
  { name: 'Beijing', country: 'China', longitude: 116.4074, latitude: 39.9042 },
  { name: 'Shanghai', country: 'China', longitude: 121.4737, latitude: 31.2304 },
  { name: 'Sydney', country: 'Australia', longitude: 151.2093, latitude: -33.8688 },
  { name: 'Melbourne', country: 'Australia', longitude: 144.9631, latitude: -37.8136 },
  { name: 'Auckland', country: 'New Zealand', longitude: 174.7633, latitude: -36.8485 },
]

/**
 * Parse a decimal-degree coordinate pair ("47.15 9.55", "47.15N 9.55E",
 * "9.55W 47.15S"). Without hemisphere suffixes the first token is treated
 * as latitude when it falls within ±90, otherwise as longitude.
 */
export function parseCoords(query: string): { longitude: number; latitude: number } | null {
  const cleaned = query.trim().replace(/,/g, ' ')
  const match = cleaned.match(
    /^(-?\d{1,3}(?:\.\d+)?)\s*([ns])?\s+(-?\d{1,3}(?:\.\d+)?)\s*([ew])?$/i,
  )
  if (!match) return null
  const rawA = parseFloat(match[1])
  const sufA = match[2]?.toLowerCase()
  const rawB = parseFloat(match[3])
  const sufB = match[4]?.toLowerCase()

  let latitude: number
  let longitude: number
  if (sufA || sufB) {
    if (sufA) {
      latitude = sufA === 's' ? -Math.abs(rawA) : Math.abs(rawA)
      longitude = rawB
    } else {
      // B carries E/W, so A must be latitude.
      latitude = rawA
      longitude = sufB === 'w' ? -Math.abs(rawB) : Math.abs(rawB)
    }
  } else if (Math.abs(rawA) <= 90) {
    latitude = rawA
    longitude = rawB
  } else {
    longitude = rawA
    latitude = rawB
  }
  if (Math.abs(latitude) > 90 || Math.abs(longitude) > 180) return null
  return { longitude, latitude }
}

/** Synchronous local search: coordinates first, then the gazetteer. */
export function searchLocal(query: string, limit = 6): SearchResult[] {
  const q = query.trim()
  if (!q) return []
  const results: SearchResult[] = []

  const coords = parseCoords(q)
  if (coords) {
    results.push({
      id: `coords:${coords.latitude},${coords.longitude}`,
      label: `${coords.latitude.toFixed(5)}, ${coords.longitude.toFixed(5)}`,
      longitude: coords.longitude,
      latitude: coords.latitude,
      height: 25_000,
      source: 'coords',
    })
  }

  const needle = q.toLowerCase()
  for (const entry of GAZETTEER) {
    const hay = `${entry.name} ${entry.country ?? ''}`.toLowerCase()
    if (hay.includes(needle)) {
      results.push({
        id: `gaz:${entry.name}`,
        label: entry.country ? `${entry.name}, ${entry.country}` : entry.name,
        longitude: entry.longitude,
        latitude: entry.latitude,
        height: 400_000,
        source: 'gazetteer',
      })
    }
    if (results.length >= limit) break
  }
  return results.slice(0, limit)
}

/** Online fallback via the Photon public geocoder (no API key). */
export async function searchPhoton(query: string, limit = 5): Promise<SearchResult[]> {
  const url = `https://photon.komoot.io/api/?q=${encodeURIComponent(query)}&limit=${limit}`
  const res = await fetch(url)
  if (!res.ok) throw new Error(`photon ${res.status}`)
  const data = (await res.json()) as {
    features: Array<{
      geometry?: { coordinates?: [number, number] }
      properties?: Record<string, string | undefined>
    }>
  }
  const results: SearchResult[] = []
  for (const [index, feature] of (data.features ?? []).entries()) {
    const [longitude, latitude] = feature.geometry?.coordinates ?? []
    if (longitude === undefined || latitude === undefined) continue
    const p = feature.properties ?? {}
    const parts = [p.name, p.city, p.state, p.country].filter(Boolean)
    results.push({
      id: `photon:${index}:${longitude},${latitude}`,
      label: parts.length > 0 ? parts.join(', ') : `${latitude}, ${longitude}`,
      longitude,
      latitude,
      height: 300_000,
      source: 'photon',
    })
  }
  return results
}

/** Local-first search; Photon only when nothing local matched. */
export async function searchPlaces(query: string): Promise<SearchResult[]> {
  const local = searchLocal(query)
  if (local.length > 0) return local
  try {
    return await searchPhoton(query)
  } catch {
    return []
  }
}
