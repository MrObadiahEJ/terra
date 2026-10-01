// Deterministic demo dataset for /network — there is no /validators API route
// yet (listed PLANNED on /status), so the page ships with a fixed, seeded
// network: same 9 validators + 4 tasks every load. Field codes mirror
// validator_profile.rs exactly (tier / availability / capability levels).
// Wallet strings are illustrative base58 labels, NOT real pubkeys.

import type { Presence, TaskRequirement } from './routingSim'

export interface DemoValidator {
  id: string
  wallet: string
  name: string
  country: string // ISO alpha-2
  countryName: string
  tier: number
  reputation: number // bps 0..10000
  availability: number
  capabilities: Record<number, number> // capability_code → level
  presence: Presence | null
  stakeSol: number
  tasksCompleted: number
  accuracyBps: number
}

// presence freshness: fresh for the session, or stale by design
const NOW = Math.floor(Date.now() / 1000)
const FRESH = NOW + 12 * 3600 // well inside PRESENCE_TTL_SECS (24h)
const STALE = NOW - 3600 // expired 1h ago — geo gate must reject

export const DEMO_VALIDATORS: DemoValidator[] = [
  {
    id: 'lagos-01',
    wallet: '7xKp9Vq2RmN5TfW8YaB3cD6eG1hJ4kL7mN0pQ9sT2uV',
    name: 'Lagos Survey Co-op',
    country: 'NG',
    countryName: 'Nigeria',
    tier: 3, // TRUSTED
    reputation: 8700,
    availability: 1, // AVAILABLE
    capabilities: { 0: 2, 3: 1, 2: 2 }, // GNSS VERIFIED, SURVEY OBSERVED, DOCUMENT VERIFIED
    presence: { latE7: 65_244_000, lonE7: 3_379_200, city: 'Lagos', expiresAt: FRESH },
    stakeSol: 12_500,
    tasksCompleted: 124,
    accuracyBps: 9870,
  },
  {
    id: 'abuja-02',
    wallet: '3fQa7Zx4Kp1Rn8Tt5Wc2Yb9Dd6Ee3Gg0Hh4Jj7Kk1Ll',
    name: 'Abuja Cadastre Lab',
    country: 'NG',
    countryName: 'Nigeria',
    tier: 2, // ESTABLISHED
    reputation: 7200,
    availability: 0, // ONLINE
    capabilities: { 0: 1, 3: 1, 2: 0 }, // GNSS OBSERVED, SURVEY OBSERVED, DOCUMENT DECLARED
    presence: { latE7: 90_579_000, lonE7: 74_951_000, city: 'Abuja', expiresAt: FRESH },
    stakeSol: 6_800,
    tasksCompleted: 58,
    accuracyBps: 9610,
  },
  {
    id: 'kano-03',
    wallet: '9mNb2Cx5Vp8Zq3Rr6Tt0Ww4Yy7Uu1Ii5Oo8Aa2Ss6Dd',
    name: 'Kano Field Unit',
    country: 'NG',
    countryName: 'Nigeria',
    tier: 2, // ESTABLISHED
    reputation: 7400,
    availability: 1, // AVAILABLE
    capabilities: { 3: 1, 0: 0 }, // SURVEY OBSERVED, GNSS DECLARED
    presence: { latE7: 120_022_000, lonE7: 85_920_000, city: 'Kano', expiresAt: STALE },
    stakeSol: 5_100,
    tasksCompleted: 41,
    accuracyBps: 9540,
  },
  {
    id: 'douala-04',
    wallet: '4dRc6Hn9Mq2Ww5Zz8Aa1Ss4Dd7Ff0Gg3Hh6Jj9Kk2L',
    name: 'Douala Port Survey',
    country: 'CM',
    countryName: 'Cameroon',
    tier: 1, // PROBATIONARY
    reputation: 6100,
    availability: 2, // BUSY
    capabilities: { 0: 1 }, // GNSS OBSERVED
    presence: { latE7: 40_511_000, lonE7: 97_679_000, city: 'Douala', expiresAt: FRESH },
    stakeSol: 2_400,
    tasksCompleted: 17,
    accuracyBps: 9310,
  },
  {
    id: 'accra-05',
    wallet: '6tYu1Jj4Kk7Ll0Pp3Rr6Tt9Ww2Yy5Uu8Ii1Oo4Aa7S',
    name: 'Accra Registry Trust',
    country: 'GH',
    countryName: 'Ghana',
    tier: 2, // ESTABLISHED
    reputation: 7800,
    availability: 1, // AVAILABLE
    capabilities: { 2: 2, 4: 1, 0: 1 }, // DOCUMENT VERIFIED, LEGAL OBSERVED, GNSS OBSERVED
    presence: { latE7: 56_037_000, lonE7: -1_870_000, city: 'Accra', expiresAt: FRESH },
    stakeSol: 7_900,
    tasksCompleted: 73,
    accuracyBps: 9720,
  },
  {
    id: 'madrid-06',
    wallet: '2wEr5Tt8Uu1Ii4Oo7Aa0Ss3Dd6Ff9Gg2Hh5Jj8Kk1L',
    name: 'Catastro Digital ES',
    country: 'ES',
    countryName: 'Spain',
    tier: 3, // TRUSTED
    reputation: 9100,
    availability: 0, // ONLINE
    capabilities: { 4: 2, 2: 3, 1: 1 }, // LEGAL VERIFIED, DOCUMENT TRUSTED, IMAGERY OBSERVED
    presence: { latE7: 404_168_000, lonE7: -37_038_000, city: 'Madrid', expiresAt: FRESH },
    stakeSol: 15_200,
    tasksCompleted: 212,
    accuracyBps: 9930,
  },
  {
    id: 'berlin-07',
    wallet: '8iOp3Aa6Ss9Dd2Ff5Gg8Hh1Jj4Kk7Ll0Pp3Rr6Tt9W',
    name: 'GeoForum Berlin',
    country: 'DE',
    countryName: 'Germany',
    tier: 3, // TRUSTED
    reputation: 9400,
    availability: 1, // AVAILABLE
    capabilities: { 1: 3, 0: 2 }, // IMAGERY TRUSTED, GNSS VERIFIED
    presence: { latE7: 525_200_000, lonE7: 134_050_000, city: 'Berlin', expiresAt: FRESH },
    stakeSol: 18_400,
    tasksCompleted: 301,
    accuracyBps: 9950,
  },
  {
    id: 'nairobi-08',
    wallet: '5aSd0Ff3Gg6Hh9Jj2Kk5Ll8Pp1Rr4Tt7Ww0Yy3Uu6I',
    name: 'Rift Valley Surveys',
    country: 'KE',
    countryName: 'Kenya',
    tier: 4, // HIGH_CAPABILITY
    reputation: 9600,
    availability: 1, // AVAILABLE
    capabilities: { 3: 3, 0: 2, 1: 2, 4: 1 }, // SURVEY TRUSTED, GNSS VERIFIED, IMAGERY VERIFIED, LEGAL OBSERVED
    presence: { latE7: -12_921_000, lonE7: 368_219_000, city: 'Nairobi', expiresAt: FRESH },
    stakeSol: 21_000,
    tasksCompleted: 268,
    accuracyBps: 9960,
  },
  {
    id: 'jakarta-09',
    wallet: '1zXc4Gg7Hh0Jj3Kk6Ll9Pp2Rr5Tt8Ww1Yy4Uu7Ii0O',
    name: 'Nusantara Geo ID',
    country: 'ID',
    countryName: 'Indonesia',
    tier: 2, // ESTABLISHED
    reputation: 6900,
    availability: 5, // SUSPENDED (admin action)
    capabilities: { 2: 2 }, // DOCUMENT VERIFIED
    presence: { latE7: -62_088_000, lonE7: 1_068_456_000, city: 'Jakarta', expiresAt: FRESH },
    stakeSol: 4_600,
    tasksCompleted: 89,
    accuracyBps: 9680,
  },
]

export const DEMO_TASKS: TaskRequirement[] = [
  {
    id: 'task-survey-lagos',
    label: 'Land survey — Lagos metro',
    description:
      'Field survey verification inside a 900 km radius of Lagos. Teaches availability (BUSY rejected), stale presence, and the SURVEY capability floor (DECLARED is too weak).',
    minTier: 2, // ESTABLISHED
    minReputation: 5000,
    capabilityCode: 3, // SURVEY
    jurisdiction: null, // any
    radiusM: 900_000,
    centerLatE7: 65_244_000,
    centerLonE7: 3_379_200,
    centerLabel: 'Lagos',
  },
  {
    id: 'task-docs-global',
    label: 'Document check — global',
    description:
      'No geo or jurisdiction scope: only availability + DOCUMENT capability (≥ OBSERVED) matter. Demonstrates the random admission gate with multiple competitors.',
    minTier: 0, // NEW
    minReputation: 0,
    capabilityCode: 2, // DOCUMENT
    jurisdiction: null,
    radiusM: 0,
    centerLatE7: 0,
    centerLonE7: 0,
    centerLabel: '—',
  },
  {
    id: 'task-legal-pt',
    label: 'Cross-border legal opinion — Portugal',
    description:
      'Jurisdiction scoped to PT: only an ES validator passes via the ACTIVE ES→PT CrossBorderBinding; a legal capability elsewhere fails the binding gate.',
    minTier: 2, // ESTABLISHED
    minReputation: 6000,
    capabilityCode: 4, // LEGAL
    jurisdiction: 'PT',
    radiusM: 0,
    centerLatE7: 0,
    centerLonE7: 0,
    centerLabel: '—',
  },
  {
    id: 'task-imagery',
    label: 'Imagery intelligence',
    description:
      'High bar: IMAGERY capability ≥ OBSERVED plus min_reputation 8000 bps — reputation filter and capability filter both bite.',
    minTier: 2, // ESTABLISHED
    minReputation: 8000,
    capabilityCode: 1, // IMAGERY
    jurisdiction: null,
    radiusM: 0,
    centerLatE7: 0,
    centerLonE7: 0,
    centerLabel: '—',
  },
]

/** Validated ACTIVE CrossBorderBinding (required → candidate country). */
export const DEMO_BINDINGS: { from: string; to: string; status: 'ACTIVE' }[] = [
  { from: 'ES', to: 'PT', status: 'ACTIVE' },
]

export function bindingActive(required: string | null, candidateCountry: string): boolean {
  if (required === null) return false
  return DEMO_BINDINGS.some((b) => b.from === candidateCountry && b.to === required && b.status === 'ACTIVE')
}

/** Demo blockhash/slot for route_seed — fixed, so every load reroutes identically. */
export const DEMO_BLOCKHASH = 'TerraDemoBlockhash9f4c2e7a1b8d5'
export const DEMO_INITIAL_SLOT = 12_345
