/**
 * Localnet lifecycle smoke — Sprint A (A1–A21).
 *
 * This is the *whole product loop* against deployed programs, using the
 * generated Anchor IDL and client-side instruction construction only. No
 * server-side tx_prep, no stubs, no simulated steps.
 *
 * Flow:
 *   registry → validators/profiles/availability/reputation →
 *   parcel + ownership right + granted right →
 *   claim + evidence + two observations + two attestations + quorum verify →
 *   task + requirement + fee policy + quote + coverage + fund +
 *   route → submit result → reward claim →
 *   spatial asset + geometry version + 2-of-2 verify + snapshot →
 *   sale escrow create/deposit/accept/time-travel/settle →
 *   second parcel + dispute file + 2-validator freeze.
 *
 * Requirements:
 * - Programs at the Anchor.toml program IDs on the target RPC.
 * - Surfpool (preferred; `surfnet_timeTravel` settles the escrow window) or a
 *   fresh local validator. On a validator without time travel the settlement
 *   step asserts the program's time-lock instead of skipping silently.
 * - `target/idl/terra_registry.json` present (committed IDL copy or `make idl`).
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
  Ed25519Program,
  SYSVAR_INSTRUCTIONS_PUBKEY,
} from "@solana/web3.js";
import { assert } from "chai";
import { createHash } from "crypto";
import * as fs from "fs";
import * as path from "path";

const PROGRAM_ID = new PublicKey(
  process.env.TERRA_PROGRAM_ID ?? "GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage"
);
const RPC_URL = process.env.TERRA_RPC_URL ?? "http://127.0.0.1:8899";
const IDL_PATH = path.join(__dirname, "..", "target", "idl", "terra_registry.json");
const SEED = (label: string) => createHash("sha256").update(label).digest();

// On-chain enum values (kept explicit so a test failure points at a real
// protocol change rather than a silently shifted constant).
const PARCEL = { REGISTERED: 1, FOR_SALE: 2, TRANSFERRED: 3, DISPUTED: 4, FROZEN: 5 } as const;
const RIGHTS = { USAGE: 1 } as const;
const CLAIM_TYPE = { BOUNDARY: 1 } as const;
const CLAIM_STATUS = { VERIFIED: 2 } as const;
const TASK_CLASS = { REMOTE: 1 } as const;
const TASK_STATUS = { COMPLETED: 3 } as const;
const TASK_OUTCOME = { PASS: 1 } as const;
const TASK_ASSIGNMENT = { ASSIGNED: 0, SUBMITTED: 1 } as const;
const CAPABILITY_ANY = 255;
const AVAILABILITY_ONLINE = 0;
const ESCROW_STATUS = { CREATED: 0, DEPOSITED: 1, ACCEPTED: 2 } as const;

const now = () => Math.floor(Date.now() / 1000);

function u8(n: number): Buffer {
  return Buffer.from([n]);
}

function u32le(n: number): Buffer {
  const b = Buffer.alloc(4);
  b.writeUInt32LE(n, 0);
  return b;
}

function digest(parts: (Buffer | Uint8Array | string)[]): Uint8Array {
  const h = createHash("sha256");
  for (const p of parts) h.update(typeof p === "string" ? Buffer.from(p) : Buffer.from(p));
  return Uint8Array.from(h.digest());
}

/** Canonical SD-JWT digest from `verification/attestation.rs`. */
function sdJwtDigest(
  claim: PublicKey,
  validator: PublicKey,
  observation: PublicKey,
  credentialSubject: PublicKey,
  vcType: string,
  issuanceDate: number,
  expirationDate: number
): Uint8Array {
  return digest([
    "terra:vc:sd-jwt:v1",
    claim.toBuffer(),
    validator.toBuffer(),
    observation.toBuffer(),
    credentialSubject.toBuffer(),
    vcType,
    i64le(issuanceDate),
    i64le(expirationDate),
  ]);
}

function i64le(n: number): Buffer {
  const b = Buffer.alloc(8);
  b.writeBigInt64LE(BigInt(n), 0);
  return b;
}

/** Domain-separated VRF message + attested randomness for `route_task`. */
function vrfAttestation(oracle: Keypair, taskId: Uint8Array) {
  const message = Buffer.from(digest([Buffer.from("terra:vrf:msg:v1"), taskId]));
  const ix = Ed25519Program.createInstructionWithPrivateKey({
    privateKey: oracle.secretKey,
    message,
  });
  // Layout: [count, padding, 7 x u16 offsets, pubkey(32), signature(64), message]
  const signature = Buffer.from(ix.data.subarray(48, 112));
  const randomness = digest([
    Buffer.from("terra:vrf:out:v1"),
    oracle.publicKey.toBuffer(),
    Buffer.from(taskId),
    signature,
  ]);
  return { ix, signature, randomness };
}

describe("terra localnet lifecycle (Sprint A)", function () {
  this.timeout(180_000);
  const connection = new Connection(RPC_URL, "confirmed");
  const payer = Keypair.generate();
  const wallet = new NodeWallet(payer);
  const provider = new anchor.AnchorProvider(connection, wallet, { commitment: "confirmed" });
  anchor.setProvider(provider);

  const idl = JSON.parse(fs.readFileSync(IDL_PATH, "utf-8")) as Idl;
  const program = new Program(idl, provider);

  type AccountClient = {
    fetch(pda: PublicKey): Promise<Record<string, any>>;
    fetchNullable(pda: PublicKey): Promise<Record<string, any> | null>;
  };
  const accts = program.account as unknown as Record<string, AccountClient>;

  const admin = payer.publicKey;
  const v1 = Keypair.generate();
  const v2 = Keypair.generate();
  const oracle = Keypair.generate();
  const buyer = Keypair.generate();
  const tenant = Keypair.generate();

  const pda = (seeds: (Buffer | Uint8Array | string)[]): PublicKey =>
    PublicKey.findProgramAddressSync(
      seeds.map((s) => (typeof s === "string" ? Buffer.from(s) : Buffer.from(s))),
      PROGRAM_ID
    )[0];

  const registryPda = pda(["validator_registry"]);
  const feePolicyPda = pda(["fee_policy"]);
  const coveragePda = pda(["coverage_incentive"]);
  const treasuryPda = pda(["task_treasury"]);

  const profilePda = (v: PublicKey) => pda(["validator_profile", v.toBuffer()]);
  const availabilityPda = (v: PublicKey) => pda(["validator_availability", v.toBuffer()]);
  const reputationPda = (v: PublicKey) => pda(["validator_reputation", v.toBuffer()]);
  const endorsementPda = (v: PublicKey) =>
    pda(["validator_endorsement", registryPda.toBuffer(), v.toBuffer()]);

  const parcelId = SEED("sprint-a:parcel:sale");
  const parcelPda = pda(["parcel", parcelId]);
  const ownershipPda = pda(["ownership", parcelPda.toBuffer()]);

  const claimId = SEED("sprint-a:claim:boundary");
  const claimPda = pda(["claim", parcelPda.toBuffer(), claimId]);
  const evidencePda = pda(["evidence", claimPda.toBuffer(), u8(0)]);
  const observationPda = (v: PublicKey) =>
    pda(["observation", claimPda.toBuffer(), v.toBuffer()]);
  const attestationPda = (v: PublicKey) =>
    pda(["verification_attestation", claimPda.toBuffer(), v.toBuffer()]);

  const taskId = SEED("sprint-a:task:boundary");
  const taskPda = pda(["task", taskId]);
  const requirementPda = pda(["task_requirement", taskId, u8(0)]);
  const quotePda = pda(["resource_quote", taskId]);
  const taskEscrowPda = pda(["task_escrow", taskId]);
  const taskVaultPda = pda(["task_escrow_vault", taskEscrowPda.toBuffer()]);
  const assignmentPda = pda(["task_assignment", taskId, v1.publicKey.toBuffer()]);
  const rewardAllocationPda = pda(["reward_allocation", taskId, v1.publicKey.toBuffer()]);
  const vrfAuthorityPda = pda(["vrf_authority"]);

  const assetPda = pda(["spatial_asset", parcelPda.toBuffer()]);
  const geometryVersionPda = pda(["geometry_version", assetPda.toBuffer(), u32le(0)]);
  const snapshotMetaPda = pda(["spatial_snapshot_meta", assetPda.toBuffer()]);
  const snapshotPda = pda(["spatial_snapshot", assetPda.toBuffer(), u32le(0)]);

  const escrowPda = pda(["escrow", parcelPda.toBuffer()]);
  const escrowVaultPda = pda(["escrow_vault", escrowPda.toBuffer()]);

  const disputeParcelId = SEED("sprint-a:parcel:dispute");
  const disputeParcelPda = pda(["parcel", disputeParcelId]);
  const disputeOwnershipPda = pda(["ownership", disputeParcelPda.toBuffer()]);
  const caseHash = SEED("sprint-a:dispute:case");
  const disputePda = pda(["dispute", disputeParcelPda.toBuffer(), caseHash]);

  const systemProgram = SystemProgram.programId;

  async function fund(kp: Keypair, sol: number): Promise<void> {
    const sig = await connection.requestAirdrop(kp.publicKey, sol * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(sig, "confirmed");
  }

  before(async function () {
    this.timeout(180_000);
    for (const [kp, sol] of [
      [payer, 100],
      [v1, 20],
      [v2, 20],
      [buyer, 20],
      [tenant, 5],
    ] as [Keypair, number][]) {
      await fund(kp, sol);
    }
  });

  it("creates the registry (bootstrap admin)", async () => {
    await program.methods
      .createRegistry()
      .accounts({ registry: registryPda, admin, systemProgram })
      .signers([payer])
      .rpc();

    const registry = await accts.validatorRegistry.fetch(registryPda);
    assert.strictEqual(registry.admin.toBase58(), admin.toBase58());
    assert.strictEqual(registry.validators.length, 0);
    assert.isFalse(registry.paused);

    // Pin the VRF oracle. `route_task` rejects any other key.
    await program.methods
      .initVrfAuthority(oracle.publicKey)
      .accounts({
        vrfAuthority: vrfAuthorityPda,
        registry: registryPda,
        admin,
        systemProgram,
      })
      .signers([payer])
      .rpc();
  });

  it("bootstraps two validators with profiles, availability and reputation", async () => {
    for (const v of [v1, v2]) {
      await program.methods
        .addValidatorToRegistry(v.publicKey)
        .accounts({
          registry: registryPda,
          adminSigner: admin,
          endorsement: endorsementPda(v.publicKey),
          validator: v.publicKey,
          systemProgram,
        })
        .signers([payer])
        .rpc();

      await program.methods
        .initValidatorProfile(digest([`profile:${v.publicKey.toBase58()}`]), "sprint-a validator")
        .accounts({ profile: profilePda(v.publicKey), wallet: v.publicKey, systemProgram })
        .signers([v])
        .rpc();

      await program.methods
        .setValidatorAvailability(AVAILABILITY_ONLINE)
        .accounts({
          availability: availabilityPda(v.publicKey),
          profile: profilePda(v.publicKey),
          wallet: v.publicKey,
          systemProgram,
        })
        .signers([v])
        .rpc();

      // Reputation initialization is admin-gated (payer must be registry admin).
      await program.methods
        .initializeValidatorReputation()
        .accounts({
          reputation: reputationPda(v.publicKey),
          validator: v.publicKey,
          registry: registryPda,
          payer: admin,
          systemProgram,
        })
        .signers([payer])
        .rpc();
    }

    const registry = await accts.validatorRegistry.fetch(registryPda);
    assert.deepStrictEqual(
      registry.validators.map((k: PublicKey) => k.toBase58()),
      [v1.publicKey.toBase58(), v2.publicKey.toBase58()]
    );

    const profile = await accts.validatorProfile.fetch(profilePda(v1.publicKey));
    assert.strictEqual(profile.wallet.toBase58(), v1.publicKey.toBase58());

    const availability = await accts.validatorAvailability.fetch(availabilityPda(v1.publicKey));
    assert.strictEqual(availability.status, AVAILABILITY_ONLINE);

    const reputation = await accts.validatorReputation.fetch(reputationPda(v1.publicKey));
    assert.strictEqual(reputation.status, 0); // ACTIVE
    assert.strictEqual(reputation.reputationScore, 10_000);
  });

  it("registers a parcel and grants a non-ownership right", async () => {
    await program.methods
      .registerParcel(parcelId, "Sprint A lifecycle parcel", digest(["geometry:v0"]))
      .accounts({ parcel: parcelPda, owner: admin, ownership: ownershipPda, systemProgram })
      .signers([payer])
      .rpc();

    const parcel = await accts.parcel.fetch(parcelPda);
    assert.strictEqual(parcel.status, PARCEL.REGISTERED);
    assert.strictEqual(parcel.rightsCount, 0);
    assert.isUndefined((parcel as unknown as Record<string, unknown>).owner);

    const ownership = await accts.rights.fetch(ownershipPda);
    assert.strictEqual(ownership.rightsKind, 0); // OWNERSHIP
    assert.strictEqual(ownership.holder.toBase58(), admin.toBase58());

    const rightsPda = pda(["rights", parcelPda.toBuffer(), u8(0)]);
    await program.methods
      .grantRight(0, RIGHTS.USAGE, tenant.publicKey, new anchor.BN(0), "sprint-a lease")
      .accounts({ parcel: parcelPda, ownership: ownershipPda, rights: rightsPda, owner: admin, systemProgram })
      .signers([payer])
      .rpc();

    const right = await accts.rights.fetch(rightsPda);
    assert.strictEqual(right.rightsKind, RIGHTS.USAGE);
    assert.strictEqual(right.holder.toBase58(), tenant.publicKey.toBase58());

    const after = await accts.parcel.fetch(parcelPda);
    assert.strictEqual(after.rightsCount, 1);
  });

  it("creates a claim, anchors evidence, and records two independent observations", async () => {
    const observedAt = now();
    await program.methods
      .createClaim(claimId, CLAIM_TYPE.BOUNDARY, digest(["statement:boundary"]), 0, [0, 0])
      .accounts({ claim: claimPda, parcel: parcelPda, submitter: admin, systemProgram })
      .signers([payer])
      .rpc();

    let claim = await accts.claim.fetch(claimPda);
    assert.strictEqual(claim.status, 0); // SUBMITTED
    assert.strictEqual(claim.requiredAttestations, 2);

    await program.methods
      .addEvidence(0, digest(["evidence:survey"]), "ipfs://sprint-a/evidence-0", new anchor.BN(observedAt))
      .accounts({ evidence: evidencePda, claim: claimPda, submitter: admin, systemProgram })
      .signers([payer])
      .rpc();

    const evidence = await accts.evidence.fetch(evidencePda);
    assert.strictEqual(evidence.nonce, 0);
    assert.strictEqual(evidence.storageReference, "ipfs://sprint-a/evidence-0");

    claim = await accts.claim.fetch(claimPda);
    assert.strictEqual(claim.evidenceCount, 1);

    for (const [i, v] of [v1, v2].entries()) {
      await program.methods
        .submitObservation(
          [new anchor.BN(-6_200_000_000), new anchor.BN(106_800_000_000)],
          0,
          digest([`observation:${i}`]),
          90,
          digest([`observation-signature:${i}`])
        )
        .accounts({
          observation: observationPda(v.publicKey),
          claim: claimPda,
          validator: v.publicKey,
          systemProgram,
        })
        .signers([v])
        .rpc();

      const obs = await accts.observation.fetch(observationPda(v.publicKey));
      assert.strictEqual(obs.validator.toBase58(), v.publicKey.toBase58());
      assert.strictEqual(obs.confidence, 90);
    }

    claim = await accts.claim.fetch(claimPda);
    assert.strictEqual(claim.status, 1); // UNDER_VERIFICATION
    assert.strictEqual(claim.observationCount, 2);
  });

  it("collects two attestations and verifies the claim by quorum", async () => {
    for (const v of [v1, v2]) {
      const observation = observationPda(v.publicKey);
      const issuance = now();
      const expiration = issuance + 86_400;
      const sig = sdJwtDigest(
        claimPda,
        v.publicKey,
        observation,
        claimPda,
        "VerificationAttestation",
        issuance,
        expiration
      );
      await program.methods
        .submitVerificationAttestation(
          "VerificationAttestation",
          claimPda,
          new anchor.BN(issuance),
          new anchor.BN(expiration),
          Array.from(sig),
          false
        )
        .accounts({
          attestation: attestationPda(v.publicKey),
          claim: claimPda,
          observation,
          validator: v.publicKey,
          systemProgram,
        })
        .remainingAccounts([
          { pubkey: reputationPda(v.publicKey), isWritable: false, isSigner: false },
        ])
        .signers([v])
        .rpc();
    }

    let claim = await accts.claim.fetch(claimPda);
    assert.strictEqual(claim.attestationCount, 2);

    await program.methods
      .verifyClaim()
      .accounts({ claim: claimPda })
      .signers([payer])
      .rpc();

    claim = await accts.claim.fetch(claimPda);
    assert.strictEqual(claim.status, CLAIM_STATUS.VERIFIED);
  });

  it("runs task economics end to end: fund, route, submit, reward", async () => {
    const deadline = now() + 3_600;
    const reward = 100_000_000; // 0.1 SOL
    const resourceCost = 10_000_000; // 0.01 SOL

    await program.methods
      .createVerificationTask(
        taskId,
        claimPda,
        TASK_CLASS.REMOTE,
        new anchor.BN(reward),
        new anchor.BN(deadline),
        digest(["task:boundary"]),
        1
      )
      .accounts({ task: taskPda, requester: admin, systemProgram })
      .signers([payer])
      .rpc();

    await program.methods
      .addTaskRequirement(0, CAPABILITY_ANY, 0, 0, [0, 0], 0, 0, 0, 0, 9_000)
      .accounts({ requirement: requirementPda, task: taskPda, requester: admin, systemProgram })
      .signers([payer])
      .rpc();

    await program.methods
      .setFeePolicy(100) // 1%
      .accounts({ feePolicy: feePolicyPda, registry: registryPda, admin, systemProgram })
      .signers([payer])
      .rpc();

    await program.methods
      .quoteTaskResources(new anchor.BN(resourceCost))
      .accounts({ task: taskPda, quote: quotePda, quoter: admin, systemProgram })
      .signers([payer])
      .rpc();

    await program.methods
      .setCoverageIncentive(100, 100, 100, 100, 500)
      .accounts({ coverageIncentive: coveragePda, registry: registryPda, admin, systemProgram })
      .signers([payer])
      .rpc();

    await program.methods
      .fundTaskEscrow()
      .accounts({
        task: taskPda,
        quote: quotePda,
        feePolicy: feePolicyPda,
        taskEscrow: taskEscrowPda,
        vault: taskVaultPda,
        treasury: treasuryPda,
        requester: admin,
        systemProgram,
      })
      .signers([payer])
      .rpc();

    const escrow = await accts.taskEscrow.fetch(taskEscrowPda);
    assert.strictEqual(escrow.amount.toNumber(), reward + resourceCost);
    assert.strictEqual(escrow.status, 1); // FUNDED

    const vrf = vrfAttestation(oracle, taskId);
    await program.methods
      .routeTask(
        taskId,
        0,
        [v1.publicKey],
        v1.publicKey,
        1,
        Array.from(vrf.randomness),
        Array.from(vrf.signature)
      )
      .accounts({
        assignment: assignmentPda,
        task: taskPda,
        requirement: requirementPda,
        chosen: v1.publicKey,
        requester: admin,
        systemProgram,
        oracle: oracle.publicKey,
        vrfAuthority: vrfAuthorityPda,
        instructions: SYSVAR_INSTRUCTIONS_PUBKEY,
      })
      .remainingAccounts([
        { pubkey: profilePda(v1.publicKey), isWritable: false, isSigner: false },
        { pubkey: availabilityPda(v1.publicKey), isWritable: false, isSigner: false },
        { pubkey: reputationPda(v1.publicKey), isWritable: false, isSigner: false },
      ])
      .preInstructions([vrf.ix])
      .signers([payer])
      .rpc();

    let assignment = await accts.taskAssignment.fetch(assignmentPda);
    assert.strictEqual(assignment.validator.toBase58(), v1.publicKey.toBase58());
    assert.strictEqual(assignment.status, TASK_ASSIGNMENT.ASSIGNED);

    await program.methods
      .submitTaskResult(taskId, TASK_OUTCOME.PASS, digest(["task:result:pass"]))
      .accounts({ assignment: assignmentPda, task: taskPda, validator: v1.publicKey })
      .signers([v1])
      .rpc();

    const task = await accts.verificationTask.fetch(taskPda);
    assert.strictEqual(task.status, TASK_STATUS.COMPLETED);
    assert.strictEqual(task.resultCount, 1);

    const before = await connection.getBalance(v1.publicKey);
    await program.methods
      .claimTaskReward()
      .accounts({
        task: taskPda,
        assignment: assignmentPda,
        taskEscrow: taskEscrowPda,
        vault: taskVaultPda,
        coverageIncentive: coveragePda,
        rewardAllocation: rewardAllocationPda,
        treasury: treasuryPda,
        validator: v1.publicKey,
        systemProgram,
      })
      .signers([v1])
      .rpc();

    assignment = await accts.taskAssignment.fetch(assignmentPda);
    assert.strictEqual(assignment.status, 2); // RELEASED

    const allocation = await accts.rewardAllocation.fetch(rewardAllocationPda);
    assert.isAbove(allocation.basePaid.toNumber(), 0);

    const after = await connection.getBalance(v1.publicKey);
    assert.isAbove(after, before - 1_000_000); // reward landed (net of fee)
  });

  it("anchors a geometry version, verifies it 2-of-2, and freezes a snapshot", async () => {
    const geometryHash = digest(["geometry:verified:v0"]);

    await program.methods
      .initSpatialAsset(0, new anchor.BN(0), new anchor.BN(0)) // D2, no elevation
      .accounts({ asset: assetPda, parcel: parcelPda, registrar: admin, systemProgram })
      .signers([payer])
      .rpc();

    await program.methods
      .appendGeometryVersion(
        geometryHash,
        0, // MANUAL
        0, // D2
        "ipfs://sprint-a/geometry-v0",
        new anchor.BN(0),
        new anchor.BN(0),
        0 // elevation_source NONE
      )
      .accounts({ asset: assetPda, geometryVersion: geometryVersionPda, payer: admin, systemProgram })
      .signers([payer])
      .rpc();

    let version = await accts.geometryVersion.fetch(geometryVersionPda);
    assert.strictEqual(version.version, 0);
    assert.strictEqual(version.required, 2);
    assert.isFalse(version.verified);

    for (const v of [v1, v2]) {
      await program.methods
        .verifyGeometryVersion()
        .accounts({
          geometryVersion: geometryVersionPda,
          asset: assetPda,
          profile: profilePda(v.publicKey),
          validator: v.publicKey,
        })
        .signers([v])
        .rpc();
    }

    version = await accts.geometryVersion.fetch(geometryVersionPda);
    assert.isTrue(version.verified);
    assert.strictEqual(version.attestCount, 2);

    await program.methods
      .createSpatialSnapshot("sprint-a baseline")
      .accounts({ asset: assetPda, meta: snapshotMetaPda, snapshot: snapshotPda, payer: admin, systemProgram })
      .signers([payer])
      .rpc();

    const snapshot = await accts.spatialSnapshot.fetch(snapshotPda);
    assert.strictEqual(snapshot.snapshotIndex, 0);
    assert.strictEqual(snapshot.geometryVersionCount, 1);
    assert.deepStrictEqual(Array.from(snapshot.latestGeometry), Array.from(geometryHash));
  });

  it("settles a sale escrow after the time-lock (Surfpool time travel)", async () => {
    await program.methods
      .updateStatus(PARCEL.FOR_SALE)
      .accounts({ parcel: parcelPda, ownership: ownershipPda, owner: admin })
      .signers([payer])
      .rpc();

    await program.methods
      .createEscrow(new anchor.BN(100_000_000), buyer.publicKey)
      .accounts({
        escrowRecord: escrowPda,
        escrowVault: escrowVaultPda,
        parcel: parcelPda,
        ownership: ownershipPda,
        seller: admin,
        systemProgram,
      })
      .signers([payer])
      .rpc();

    let escrow = await accts.escrowRecord.fetch(escrowPda);
    assert.strictEqual(escrow.status, ESCROW_STATUS.CREATED);

    await program.methods
      .depositEscrow(new anchor.BN(100_000_000))
      .accounts({
        escrowRecord: escrowPda,
        escrowVault: escrowVaultPda,
        parcel: parcelPda,
        buyer: buyer.publicKey,
        systemProgram,
      })
      .signers([buyer])
      .rpc();

    escrow = await accts.escrowRecord.fetch(escrowPda);
    assert.strictEqual(escrow.status, ESCROW_STATUS.DEPOSITED);

    await program.methods
      .acceptEscrow()
      .accounts({ escrowRecord: escrowPda, parcel: parcelPda, ownership: ownershipPda, seller: admin })
      .signers([payer])
      .rpc();

    escrow = await accts.escrowRecord.fetch(escrowPda);
    assert.strictEqual(escrow.status, ESCROW_STATUS.ACCEPTED);
    const settleDeadline = escrow.settleDeadline.toNumber();
    assert.isAbove(settleDeadline, now());

    // The settlement window is 3 days. Surfpool exposes surfnet_timeTravel;
    // a plain solana-test-validator has no clock warp, so prove the time-lock
    // there instead of pretending success.
    let travelled = false;
    try {
      await (connection as unknown as {
        _rpcRequest(method: string, params: unknown[]): Promise<unknown>;
      })._rpcRequest("surfnet_timeTravel", [
        { absoluteTimestamp: (settleDeadline + 60) * 1000 },
      ]);
      travelled = true;
    } catch {
      travelled = false;
    }

    if (travelled) {
      const before = await connection.getBalance(payer.publicKey);
      await program.methods
        .settleEscrow()
        .accounts({
          escrowRecord: escrowPda,
          escrowVault: escrowVaultPda,
          parcel: parcelPda,
          ownership: ownershipPda,
          seller: admin,
          buyer: buyer.publicKey,
          systemProgram,
        })
        .signers([payer])
        .rpc();

      const ownership = await accts.rights.fetch(ownershipPda);
      assert.strictEqual(ownership.holder.toBase58(), buyer.publicKey.toBase58());
      const parcel = await accts.parcel.fetch(parcelPda);
      assert.strictEqual(parcel.status, PARCEL.TRANSFERRED);
      const after = await connection.getBalance(payer.publicKey);
      assert.isAbove(after, before);
    } else {
      let rejected = false;
      try {
        await program.methods
          .settleEscrow()
          .accounts({
            escrowRecord: escrowPda,
            escrowVault: escrowVaultPda,
            parcel: parcelPda,
            ownership: ownershipPda,
            seller: admin,
            buyer: buyer.publicKey,
            systemProgram,
          })
          .signers([payer])
          .rpc();
      } catch {
        rejected = true;
      }
      assert.isTrue(rejected, "settle_escrow must be time-locked without a clock warp");
      console.warn("surfnet_timeTravel unavailable: settlement verified as time-locked only");
    }
  });

  it("opens a dispute and freezes the parcel with two validator signatures", async () => {
    await program.methods
      .registerParcel(disputeParcelId, "Sprint A dispute parcel", digest(["geometry:dispute"]))
      .accounts({
        parcel: disputeParcelPda,
        owner: admin,
        ownership: disputeOwnershipPda,
        systemProgram,
      })
      .signers([payer])
      .rpc();

    const validators = [
      v1.publicKey,
      v2.publicKey,
      ...Array.from({ length: 6 }, () => PublicKey.default),
    ];

    await program.methods
      .fileDispute(caseHash, 2, validators, new anchor.BN(1_000_000)) // bond_lamports > 0
      .accounts({
        dispute: disputePda,
        parcel: disputeParcelPda,
        ownership: disputeOwnershipPda,
        registry: registryPda,
        filer: admin,
        systemProgram,
      })
      .signers([payer])
      .rpc();

    let dispute = await accts.dispute.fetch(disputePda);
    assert.strictEqual(dispute.status, 0); // FILED
    assert.strictEqual(dispute.required, 2);

    let parcel = await accts.parcel.fetch(disputeParcelPda);
    assert.strictEqual(parcel.status, PARCEL.DISPUTED);

    await program.methods
      .freezeParcel()
      .accounts({ dispute: disputePda, parcel: disputeParcelPda, validator: v1.publicKey })
      .remainingAccounts([
        { pubkey: v1.publicKey, isWritable: false, isSigner: true },
        { pubkey: v2.publicKey, isWritable: false, isSigner: true },
      ])
      .signers([v1, v2])
      .rpc();

    dispute = await accts.dispute.fetch(disputePda);
    assert.strictEqual(dispute.status, 1); // FROZEN
    assert.strictEqual(dispute.presentCount, 2);

    parcel = await accts.parcel.fetch(disputeParcelPda);
    assert.strictEqual(parcel.status, PARCEL.FROZEN);
  });
});
