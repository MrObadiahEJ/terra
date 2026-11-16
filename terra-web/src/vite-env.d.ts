/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** Solana JSON-RPC endpoint. Defaults to devnet; set for localnet in E2E. */
  readonly VITE_TERRA_RPC_URL?: string
  /** Off-chain API base (default `/api/v1`). */
  readonly VITE_API_BASE?: string
  /**
   * Development/E2E only: a JSON secret key used by Playwright as the connected
   * wallet. It is ignored in production builds.
   */
  readonly VITE_TERRA_E2E_SECRET_KEY?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
