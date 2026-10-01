// TypeScript mirror of terra-core/programs/terra_registry/src/routing.rs.
// The /network inspector runs the SAME predicates as `route_task`, in the
// same order (Design Rule 3): availability → tier → reputation → capability →
// jurisdiction → geo, then deterministic winner pick + random admission gate.
//
// On-chain `draw_bps` / `select_winner` / `route_seed` hash with sha256
// (hashv); the demo mirrors their *semantics* (draw ∈ [0,10000) bps,
// order-independent winner over sorted candidates) with a sync FNV-1a so
// results stay deterministic offline. Flagged as DEMO in the UI.

export const EARTH_RADIUS_M = 6_371_000
export const MAX_ROUTE_CANDIDATES = 8
export const MAX_COMPETITOR_COUNT = 64
export const CAPABILITY_ANY = 255
export const MAX_REPUTATION = 10_000
export const PRESENCE_TTL_SECS = 24 * 3600

// validator_profile.rs const modules (exact codes)
export const TIERS = ['NEW', 'PROBATIONARY', 'ESTABLISHED', 'TRUSTED', 'HIGH_CAPABILITY'] as const
export const AVAILABILITIES = ['ONLINE', 'AVAILABLE', 'BUSY', 'OFFLINE', 'TEMP_UNAVAILABLE', 'SUSPENDED'] as const
export const CAPABILITY_LEVELS = ['DECLARED', 'OBSERVED', 'VERIFIED', 'TRUSTED'] as const
export const CAPABILITY_CODES = ['GNSS', 'IMAGERY', 'DOCUMENT', 'SURVEY', 'LEGAL', 'PHYSICAL', 'REMOTE'] as const

export interface Presence {
  latE7: number
  lonE7: number
  city: string
  expiresAt: number // unix seconds
}

export interface TaskRequirement {
  id: string
  label: string
  description: string
  minTier: number
  minReputation: number
  capabilityCode: number // 255 = CAPABILITY_ANY
  jurisdiction: string | null // ISO alpha-2, null = [0,0] any
  radiusM: number // 0 = no geo constraint
  centerLatE7: number
  centerLonE7: number
  centerLabel: string
}

export interface GateResult {
  pass: boolean
  detail: string
}

export interface Eligibility {
  eligible: boolean
  availability: GateResult
  tier: GateResult
  reputation: GateResult
  capability: GateResult
  jurisdiction: GateResult
  geo: GateResult
}

/** Haversine great-circle distance in meters (e7 fixed-point), routing.rs:40. */
export function distanceM(lat1e7: number, lon1e7: number, lat2e7: number, lon2e7: number): number {
  const toRad = (deg: number) => (deg * Math.PI) / 180
  const lat1 = toRad(lat1e7 / 1e7)
  const lat2 = toRad(lat2e7 / 1e7)
  const dlat = toRad((lat2e7 - lat1e7) / 1e7)
  const dlon = toRad((lon2e7 - lon1e7) / 1e7)
  const a =
    Math.sin(dlat / 2) ** 2 + Math.cos(lat1) * Math.cos(lat2) * Math.sin(dlon / 2) ** 2
  const c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a))
  return Math.round(EARTH_RADIUS_M * c)
}

/** routing.rs:52 — AVAILABLE or ONLINE only. */
export function passesAvailability(status: number): boolean {
  return status === 1 || status === 0
}

/** routing.rs:60 — profile_tier >= min_tier (min 0 always passes). */
export function passesTier(profileTier: number, minTier: number): boolean {
  return profileTier >= minTier
}

/** routing.rs:65 — score >= min_reputation (min 0 always passes). */
export function passesReputation(score: number, minReputation: number): boolean {
  return score >= minReputation
}

/** routing.rs:71 — ANY passes; else level must be OBSERVED..=TRUSTED (declared-only too weak). */
export function passesCapability(requiredCode: number, candidateLevel: number | null): boolean {
  if (requiredCode === CAPABILITY_ANY) return true
  if (candidateLevel === null) return false
  return candidateLevel >= 1 && candidateLevel <= 3
}

/** routing.rs:83 — radius 0 passes; else fresh presence within radius. */
export function passesGeo(
  radiusM: number,
  centerLatE7: number,
  centerLonE7: number,
  presence: Presence | null,
  now: number,
): boolean {
  if (radiusM === 0) return true
  if (!presence) return false
  if (presence.expiresAt < now) return false // presence_is_stale
  return distanceM(presence.latE7, presence.lonE7, centerLatE7, centerLonE7) <= radiusM
}

/** routing.rs:108 — [0,0]=any; undeclared never satisfies; else equal or ACTIVE binding. */
export function passesJurisdiction(
  required: string | null,
  profileJurisdiction: string,
  bindingOk: boolean,
): boolean {
  if (required === null) return true
  if (profileJurisdiction === '') return false
  if (profileJurisdiction === required) return true
  return bindingOk
}

/** Composite eligibility — same argument order/gates as routing.rs:131 is_eligible. */
export function evaluateEligibility(
  req: TaskRequirement,
  v: {
    tier: number
    reputation: number
    availability: number
    jurisdiction: string
    capabilities: Record<number, number>
    presence: Presence | null
  },
  bindingOk: boolean,
  now: number,
): Eligibility {
  const availabilityPass = passesAvailability(v.availability)
  const tierPass = passesTier(v.tier, req.minTier)
  const repPass = passesReputation(v.reputation, req.minReputation)
  const level =
    req.capabilityCode === CAPABILITY_ANY ? null : (v.capabilities[req.capabilityCode] ?? null)
  const capPass = passesCapability(req.capabilityCode, level)
  const jurPass = passesJurisdiction(req.jurisdiction, v.jurisdiction, bindingOk)

  let geo: GateResult
  if (req.radiusM === 0) {
    geo = { pass: true, detail: 'radius_m = 0 — no geographic constraint' }
  } else if (!v.presence) {
    geo = { pass: false, detail: 'no presence record — geo-scoped task requires fresh presence' }
  } else if (v.presence.expiresAt < now) {
    const ago = Math.round((now - v.presence.expiresAt) / 60)
    geo = { pass: false, detail: `presence stale — expired ${ago} min ago (PRESENCE_TTL_SECS = 24h)` }
  } else {
    const d = distanceM(v.presence.latE7, v.presence.lonE7, req.centerLatE7, req.centerLonE7)
    geo = {
      pass: d <= req.radiusM,
      detail: `${(d / 1000).toFixed(1)} km from ${req.centerLabel} ≤ radius ${(req.radiusM / 1000).toFixed(0)} km`,
    }
  }

  const availability: GateResult = {
    pass: availabilityPass,
    detail: `status = ${AVAILABILITIES[v.availability]} — ONLINE or AVAILABLE required`,
  }
  const tier: GateResult = {
    pass: tierPass,
    detail: `${TIERS[v.tier]} (${v.tier}) ≥ min_tier ${TIERS[req.minTier]} (${req.minTier})`,
  }
  const reputation: GateResult = {
    pass: repPass,
    detail: `score ${v.reputation} ≥ min_reputation ${req.minReputation} bps (MAX ${MAX_REPUTATION})`,
  }
  const capability: GateResult = {
    pass: capPass,
    detail:
      req.capabilityCode === CAPABILITY_ANY
        ? 'CAPABILITY_ANY (255) — always passes'
        : level === null
          ? `no ${CAPABILITY_CODES[req.capabilityCode]} capability declared`
          : `${CAPABILITY_CODES[req.capabilityCode]} level ${CAPABILITY_LEVELS[level]} (${level}) — OBSERVED (1) required`,
  }
  const jurisdiction: GateResult = {
    pass: jurPass,
    detail:
      req.jurisdiction === null
        ? 'jurisdiction [0,0] — any'
        : v.jurisdiction === req.jurisdiction
          ? `${v.jurisdiction} = required ${req.jurisdiction} (same country)`
          : v.jurisdiction === ''
            ? 'profile jurisdiction undeclared [0,0] — never satisfies a scoped task'
            : bindingOk
              ? `${v.jurisdiction} → ${req.jurisdiction}: ACTIVE CrossBorderBinding ✓`
              : `${v.jurisdiction} → ${req.jurisdiction}: no valid ACTIVE binding`,
  }

  const eligible = availabilityPass && tierPass && repPass && capPass && jurPass && geo.pass
  return { eligible, availability, tier, reputation, capability, jurisdiction, geo }
}

// ---------------------------------------------------------------------------
// Deterministic demo draws (mirror route_seed / draw_bps / select_winner)
// ---------------------------------------------------------------------------

function fnv1a(str: string): number {
  let h = 0x811c9dc5
  for (let i = 0; i < str.length; i++) {
    h ^= str.charCodeAt(i)
    h = Math.imul(h, 0x01000193) >>> 0
  }
  return h >>> 0
}

/** Demo route seed: mirrors route_seed(task_id, blockhash, slot) as a string. */
export function routeSeed(taskId: string, blockhash: string, slot: number): string {
  return `${taskId}|${blockhash}|${slot}`
}

/** Mirrors draw_bps — ∈ [0, 10000). */
export function drawBps(seed: string, validatorId: string): number {
  return fnv1a(`${seed} ${validatorId} terra_draw`) % 10_000
}

/** Mirrors select_winner — sorted candidates, hash(seed+"terra_route") % n. */
export function selectWinner(seed: string, candidates: string[]): string | null {
  if (candidates.length === 0) return null
  const sorted = [...candidates].sort()
  const idx = fnv1a(`${seed} terra_route`) % sorted.length
  return sorted[idx]
}

/** routing.rs:193 — draw < 10000/competitor_count; 1 competitor always passes. */
export function passesRandomGate(draw: number, competitorCount: number): boolean {
  if (competitorCount === 0 || competitorCount > MAX_COMPETITOR_COUNT) return false
  if (competitorCount === 1) return true
  return draw < Math.floor(10_000 / competitorCount)
}

export function admitBps(competitorCount: number): number {
  if (competitorCount <= 1) return 10_000
  if (competitorCount > MAX_COMPETITOR_COUNT) return 0
  return Math.floor(10_000 / competitorCount)
}
