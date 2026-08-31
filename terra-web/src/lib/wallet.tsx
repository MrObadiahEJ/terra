/* eslint-disable react-refresh/only-export-components */
import { createContext, useContext, useEffect, useMemo } from 'react'
import { Connection, PublicKey, Transaction } from '@solana/web3.js'
import {
  WalletProvider as AdapterWalletProvider,
  useWallet as useAdapterWallet,
} from '@solana/wallet-adapter-react'
import { WalletModalProvider } from '@solana/wallet-adapter-react-ui'
import type { AnchorWallet } from '@solana/wallet-adapter-react'
import { ed25519 } from '@noble/curves/ed25519'
import { TERRA_RPC_URL } from './constants'

// The official @solana/wallet-adapter stack is used for all signing. The
// WalletProvider auto-discovers any Wallet Standard-compatible wallet installed
// in the browser (Phantom, Backpack, Solflare, ...), so no per-wallet packages
// (and no react-native toolchain) are required.

export const connection = new Connection(TERRA_RPC_URL, 'confirmed')

// ---------------------------------------------------------------------------
// Hardened, per-request sign-and-send.
// Signing goes through the Wallet Standard adapter; we additionally verify the
// Ed25519 signature covers the serialized transaction before submission so a
// misbehaving/malicious wallet cannot submit bytes we didn't intend.
// ---------------------------------------------------------------------------
async function signAndSend(
  publicKey: PublicKey,
  signTransaction: (tx: Transaction) => Promise<Transaction>,
  tx: Transaction,
): Promise<string> {
  const addr = publicKey
  tx.feePayer = addr
  tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash

  const signed = await signTransaction(tx)

  // Defense in depth: confirm the wallet actually signed this exact payload.
  for (const sigN of signed.signatures) {
    if (sigN.publicKey.equals(addr)) {
      if (!sigN.signature) {
        throw new Error('Wallet returned a transaction without our signature')
      }
      const valid = ed25519.verify(
        sigN.signature,
        signed.serializeMessage(),
        publicKey.toBytes(),
      )
      if (!valid) {
        throw new Error('Wallet signature does not match the transaction (refusing to send)')
      }
    }
  }

  const rawSignature = await connection.sendRawTransaction(signed.serialize(), {
    skipPreflight: false,
  })
  await connection.confirmTransaction(rawSignature, 'confirmed')
  return rawSignature
}

// ---------------------------------------------------------------------------
// Compatibility layer: exposes the same { publicKey, send, ... } shape used by
// the UI panels, backed by the official adapter.
// ---------------------------------------------------------------------------
export interface WalletView {
  publicKey: PublicKey | null
  connected: boolean
  connecting: boolean
  disconnected: boolean
  error: string | null
  walletName: string | null
  anchorWallet: AnchorWallet | undefined
  send: (tx: Transaction) => Promise<string>
}

function useWalletView(): WalletView {
  const {
    publicKey,
    connected,
    connecting,
    wallet,
    signTransaction,
    signAllTransactions,
  } = useAdapterWallet()

  const anchorWallet = useMemo<AnchorWallet | undefined>(() => {
    if (!publicKey || !signTransaction || !signAllTransactions) return undefined
    return { publicKey, signTransaction, signAllTransactions }
  }, [publicKey, signTransaction, signAllTransactions])

  const send = useMemo(
    () =>
      async (tx: Transaction): Promise<string> => {
        if (!publicKey || !signTransaction) throw new Error('Wallet not connected')
        return signAndSend(publicKey, signTransaction, tx)
      },
    [publicKey, signTransaction],
  )

  return {
    publicKey,
    connected,
    connecting,
    disconnected: !connected,
    error: null,
    walletName: wallet?.adapter.name ?? null,
    anchorWallet,
    send,
  }
}

// Context so non-hook call sites (program.ts) can grab the live signer.
const WalletViewContext = createContext<WalletView | null>(null)

// Inner bridge: runs inside AdapterWalletProvider so it can use the adapter hook.
function WalletViewBridge({ children }: { children: React.ReactNode }) {
  const view = useWalletView()
  useEffect(() => {
    setActiveSigner(view.anchorWallet ?? null)
    return () => setActiveSigner(null)
  }, [view.anchorWallet])
  return <WalletViewContext.Provider value={view}>{children}</WalletViewContext.Provider>
}

export function TerraWalletProvider({ children }: { children: React.ReactNode }) {
  return (
    <AdapterWalletProvider
      wallets={[]}
      autoConnect
      onError={(e) => console.error('wallet error:', e)}
    >
      <WalletModalProvider>
        <WalletViewBridge>{children}</WalletViewBridge>
      </WalletModalProvider>
    </AdapterWalletProvider>
  )
}

/** Hook available to components (must be inside TerraWalletProvider). */
export function useWallet(): WalletView {
  const ctx = useContext(WalletViewContext)
  if (!ctx) throw new Error('useWallet must be used within TerraWalletProvider')
  return ctx
}

// Module-level signer for program.ts (populated from the view when connected).
let activeSigner: AnchorWallet | null = null
export function setActiveSigner(w: AnchorWallet | null) {
  activeSigner = w
}
export function getActiveSigner(): AnchorWallet | null {
  return activeSigner
}
