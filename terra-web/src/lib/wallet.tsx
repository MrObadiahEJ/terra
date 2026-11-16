/* eslint-disable react-refresh/only-export-components */
import { createContext, useContext, useEffect, useMemo } from 'react'
import { Connection, Keypair, PublicKey, Transaction } from '@solana/web3.js'
import {
  WalletProvider as AdapterWalletProvider,
  useWallet as useAdapterWallet,
} from '@solana/wallet-adapter-react'
import { WalletModalProvider } from '@solana/wallet-adapter-react-ui'
import type { AnchorWallet } from '@solana/wallet-adapter-react'
import { ed25519 } from '@noble/curves/ed25519'
import { TERRA_RPC_URL } from './constants'
import { setStorySigner } from '../story/api'

// The official @solana/wallet-adapter stack is used for all signing. The
// WalletProvider auto-discovers any Wallet Standard-compatible wallet installed
// in the browser (Phantom, Backpack, Solflare, ...), so no per-wallet packages
// (and no react-native toolchain) are required.

export const connection = new Connection(TERRA_RPC_URL, 'confirmed')

// ---------------------------------------------------------------------------
// Playwright localnet signer.
//
// The browser E2E has no extension wallet. When VITE_TERRA_E2E_SECRET_KEY is
// set (development builds only) this keypair becomes the connected wallet. It
// really signs and lands every transaction — it is an automated signer, not a
// simulation. Production builds tree-shake the branch (`import.meta.env.DEV`
// is statically false) and never read the variable.
// ---------------------------------------------------------------------------
const E2E_SECRET_KEY = import.meta.env.DEV
  ? import.meta.env.VITE_TERRA_E2E_SECRET_KEY
  : undefined

function parseE2EKeypair(): Keypair | null {
  if (!E2E_SECRET_KEY) return null
  try {
    return Keypair.fromSecretKey(Uint8Array.from(JSON.parse(E2E_SECRET_KEY)))
  } catch {
    console.error('VITE_TERRA_E2E_SECRET_KEY is not a valid JSON secret key; ignoring')
    return null
  }
}

const e2eKeypair = parseE2EKeypair()

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
  signMessage: ((message: Uint8Array) => Promise<Uint8Array>) | null
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
    signMessage,
  } = useAdapterWallet()

  const effectivePublicKey = e2eKeypair?.publicKey ?? publicKey
  const effectiveConnected = e2eKeypair ? true : connected

  const e2eSignTransaction = useMemo(
    () =>
      e2eKeypair
        ? async (tx: Transaction): Promise<Transaction> => {
            tx.partialSign(e2eKeypair)
            return tx
          }
        : null,
    [],
  )
  const e2eSignMessage = useMemo(
    () =>
      e2eKeypair
        ? async (message: Uint8Array): Promise<Uint8Array> =>
            ed25519.sign(message, e2eKeypair.secretKey.slice(0, 32))
        : null,
    [],
  )

  const activeSignTransaction = e2eSignTransaction ?? signTransaction
  const activeSignAllTransactions = e2eKeypair
    ? async (txs: Transaction[]): Promise<Transaction[]> => {
        for (const tx of txs) tx.partialSign(e2eKeypair)
        return txs
      }
    : signAllTransactions

  const anchorWallet = useMemo<AnchorWallet | undefined>(() => {
    if (!effectivePublicKey || !activeSignTransaction || !activeSignAllTransactions) {
      return undefined
    }
    return {
      publicKey: effectivePublicKey,
      signTransaction: activeSignTransaction,
      signAllTransactions: activeSignAllTransactions,
    }
  }, [effectivePublicKey, activeSignTransaction, activeSignAllTransactions])

  const send = useMemo(
    () =>
      async (tx: Transaction): Promise<string> => {
        if (!effectivePublicKey || !activeSignTransaction) {
          throw new Error('Wallet not connected')
        }
        return signAndSend(effectivePublicKey, activeSignTransaction, tx)
      },
    [effectivePublicKey, activeSignTransaction],
  )

  return {
    publicKey: effectivePublicKey,
    connected: effectiveConnected,
    connecting: e2eKeypair ? false : connecting,
    disconnected: !effectiveConnected,
    error: null,
    walletName: e2eKeypair ? 'Terra local E2E signer' : wallet?.adapter.name ?? null,
    anchorWallet,
    signMessage: e2eSignMessage ?? signMessage ?? null,
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
  // Story writes are signed with the wallet when connected, so they are
  // attributed to it; otherwise the story client falls back to an ephemeral key.
  useEffect(() => {
    setStorySigner(view.signMessage, view.publicKey ? view.publicKey.toBase58() : null)
    return () => setStorySigner(null, null)
  }, [view.signMessage, view.publicKey])
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
