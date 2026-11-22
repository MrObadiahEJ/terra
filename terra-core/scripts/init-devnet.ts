#!/usr/bin/env npx ts-node
/**
 * scripts/init-devnet.ts — Post-deploy protocol initialization for devnet.
 *
 * Ports the on-chain initialization sequence from
 * tests/localnet_e2e.ts (Sprint A bootstrap) so that after `deploy.sh devnet`
 * the protocol is actually usable — not just two deployed .so files with
 * zero accounts.
 *
 * Devnet-specific: uses durable, persisted keypairs (not ephemeral random ones)
 * and funds via faucet rotation instead of `requestAirdrop(100 SOL)`, which
 * devnet rate-limits and would exhaust the faucet budget.
 *
 * Initializes, in order:
 *   1. validator_registry (bootstrap admin = DEPLOY_KEYPAIR)
 *   2. vrf_authority (pins the route_task oracle key = ORACLE_KEYPAIR)
 *   3. N validators: addValidatorToRegistry + initValidatorProfile +
 *      setValidatorAvailability + initializeValidatorReputation
 *   4. fee_policy (setFeePolicy)
 *   5. coverage_incentive (setCoverageIncentive)
 *
 * Reads program IDs from Anchor.toml / environment, connects to the target
 * RPC, funds via faucet rotation, then submits every instruction with real
 * signers. Idempotent-ish: re-running will error on already-existing PDAs
 * (expected — use a fresh devnet or accept the errors).
 *
 * Usage:
 *   DEPLOY_KEYPAIR=~/.config/solana/id.json \
 *   ORACLE_KEYPAIR=.devnet/oracle-keypair.json \
 *   VALIDATOR_KEYPAIRS=".devnet/v1.json,.devnet/v2.json" \
 *   TERRA_RPC_URL=https://api.devnet.solana.com \
 *   npx ts-node scripts/init-devnet.ts
 *
 * Env:
 *   DEPLOY_KEYPAIR      Path to durable deploy keypair (admin/registry owner)
 *                       (default: ~/.config/solana/id.json)
 *   ORACLE_KEYPAIR      Path to persisted oracle keypair. Its pubkey is the
 *                       value to set as VRF_ORACLE_PUBKEY in the API env
 *                       (closes DEEP_AUDIT "PoP oracle not pinned" gap).
 *                       (default: .devnet/oracle-keypair.json)
 *   VALIDATOR_KEYPAIRS  Comma-separated list of persisted validator keypair
 *                       paths (default: empty — no validators bootstrapped)
 *   TERRA_RPC_URL       JSON-RPC endpoint (default https://api.devnet.solana.com)
 *   TERRA_PROGRAM_ID    terra_registry program ID (default from Anchor.toml)
 *   TERRA_IDENTITY_ID   terra_identity program ID (default from Anchor.toml)
 *   FAUCET_URLS         Comma-separated devnet faucet URLs for rotation
 *                       (default: https://api.devnet.solana.com)
 *   MAX_FAUCET_RETRIES  Max faucet attempts per address (default: 10)
 */

import * as anchor from "@coral-xyz/anchor";
import { Program, Idl } from "@coral-xyz/anchor";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import {
  Keypair,
  PublicKey,
  Connection,
  SystemProgram,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import * as fs from "fs";
import * as path from "path";
import { assert } from "chai";

const DEPLOY_KEYPAIR_PATH = process.env.DEPLOY_KEYPAIR ?? "$HOME/.config/solana/id.json";
const ORACLE_KEYPAIR_PATH = process.env.ORACLE_KEYPAIR ?? ".devnet/oracle-keypair.json";
const VALIDATOR_KEYPAIR_PATHS = (process.env.VALIDATOR_KEYPAIRS ?? "")
  .split(",")
  .map((s) => s.trim())
  .filter(Boolean);
const RPC_URL = process.env.TERRA_RPC_URL ?? "https://api.devnet.solana.com";
const FAUCET_URLS = (process.env.FAUCET_URLS ?? "https://api.devnet.solana.com")
  .split(",")
  .map((s) => s.trim())
  .filter(Boolean);
const MAX_FAUCET_RETRIES = Number(process.env.MAX_FAUCET_RETRIES ?? "10");
const LAMPORTS_PER_SOLn = BigInt(LAMPORTS_PER_SOL);
const IDL_PATH = path.join(__dirname, "..", "target", "idl", "terra_registry.json");

const PARCEL = { REGISTERED: 1 } as const;
const RIGHTS = { USAGE: 1 } as const;
const AVAILABILITY_ONLINE = 0;
const CAPABILITY_ANY = 255;

function loadKeypair(p: string): Keypair {
  const secret = JSON.parse(fs.readFileSync(p, "utf-8"));
  return Keypair.fromSecretKey(new Uint8Array(secret));
}

function u8(n: number): Buffer {
  return Buffer.from([n]);
}

function u32le(n: number): Buffer {
  const b = Buffer.alloc(4);
  b.writeUInt32LE(n, 0);
  return b;
}

function pda(seeds: (Buffer | Uint8Array | string)[]): PublicKey {
  return PublicKey.findProgramAddressSync(
    seeds.map((s) => (typeof s === "string" ? Buffer.from(s) : Buffer.from(s))),
    PROGRAM_ID
  )[0];
}

const PROGRAM_ID = new PublicKey(
  process.env.TERRA_PROGRAM_ID ?? "GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage"
);
const IDENTITY_ID = new PublicKey(
  process.env.TERRA_IDENTITY_ID ?? "HyjpaHY9yQg1KneLZGjpLDQfP9WV4LUEbNZcb7C4Vo6p"
);

const registryPda = pda(["validator_registry"]);
const feePolicyPda = pda(["fee_policy"]);
const coveragePda = pda(["coverage_incentive"]);
const treasuryPda = pda(["task_treasury"]);
const vrfAuthorityPda = pda(["vrf_authority"]);

const profilePda = (v: PublicKey) => pda(["validator_profile", v.toBuffer()]);
const availabilityPda = (v: PublicKey) => pda(["validator_availability", v.toBuffer()]);
const reputationPda = (v: PublicKey) => pda(["validator_reputation", v.toBuffer()]);
const endorsementPda = (v: PublicKey) =>
  pda(["validator_endorsement", registryPda.toBuffer(), v.toBuffer()]);

async function main() {
  const connection = new Connection(RPC_URL, "confirmed");

  // --- Load durable keys (no ephemeral generation) -------------------------
  const payer = loadKeypair(DEPLOY_KEYPAIR_PATH);
  const wallet = new NodeWallet(payer);
  const provider = new anchor.AnchorProvider(connection, wallet, { commitment: "confirmed" });
  anchor.setProvider(provider);

  const idl = JSON.parse(fs.readFileSync(IDL_PATH, "utf-8")) as Idl;
  const program = new Program(idl, provider);

  const accts = program.account as unknown as Record<
    string,
    { fetch(pda: PublicKey): Promise<Record<string, any>> }
  >;

  console.log("Payer (deploy key):", payer.publicKey.toBase58());
  console.log("Registry PDA:", registryPda.toBase58());

  const validators = VALIDATOR_KEYPAIR_PATHS.map((p) => loadKeypair(p));
  const oracle = loadKeypair(ORACLE_KEYPAIR_PATH);

  console.log("Validators:", validators.map((v) => v.publicKey.toBase58()));
  console.log("VRF Oracle:", oracle.publicKey.toBase58());
  console.log("Oracle pubkey for VRF_ORACLE_PUBKEY env:", oracle.publicKey.toBase58());

  // --- Faucet rotation funding ---------------------------------------------
  // Devnet rate-limits requestAirdrop to ~2 SOL per request per IP. Rotate
  // through multiple faucet URLs to accumulate enough SOL for rent + tx fees.
  async function fund(kp: Keypair, targetSol: number): Promise<void> {
    let accumulated = BigInt(0);
    let attempt = 0;
    let faucetIdx = 0;

    while (
      accumulated < BigInt(targetSol) * LAMPORTS_PER_SOLn &&
      attempt < MAX_FAUCET_RETRIES
    ) {
      attempt++;
      const url = FAUCET_URLS[faucetIdx % FAUCET_URLS.length];
      faucetIdx++;

      // Most devnet faucets double as RPC endpoints for requestAirdrop.
      const conn = new Connection(url, "confirmed");
      try {
        const sig = await conn.requestAirdrop(kp.publicKey, 2 * LAMPORTS_PER_SOL);
        await conn.confirmTransaction(sig, "confirmed");
        accumulated += BigInt(2) * LAMPORTS_PER_SOLn;
        console.log(
          `  funded ${kp.publicKey.toBase58().slice(0, 8)}... +2 SOL via ${url} (attempt ${attempt})`
        );
      } catch {
        // Try next faucet on next loop iteration
      }
    }

    if (accumulated < BigInt(targetSol) * LAMPORTS_PER_SOLn) {
      throw new Error(
        `Failed to fund ${kp.publicKey.toBase58()}: accumulated ${accumulated / LAMPORTS_PER_SOLn} SOL, need ${targetSol}`
      );
    }
  }

  console.log("Funding accounts via faucet rotation...");
  const funding: [Keypair, number][] = [
    [payer, 100],
    ...validators.map((v): [Keypair, number] => [v, 20]),
    [oracle, 20],
  ];
  for (const [kp, sol] of funding) {
    await fund(kp, sol);
  }

  // 1. Create the registry (bootstrap admin = payer = DEPLOY_KEYPAIR).
  console.log("Creating registry...");
  await program.methods
    .createRegistry()
    .accounts({
      registry: registryPda,
      admin: payer.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .signers([payer])
    .rpc();

  const registry = await accts.validatorRegistry.fetch(registryPda);
  assert.strictEqual(registry.admin.toBase58(), payer.publicKey.toBase58());
  assert.strictEqual(registry.validators.length, 0);
  assert.isFalse(registry.paused);
  console.log("Registry created; admin =", payer.publicKey.toBase58());

  // 2. Pin the VRF oracle (route_task rejects any other key).
  console.log("Pinning VRF oracle...");
  await program.methods
    .initVrfAuthority(oracle.publicKey)
    .accounts({
      vrfAuthority: vrfAuthorityPda,
      registry: registryPda,
      admin: payer.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .signers([payer])
    .rpc();
  console.log("VRF oracle pinned:", oracle.publicKey.toBase58());
  console.log(
    "Set VRF_ORACLE_PUBKEY=" + oracle.publicKey.toBase58() + " in the API environment."
  );

  // 3. Bootstrap validators with profiles, availability, reputation.
  console.log("Bootstrapping", validators.length, "validators...");
  for (const v of validators) {
    await program.methods
      .addValidatorToRegistry(v.publicKey)
      .accounts({
        registry: registryPda,
        adminSigner: payer.publicKey,
        endorsement: endorsementPda(v.publicKey),
        validator: v.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([payer])
      .rpc();

    await program.methods
      .initValidatorProfile(
        Buffer.from(`profile:${v.publicKey.toBase58()}`),
        "devnet validator"
      )
      .accounts({
        profile: profilePda(v.publicKey),
        wallet: v.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([v])
      .rpc();

    await program.methods
      .setValidatorAvailability(AVAILABILITY_ONLINE)
      .accounts({
        availability: availabilityPda(v.publicKey),
        profile: profilePda(v.publicKey),
        wallet: v.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([v])
      .rpc();

    await program.methods
      .initializeValidatorReputation()
      .accounts({
        reputation: reputationPda(v.publicKey),
        validator: v.publicKey,
        registry: registryPda,
        payer: payer.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([payer])
      .rpc();

    console.log("  Validator", v.publicKey.toBase58(), "registered + profiled + available + reputed");
  }

  const reg = await accts.validatorRegistry.fetch(registryPda);
  assert.strictEqual(reg.validators.length, validators.length);
  console.log("All", validators.length, "validators in registry.");

  // 4. Set fee policy (1%).
  console.log("Setting fee policy...");
  await program.methods
    .setFeePolicy(100) // 1%
    .accounts({
      feePolicy: feePolicyPda,
      registry: registryPda,
      admin: payer.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .signers([payer])
    .rpc();
  console.log("Fee policy set (1%).");

  // 5. Set coverage incentive.
  console.log("Setting coverage incentive...");
  await program.methods
    .setCoverageIncentive(100, 100, 100, 100, 500)
    .accounts({
      coverageIncentive: coveragePda,
      registry: registryPda,
      admin: payer.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .signers([payer])
    .rpc();
  console.log("Coverage incentive set.");

  console.log("\n=== Devnet initialization complete ===");
  console.log("Program ID:", PROGRAM_ID.toBase58());
  console.log("Registry PDA:", registryPda.toBase58());
  console.log("VRF Authority PDA:", vrfAuthorityPda.toBase58());
  console.log("Fee Policy PDA:", feePolicyPda.toBase58());
  console.log("Coverage Incentive PDA:", coveragePda.toBase58());
  console.log("Treasury PDA:", treasuryPda.toBase58());
  console.log("Payer (admin, durable):", payer.publicKey.toBase58());
  console.log("VRF Oracle (durable):", oracle.publicKey.toBase58());
  console.log("Validators:", validators.map((v) => v.publicKey.toBase58()));
}

main().catch((err) => {
  console.error("init-devnet failed:", err);
  process.exit(1);
});
