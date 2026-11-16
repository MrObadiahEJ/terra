// Terra on-chain + off-chain constants shared across the frontend.

// Anchor program id (must match terra-core/programs/terra_registry declare_id).
import type { TranslationKey } from './locale'

export const TERRA_PROGRAM_ID = 'GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage'

// Separate on-chain program for identity / succession / guardianship.
export const TERRA_IDENTITY_PROGRAM_ID =
  'HyjpaHY9yQg1KneLZGjpLDQfP9WV4LUEbNZcb7C4Vo6p'

// Solana cluster. Terra targets Devnet for the Phase 1 MVP; the Sprint B
// Playwright loop overrides this with VITE_TERRA_RPC_URL on a local Surfnet.
export const TERRA_RPC_URL =
  import.meta.env.VITE_TERRA_RPC_URL?.trim() || 'https://api.devnet.solana.com'

function clusterName(url: string): string {
  if (/127\.0\.0\.1|localhost/.test(url)) return 'localnet'
  if (/devnet/.test(url)) return 'devnet'
  if (/testnet/.test(url)) return 'testnet'
  if (/mainnet/.test(url)) return 'mainnet-beta'
  return 'custom'
}

export const TERRA_CLUSTER = clusterName(TERRA_RPC_URL)
export const TERRA_IS_LOCALNET = TERRA_CLUSTER === 'localnet'

// Off-chain Axum API. Served through the Vite dev proxy at /api, so use a
// relative base here. Point this at the deployed host in production.
export const API_BASE = import.meta.env.VITE_API_BASE?.trim() || '/api/v1'

// Camera/journal target for the pilot region in Cameroon (Soa / Yaoundé area)
// used as the default focus of the 3D globe view.
export const DEFAULT_FOCUS = {
  longitude: 11.502,
  latitude: 3.848,
  height: 12000,
}

// Parcel statuses on-chain (must match lib.rs `parcel_status` module).
export const PARCEL_STATUS: Record<number, TranslationKey> = {
  0: 'statusPending',
  1: 'statusRegistered',
  2: 'statusForSale',
  3: 'statusTransferred',
}

// Right kinds on-chain (must match lib.rs `right_kind` module).
export const RIGHT_KINDS: Record<number, TranslationKey> = {
  0: 'rightOwnership',
  1: 'rightUsage',
  2: 'rightEasement',
  3: 'rightServitude',
  4: 'rightLien',
}

// Infrastructure flag bitmask (must match lib.rs `infra_flag` module).
export const INFRA_FLAGS: { bit: number; labelKey: TranslationKey }[] = [
  { bit: 0, labelKey: 'infraWastewater' },
  { bit: 1, labelKey: 'infraWater' },
  { bit: 2, labelKey: 'infraPower' },
  { bit: 3, labelKey: 'infraGas' },
  { bit: 4, labelKey: 'infraTelecom' },
  { bit: 5, labelKey: 'infraRoadAccess' },
  { bit: 6, labelKey: 'infraBuilding' },
]

export function infraLabelKeys(mask: number): TranslationKey[] {
  return INFRA_FLAGS.filter((f) => (mask & (1 << f.bit)) !== 0).map((f) => f.labelKey)
}
