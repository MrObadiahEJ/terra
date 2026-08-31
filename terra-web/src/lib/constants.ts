// Terra on-chain + off-chain constants shared across the frontend.

// Anchor program id (must match terra-core/programs/terra_registry declare_id).
export const TERRA_PROGRAM_ID = 'GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage'

// Solana cluster. Terra targets Devnet for the Phase 1 MVP.
export const TERRA_CLUSTER = 'devnet'
export const TERRA_RPC_URL =
  'https://api.devnet.solana.com'

// Off-chain Axum API. Served through the Vite dev proxy at /api, so use a
// relative base here. Point this at the deployed host in production.
export const API_BASE = '/api/v1'

// Camera/journal target for the pilot region in Cameroon (Soa / Yaoundé area)
// used as the default focus of the 3D globe view.
export const DEFAULT_FOCUS = {
  longitude: 11.502,
  latitude: 3.848,
  height: 12000,
}

// Parcel statuses on-chain (must match lib.rs `parcel_status` module).
export const PARCEL_STATUS: Record<number, string> = {
  0: 'Pending',
  1: 'Registered',
  2: 'For Sale',
  3: 'Transferred',
}

// Right kinds on-chain (must match lib.rs `right_kind` module).
export const RIGHT_KINDS: Record<number, string> = {
  0: 'Ownership',
  1: 'Usage',
  2: 'Easement',
  3: 'Servitude',
  4: 'Lien',
}

// Infrastructure flag bitmask (must match lib.rs `infra_flag` module).
export const INFRA_FLAGS: { bit: number; label: string }[] = [
  { bit: 0, label: 'Wastewater' },
  { bit: 1, label: 'Water' },
  { bit: 2, label: 'Power' },
  { bit: 3, label: 'Gas' },
  { bit: 4, label: 'Telecom' },
  { bit: 5, label: 'Road Access' },
  { bit: 6, label: 'Building' },
]

export function infraLabels(mask: number): string[] {
  return INFRA_FLAGS.filter((f) => (mask & (1 << f.bit)) !== 0).map((f) => f.label)
}
