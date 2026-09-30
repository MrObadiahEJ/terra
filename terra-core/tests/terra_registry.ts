/**
 * Local-validator E2E smoke test — RRR ownership model.
 *
 * Runs against `solana-test-validator` (requires an AVX-capable CPU) with
 * terra_registry deployed at TERRA_PROGRAM_ID. From `terra-core/`:
 *
 *   make idl && make deploy && make test-ts
 *
 * Architecture under test — the Parcel account carries NO ownership field.
 * Ownership is the canonical `Rights` PDA at `["ownership", parcel]`
 * (`rights_kind == OWNERSHIP`), created atomically with the parcel:
 *
 *   Parcel ──["ownership", parcel]──▶ canonical ownership Rights PDA
 *                                        holder = wallet | Identity PDA
 *
 * Assertions here deliberately reject the legacy model (`Parcel.owner`) —
 * do NOT reintroduce a parcel-level owner to make a test pass.
 *
 * Deep invariant / adversarial coverage (identity-holder authorization,
 * remaining_accounts substitution, fake PDA rejection, ...) lives in the
 * Rust BPF suite: `programs/terra_registry/tests/integration.rs`.
 * This file is the client-side smoke test for the current account model.
 */
import * as anchor from "@coral-xyz/anchor";
import { Program, Idl } from "@coral-xyz/anchor";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { Keypair, PublicKey, Connection, SystemProgram } from "@solana/web3.js";
import { assert } from "chai";
import * as fs from "fs";
import * as path from "path";

const IDL_PATH = path.join(__dirname, "..", "target", "idl", "terra_registry.json");

const RIGHT_KIND = { OWNERSHIP: 0, USAGE: 1 } as const;
const PARCEL_STATUS = { REGISTERED: 1, FOR_SALE: 2 } as const;
const RIGHT_STATUS = { ACTIVE: 0 } as const;

async function expectRejected(label: string, promise: Promise<unknown>): Promise<void> {
  try {
    await promise;
  } catch {
    return;
  }
  assert.fail(`${label}: expected the transaction to be rejected`);
}

describe("terra-registry (RRR ownership model)", () => {
  const programId = new PublicKey("GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage");
  const connection = new Connection("http://127.0.0.1:8899", "confirmed");

  const payer = Keypair.generate();
  const wallet = new NodeWallet(payer);
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });
  anchor.setProvider(provider);

  const idl = JSON.parse(fs.readFileSync(IDL_PATH, "utf-8")) as Idl;
  const program = new Program(idl, provider);

  // `make idl` emits JSON only; a generic `Idl` has no per-account typing,
  // so resolve account clients through this small shim (runtime names come
  // from the generated IDL).
  type AccountClient = {
    fetch(pda: PublicKey): Promise<Record<string, any>>;
    fetchNullable(pda: PublicKey): Promise<Record<string, any> | null>;
  };
  const accts = program.account as unknown as {
    parcel: AccountClient;
    rights: AccountClient;
  };

  const owner = payer.publicKey;
  const newOwner = Keypair.generate();
  const holder = Keypair.generate();
  const holder2 = Keypair.generate();
  const stranger = Keypair.generate();

  const parcelId = Uint8Array.from({ length: 32 }, (_, i) => i + 1);
  const geometryHash = Uint8Array.from({ length: 32 }, (_, i) => i * 2);

  const parcelPda = PublicKey.findProgramAddressSync(
    [Buffer.from("parcel"), Buffer.from(parcelId)],
    programId
  )[0];
  const ownershipPda = PublicKey.findProgramAddressSync(
    [Buffer.from("ownership"), parcelPda.toBuffer()],
    programId
  )[0];
  const rightsPda = (nonce: number) =>
    PublicKey.findProgramAddressSync(
      [Buffer.from("rights"), parcelPda.toBuffer(), Buffer.from([nonce])],
      programId
    )[0];

  const registerAccounts = () => ({
    parcel: parcelPda,
    owner,
    ownership: ownershipPda,
    systemProgram: SystemProgram.programId,
  });

  before(async () => {
    // Fund the payer against the local validator.
    const airdropSig = await connection.requestAirdrop(
      payer.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(airdropSig, "confirmed");
  });

  it("register_parcel creates the parcel AND the canonical ownership right", async () => {
    await program.methods
      .registerParcel(parcelId, "Soa/Biteng demo parcel", geometryHash)
      .accounts(registerAccounts())
      .signers([payer])
      .rpc();

    const parcel = await accts.parcel.fetch(parcelPda);
    assert.strictEqual(parcel.name, "Soa/Biteng demo parcel");
    assert.strictEqual(parcel.status, PARCEL_STATUS.REGISTERED);
    assert.strictEqual(parcel.rightsCount, 0);
    // RRR: ownership is not a Parcel field — assert the legacy model stays gone.
    assert.isUndefined((parcel as unknown as Record<string, unknown>).owner);

    const ownership = await accts.rights.fetch(ownershipPda);
    assert.strictEqual(ownership.parcel.toBase58(), parcelPda.toBase58());
    assert.strictEqual(ownership.rightsKind, RIGHT_KIND.OWNERSHIP);
    assert.strictEqual(ownership.holder.toBase58(), owner.toBase58());
    assert.strictEqual(ownership.granter.toBase58(), owner.toBase58());
    assert.strictEqual(ownership.status, RIGHT_STATUS.ACTIVE);
    assert.strictEqual(ownership.expiresAt.toNumber(), 0);
  });

  it("rejects duplicate id (same PDA)", async () => {
    await expectRejected(
      "duplicate register_parcel",
      program.methods
        .registerParcel(parcelId, "dupe", geometryHash)
        .accounts(registerAccounts())
        .signers([payer])
        .rpc()
    );
  });

  it("transfers the ownership right; old holder loses authorization", async () => {
    await program.methods
      .transferParcel()
      .accounts({ parcel: parcelPda, ownership: ownershipPda, owner, newOwner: newOwner.publicKey })
      .signers([payer])
      .rpc();

    // The canonical ownership PDA moved — no Parcel field did.
    const ownership = await accts.rights.fetch(ownershipPda);
    assert.strictEqual(ownership.holder.toBase58(), newOwner.publicKey.toBase58());
    assert.strictEqual(ownership.granter.toBase58(), owner.toBase58());
    const parcel = await accts.parcel.fetch(parcelPda);
    assert.isUndefined((parcel as unknown as Record<string, unknown>).owner);

    // Old holder is rejected by the handler (still a valid signer account).
    await expectRejected(
      "old holder update_status",
      program.methods
        .updateStatus(PARCEL_STATUS.FOR_SALE)
        .accounts({ parcel: parcelPda, ownership: ownershipPda, owner })
        .signers([payer])
        .rpc()
    );

    // New holder can authorize mutations.
    await program.methods
      .updateStatus(PARCEL_STATUS.FOR_SALE)
      .accounts({ parcel: parcelPda, ownership: ownershipPda, owner: newOwner.publicKey })
      .signers([newOwner])
      .rpc();
    const updated = await accts.parcel.fetch(parcelPda);
    assert.strictEqual(updated.status, PARCEL_STATUS.FOR_SALE);

    // Transfer back so the original wallet drives the rest of the suite.
    await program.methods
      .transferParcel()
      .accounts({
        parcel: parcelPda,
        ownership: ownershipPda,
        owner: newOwner.publicKey,
        newOwner: owner,
      })
      .signers([newOwner])
      .rpc();
    const back = await accts.rights.fetch(ownershipPda);
    assert.strictEqual(back.holder.toBase58(), owner.toBase58());
  });

  it("grant_right allocates nonce-indexed rights; guards nonce and kind", async () => {
    await program.methods
      .grantRight(0, RIGHT_KIND.USAGE, holder.publicKey, 0, "")
      .accounts({
        parcel: parcelPda,
        ownership: ownershipPda,
        rights: rightsPda(0),
        owner,
        systemProgram: SystemProgram.programId,
      })
      .signers([payer])
      .rpc();

    const parcel = await accts.parcel.fetch(parcelPda);
    assert.strictEqual(parcel.rightsCount, 1);

    const right = await accts.rights.fetch(rightsPda(0));
    assert.strictEqual(right.parcel.toBase58(), parcelPda.toBase58());
    assert.strictEqual(right.rightsKind, RIGHT_KIND.USAGE);
    assert.strictEqual(right.holder.toBase58(), holder.publicKey.toBase58());
    assert.strictEqual(right.granter.toBase58(), owner.toBase58());
    assert.strictEqual(right.status, RIGHT_STATUS.ACTIVE);

    // Granting a non-ownership right must not disturb the ownership PDA.
    const ownership = await accts.rights.fetch(ownershipPda);
    assert.strictEqual(ownership.rightsKind, RIGHT_KIND.OWNERSHIP);
    assert.strictEqual(ownership.holder.toBase58(), owner.toBase58());

    // Fresh PDA per attempt so the handler's guard (not account init) fires:
    // nonce != rights_count → InvalidNonce.
    await expectRejected(
      "grant with stale nonce",
      program.methods
        .grantRight(7, RIGHT_KIND.USAGE, holder.publicKey, 0, "")
        .accounts({
          parcel: parcelPda,
          ownership: ownershipPda,
          rights: rightsPda(7),
          owner,
          systemProgram: SystemProgram.programId,
        })
        .signers([payer])
        .rpc()
    );

    // OWNERSHIP is excluded from grant_right — it exists only at the
    // canonical ["ownership", parcel] PDA and moves via transfer_parcel.
    await expectRejected(
      "grant OWNERSHIP kind",
      program.methods
        .grantRight(8, RIGHT_KIND.OWNERSHIP, holder.publicKey, 0, "")
        .accounts({
          parcel: parcelPda,
          ownership: ownershipPda,
          rights: rightsPda(8),
          owner,
          systemProgram: SystemProgram.programId,
        })
        .signers([payer])
        .rpc()
    );
  });

  it("revoke_right closes the account, decrements the counter, frees the nonce", async () => {
    // Second right: nonce must equal the current rights_count (1).
    await program.methods
      .grantRight(1, RIGHT_KIND.USAGE, holder2.publicKey, 0, "")
      .accounts({
        parcel: parcelPda,
        ownership: ownershipPda,
        rights: rightsPda(1),
        owner,
        systemProgram: SystemProgram.programId,
      })
      .signers([payer])
      .rpc();
    let parcel = await accts.parcel.fetch(parcelPda);
    assert.strictEqual(parcel.rightsCount, 2);

    // Revoke the most recent right (counter tracks the next free nonce).
    await program.methods
      .revokeRight(1)
      .accounts({
        parcel: parcelPda,
        ownership: ownershipPda,
        rights: rightsPda(1),
        owner,
        systemProgram: SystemProgram.programId,
      })
      .signers([payer])
      .rpc();
    parcel = await accts.parcel.fetch(parcelPda);
    assert.strictEqual(parcel.rightsCount, 1);
    assert.isNull(await accts.rights.fetchNullable(rightsPda(1)));

    // Revoke the last one and prove the freed nonce is reusable.
    await program.methods
      .revokeRight(0)
      .accounts({
        parcel: parcelPda,
        ownership: ownershipPda,
        rights: rightsPda(0),
        owner,
        systemProgram: SystemProgram.programId,
      })
      .signers([payer])
      .rpc();
    parcel = await accts.parcel.fetch(parcelPda);
    assert.strictEqual(parcel.rightsCount, 0);
    assert.isNull(await accts.rights.fetchNullable(rightsPda(0)));

    await program.methods
      .grantRight(0, RIGHT_KIND.USAGE, holder.publicKey, 0, "")
      .accounts({
        parcel: parcelPda,
        ownership: ownershipPda,
        rights: rightsPda(0),
        owner,
        systemProgram: SystemProgram.programId,
      })
      .signers([payer])
      .rpc();
    parcel = await accts.parcel.fetch(parcelPda);
    assert.strictEqual(parcel.rightsCount, 1);

    // The canonical ownership right is not revocable via revoke_right.
    const ownership = await accts.rights.fetch(ownershipPda);
    assert.strictEqual(ownership.rightsKind, RIGHT_KIND.OWNERSHIP);
    assert.strictEqual(ownership.holder.toBase58(), owner.toBase58());
  });

  it("update_status enforces holder authorization and status bounds", async () => {
    // A valid signer that is not the holder cannot mutate the parcel.
    await expectRejected(
      "non-holder update_status",
      program.methods
        .updateStatus(PARCEL_STATUS.REGISTERED)
        .accounts({
          parcel: parcelPda,
          ownership: ownershipPda,
          owner: stranger.publicKey,
        })
        .signers([stranger])
        .rpc()
    );

    // Holder signs, but the status is out of range.
    await expectRejected(
      "out-of-range status",
      program.methods
        .updateStatus(200)
        .accounts({ parcel: parcelPda, ownership: ownershipPda, owner })
        .signers([payer])
        .rpc()
    );
  });

  it("updates infrastructure with a canonical access hash", async () => {
    const flags = 1 << 5; // ROAD_ACCESS
    const accessHash = Uint8Array.from({ length: 32 }, (_, i) => i + 7);

    await program.methods
      .updateInfrastructure(flags, accessHash)
      .accounts({ parcel: parcelPda, ownership: ownershipPda, owner })
      .signers([payer])
      .rpc();

    const parcel = await accts.parcel.fetch(parcelPda);
    assert.strictEqual(parcel.infrastructureFlags, flags);
    assert.deepStrictEqual(Array.from(parcel.accessHash), Array.from(accessHash));
  });

  it("rejects a zero access hash", async () => {
    const zeroHash = new Uint8Array(32);
    await expectRejected(
      "zero access hash",
      program.methods
        .updateInfrastructure(0, zeroHash)
        .accounts({ parcel: parcelPda, ownership: ownershipPda, owner })
        .signers([payer])
        .rpc()
    );
  });
});
