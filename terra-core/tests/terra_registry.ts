import * as anchor from "@coral-xyz/anchor";
import { Program, Idl } from "@coral-xyz/anchor";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { Keypair, PublicKey, Connection } from "@solana/web3.js";
import { assert } from "chai";
import * as fs from "fs";
import * as path from "path";

const IDL_PATH = path.join(__dirname, "..", "target", "idl", "terra_registry.json");

describe("terra-registry", () => {
  const programId = new PublicKey("GaEDbktvpZ3qiqp4PmFgHwDSa6JsFfVjXFqNb2nTbage");
  const connection = new Connection("http://127.0.0.1:8899", "confirmed");

  const payer = Keypair.generate();
  const wallet = new NodeWallet(payer);
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });
  anchor.setProvider(provider);

  const idl = JSON.parse(fs.readFileSync(IDL_PATH, "utf-8")) as Idl;
  const program = new Program<TerraRegistry>(idl, programId, provider);

  const owner = wallet.publicKey;
  let newOwner = Keypair.generate();
  const holder = Keypair.generate();

  const parcelId = Uint8Array.from({ length: 32 }, (_, i) => i + 1);
  const geometryHash = Uint8Array.from({ length: 32 }, (_, i) => i * 2);

  const [parcelPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("parcel"), Buffer.from(parcelId)],
    programId
  );

  before(async () => {
    // Fund the payer against the local validator.
    const airdropSig = await connection.requestAirdrop(
      payer.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(airdropSig, "confirmed");
  });

  it("registers a parcel", async () => {
    await program.methods
      .registerParcel(parcelId, "Soa/Biteng demo parcel", geometryHash)
      .accounts({ owner })
      .signers([payer])
      .rpc();

    const parcel = await program.account.parcel.fetch(parcelPda);
    assert.strictEqual(parcel.owner.toBase58(), owner.toBase58());
    assert.strictEqual(parcel.name, "Soa/Biteng demo parcel");
    assert.strictEqual(parcel.status, 1); // REGISTERED
  });

  it("rejects duplicate id (same PDA)", async () => {
    await assert.isRejected(
      program.methods
        .registerParcel(parcelId, "dupe", geometryHash)
        .accounts({ owner })
        .signers([payer])
        .rpc()
    );
  });

  it("transfers the parcel", async () => {
    await program.methods
      .transferParcel()
      .accounts({ parcel: parcelPda, owner, newOwner: newOwner.publicKey })
      .signers([payer])
      .rpc();

    const parcel = await program.account.parcel.fetch(parcelPda);
    assert.strictEqual(parcel.owner.toBase58(), newOwner.publicKey.toBase58());
  });

  it("grants and revokes a right", async () => {
    // Re-transfer back to the provider wallet so it can keep acting as owner.
    await program.methods
      .transferParcel()
      .accounts({
        parcel: parcelPda,
        owner: newOwner.publicKey,
        newOwner: owner,
      })
      .signers([newOwner, payer])
      .rpc();

    await program.methods
      .grantRight(0, 1, holder.publicKey, 0, "")
      .accounts({ parcel: parcelPda, owner })
      .signers([payer])
      .rpc();

    const parcel = await program.account.parcel.fetch(parcelPda);
    assert.strictEqual(parcel.rightsCount, 1);

    const [rightPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("rights"), parcelPda.toBuffer(), Buffer.from([0])],
      programId
    );
    const rights = await program.account.rights.fetch(rightPda);
    assert.strictEqual(rights.rightsKind, 1); // USAGE
    assert.strictEqual(rights.holder.toBase58(), holder.publicKey.toBase58());

    await program.methods
      .revokeRight(0)
      .accounts({ parcel: parcelPda, owner })
      .signers([payer])
      .rpc();

    await assert.isRejected(
      program.account.rights.fetch(rightPda),
      undefined,
      "closed rights account should not be readable"
    );
  });

  it("updates infrastructure with a canonical access hash", async () => {
    const flags = 1 << 5; // ROAD_ACCESS
    const accessHash = Uint8Array.from({ length: 32 }, (_, i) => i + 7);

    await program.methods
      .updateInfrastructure(flags, accessHash)
      .accounts({ parcel: parcelPda, owner })
      .signers([payer])
      .rpc();

    const parcel = await program.account.parcel.fetch(parcelPda);
    assert.strictEqual(parcel.infrastructureFlags, flags);
    assert.deepStrictEqual(Array.from(parcel.accessHash), Array.from(accessHash));
  });

  it("rejects a zero access hash", async () => {
    const zeroHash = new Uint8Array(32);
    await assert.isRejected(
      program.methods
        .updateInfrastructure(0, zeroHash)
        .accounts({ parcel: parcelPda, owner })
        .signers([payer])
        .rpc()
    );
  });
});