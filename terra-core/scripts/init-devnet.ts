#!/usr/bin/env npx ts-node
/**
 * scripts/init-devnet.ts — Post-deploy protocol initialization for devnet.
 *
 * Ports the on-chain initialization sequence from
 * tests/localnet_e2e.ts (Sprint A bootstrap) so that after `deploy.sh devnet`
 * the protocol is actually usable — not just two deployed .so files with
 * zero accounts.
 *
 * Initializes, in order:
 *   1. validator_registry (bootstrap admin = payer)
 *   2. vrf_authority (pins the route_task oracle key)
 *   3. N validators: addValidatorToRegistry + initValidatorProfile +
 *      setValidatorAvailability + initializeValidatorReputation
 *   4. fee_policy (setFeePolicy)
 *   5. coverage_incentive (setCoverageIncentive)
 *
 * Reads program IDs from Anchor.toml / environment, connects to the target
 * RPC, funds the payer + validators via airdrop, then submits every
 * instruction with real signers. Idempotent-ish: re-running will error on
 * already-existing PDAs (expected — use a fresh devnet or accept the errors).
 *
 * Usage:
 *   TERRA_RPC_URL=http://... TERRA_PROGRAM_ID=... npx ts-node scripts/init-devnet.ts
 *
 * Env:
 *   TERRA_RPC_URL        JSON-RPC endpoint (default http://127.0.0.1:8899)
 *   TERRA_PROGRAM_ID     terra_registry program ID (default from Anchor.toml)
 *   TERRA_IDENTITY_ID    terra_identity program ID (default from Anchor.toml)
 *   NUM_VALIDATORS       how many validator keypairs to bootstrap (default 2)
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

const PROGRAM_ID = new PublicKey(
  process.env.TERRA_PROGRAM_ID ?? "GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage"
);
const IDENTITY_ID = new PublicKey(
  process.env.TERRA_IDENTITY_ID ?? "HyjpaHY9yQg1KneLZGjpLDQfP9WV4LUEbNZcb7C4Vo6p"
);
const RPC_URL = process.env.TERRA_RPC_URL ?? "http://127.0.0.1:8899";
const NUM_VALIDATORS = Number(process.env.NUM_VALIDATORS ?? "2");
const IDL_PATH = path.join(__dirname, "..", "target", "idl", "terra_registry.json");

const PARCEL = { REGISTERED: 1 } as const;
const RIGHTS = { USAGE: 1 } as const;
const AVAILABILITY_ONLINE = 0;
const CAPABILITY_ANY = 255;

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

const registryPda = pda(["validator_registry"]);
const feePolicyPda = pda(["fee_policy"]);
const coveragePda = pda(["coverage_incentive"]);
const treasuryPda = pda(["task_treasury"]);
const vrfAuthorityPda = pda(["vrf_authority"]);

const validators: Keypair[] = Array.from({ length: NUM_VALIDATORS }, () => Keypair.generate());
const oracle = Keypair.generate();

const profilePda = (v: PublicKey) => pda(["validator_profile", v.toBuffer()]);
const availabilityPda = (v: PublicKey) => pda(["validator_availability", v.toBuffer()]);
const reputationPda = (v: PublicKey) => pda(["validator_reputation", v.toBuffer()]);
const endorsementPda = (v: PublicKey) =>
  pda(["validator_endorsement", registryPda.toBuffer(), v.toBuffer()]);

async function main() {
  const connection = new Connection(RPC_URL, "confirmed");
  const payer = Keypair.generate();
  const wallet = new NodeWallet(payer);
  const provider = new anchor.AnchorProvider(connection, wallet, { commitment: "confirmed" });
  anchor.setProvider(provider);

  const idl = JSON.parse(fs.readFileSync(IDL_PATH, "utf-8")) as Idl;
  const program = new Program(idl, provider);

  const accts = program.account as unknown as Record<
    string,
    { fetch(pda: PublicKey): Promise<Record<string, any>> }
  >;

  console.log("Payer:", payer.publicKey.toBase58());
  console.log("Registry PDA:", registryPda.toBase58());
  console.log("Validators:", validators.map((v) => v.publicKey.toBase58()));
  console.log("VRF Oracle:", oracle.publicKey.toBase58());

  // Fund payer + validators via airdrop (devnet/localnet only).
  async function fund(kp: Keypair, sol: number): Promise<void> {
    const sig = await connection.requestAirdrop(kp.publicKey, sol * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(sig, "confirmed");
  }

  console.log("Funding accounts...");
  const funding: [Keypair, number][] = [
    [payer, 100],
    ...validators.map((v): [Keypair, number] => [v, 20]),
    [oracle, 20],
  ];
  for (const [kp, sol] of funding) {
    await fund(kp, sol);
  }

  // 1. Create the registry (bootstrap admin = payer).
  console.log("Creating registry...");
  await program.methods
    .createRegistry()
    .accounts({ registry: registryPda, admin: payer.publicKey, systemProgram: SystemProgram.programId })
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

  // 3. Bootstrap validators with profiles, availability, reputation.
  console.log("Bootstrapping", NUM_VALIDATORS, "validators...");
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
      .accounts({ profile: profilePda(v.publicKey), wallet: v.publicKey, systemProgram: SystemProgram.programId })
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
  assert.strictEqual(reg.validators.length, NUM_VALIDATORS);
  console.log("All", NUM_VALIDATORS, "validators in registry.");

  // 4. Set fee policy (1%).
  console.log("Setting fee policy...");
  await program.methods
    .setFeePolicy(100) // 1%
    .accounts({ feePolicy: feePolicyPda, registry: registryPda, admin: payer.publicKey, systemProgram: SystemProgram.programId })
    .signers([payer])
    .rpc();
  console.log("Fee policy set (1%).");

  // 5. Set coverage incentive.
  console.log("Setting coverage incentive...");
  await program.methods
    .setCoverageIncentive(100, 100, 100, 100, 500)
    .accounts({ coverageIncentive: coveragePda, registry: registryPda, admin: payer.publicKey, systemProgram: SystemProgram.programId })
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
  console.log("Payer (admin):", payer.publicKey.toBase58());
  console.log("VRF Oracle:", oracle.publicKey.toBase58());
}

main().catch((err) => {
  console.error("init-devnet failed:", err);
  process.exit(1);
});
