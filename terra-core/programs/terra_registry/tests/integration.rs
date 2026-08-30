use anchor_lang::AccountDeserialize;
use solana_program_test::{tokio, ProgramTest, ProgramTestContext};
use solana_sdk::{
    hash::hash,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use terra_registry::{infra_flag, parcel_status, right_kind, ID as PROGRAM_ID, Parcel, Rights};

fn parcel_pda(id: &[u8; 32]) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"parcel".as_ref(), id.as_ref()], &PROGRAM_ID)
}

fn rights_pda(parcel: &Pubkey, nonce: u8) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"rights".as_ref(), parcel.as_ref(), &[nonce]],
        &PROGRAM_ID,
    )
}

fn discriminator(namespace: &str, name: &str) -> [u8; 8] {
    let result = hash(format!("{}:{}", namespace, name).as_bytes());
    let mut out = [0u8; 8];
    out.copy_from_slice(&result.to_bytes()[..8]);
    out
}

fn borsh_ser<T: borsh::BorshSerialize>(v: &T) -> Vec<u8> {
    let mut out = Vec::new();
    v.serialize(&mut out).unwrap();
    out
}

fn register_ix(id: &[u8; 32], name: &str, geo: &[u8; 32], payer: &Pubkey) -> Instruction {
    let mut data = discriminator("global", "register_parcel").to_vec();
    data.extend_from_slice(id);
    data.extend_from_slice(&borsh_ser(&name.to_string()));
    data.extend_from_slice(geo);
    let (parcel_pk, _) = parcel_pda(id);
    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(parcel_pk, false),
            AccountMeta::new(*payer, true),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::id(), false),
        ],
        data,
    }
}

async fn setup() -> (ProgramTestContext, Keypair) {
    let mut pt = ProgramTest::new("terra_registry", PROGRAM_ID, None);
    pt.add_program("terra_registry", PROGRAM_ID, None);
    pt.set_compute_max_units(500_000);
    let ctx = pt.start_with_context().await;
    let payer = ctx.payer.insecure_clone();
    (ctx, payer)
}

async fn process(
    ctx: &ProgramTestContext,
    payer: &Keypair,
    ix: Instruction,
) -> Result<(), solana_program_test::BanksClientError> {
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[payer],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.map(|_| ())
}

#[tokio::test]
async fn register_transfer_infrastructure() {
    let (mut ctx, payer) = setup().await;
    let id: [u8; 32] = [7u8; 32];
    let geo: [u8; 32] = [9u8; 32];
    let hash: [u8; 32] = [5u8; 32];
    let (parcel_pk, _) = parcel_pda(&id);

    // register
    let mut data = discriminator("global", "register_parcel").to_vec();
    data.extend_from_slice(&id);
    data.extend_from_slice(&borsh_ser(&"Plot 7".to_string()));
    data.extend_from_slice(&geo);
    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(parcel_pk, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::id(), false),
        ],
        data,
    };
    process(&ctx, &payer, ix).await.expect("register failed");

    let parcel = ctx
        .banks_client
        .get_account(parcel_pk)
        .await
        .unwrap()
        .expect("parcel account missing");
    let mut data: &[u8] = &parcel.data;
    let decoded = Parcel::try_deserialize(&mut data).unwrap();
    assert_eq!(decoded.owner, payer.pubkey());
    assert_eq!(decoded.status, parcel_status::REGISTERED);
    assert_eq!(decoded.rights_count, 0);

    // update_infrastructure
    let mut data = discriminator("global", "update_infrastructure").to_vec();
    data.extend_from_slice(&borsh_ser(&infra_flag::ALL));
    data.extend_from_slice(&hash);
    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(parcel_pk, false),
            AccountMeta::new(payer.pubkey(), true),
        ],
        data,
    };
    process(&ctx, &payer, ix).await.expect("update infra failed");

    let parcel = ctx
        .banks_client
        .get_account(parcel_pk)
        .await
        .unwrap()
        .unwrap();
    let mut data: &[u8] = &parcel.data;
    let decoded = Parcel::try_deserialize(&mut data).unwrap();
    assert_eq!(decoded.infrastructure_flags, infra_flag::ALL);
    assert_eq!(decoded.access_hash, hash);
}

#[tokio::test]
async fn duplicate_parcel_rejected() {
    let (mut ctx, payer) = setup().await;
    let id: [u8; 32] = [3u8; 32];
    let ix = register_ix(&id, "One", &[1u8; 32], &payer.pubkey());
    process(&ctx, &payer, ix).await.expect("first register");

    let ix2 = register_ix(&id, "Two", &[2u8; 32], &payer.pubkey());
    let res = process(&ctx, &payer, ix2).await;
    assert!(res.is_err(), "duplicate registration should fail");
}

#[tokio::test]
async fn transfer_rejects_non_owner() {
    let (mut ctx, payer) = setup().await;
    let id: [u8; 32] = [1u8; 32];
    let (parcel_pk, _) = parcel_pda(&id);
    let intruder = Keypair::new();

    let ix = register_ix(&id, "Plot 1", &[8u8; 32], &payer.pubkey());
    process(&ctx, &payer, ix).await.expect("register failed");

    let data = discriminator("global", "transfer_parcel").to_vec();
    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(parcel_pk, false),
            AccountMeta::new(intruder.pubkey(), true),
            AccountMeta::new_readonly(intruder.pubkey(), false),
        ],
        data,
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&intruder.pubkey()),
        &[&intruder],
        ctx.last_blockhash,
    );
    let res = ctx.banks_client.process_transaction(tx).await;
    assert!(res.is_err(), "non-owner transfer should fail");
}

#[tokio::test]
async fn rights_lifecycle() {
    let (mut ctx, payer) = setup().await;
    let id: [u8; 32] = [2u8; 32];
    let (parcel_pk, _) = parcel_pda(&id);
    let holder = Keypair::new();
    let nonce: u8 = 0;
    let (rights_pk, _) = rights_pda(&parcel_pk, nonce);

    let ix = register_ix(&id, "Rights plot", &[4u8; 32], &payer.pubkey());
    process(&ctx, &payer, ix).await.expect("register failed");

    // grant_right
    let mut data = discriminator("global", "grant_right").to_vec();
    data.extend_from_slice(&borsh_ser(&nonce));
    data.extend_from_slice(&borsh_ser(&right_kind::USAGE));
    data.extend_from_slice(&borsh_ser(&holder.pubkey()));
    data.extend_from_slice(&borsh_ser(&0i64));
    data.extend_from_slice(&borsh_ser(&"grazing".to_string()));
    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(parcel_pk, false),
            AccountMeta::new(rights_pk, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::id(), false),
        ],
        data,
    };
    process(&ctx, &payer, ix).await.expect("grant right failed");

    let acc = ctx
        .banks_client
        .get_account(rights_pk)
        .await
        .unwrap()
        .expect("rights account missing");
    let mut data: &[u8] = &acc.data;
    let decoded = Rights::try_deserialize(&mut data).unwrap();
    assert_eq!(decoded.rights_kind, right_kind::USAGE);
    assert_eq!(decoded.holder, holder.pubkey());
    assert_eq!(decoded.granter, payer.pubkey());

    // revoke_right
    let mut data = discriminator("global", "revoke_right").to_vec();
    data.extend_from_slice(&borsh_ser(&nonce));
    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(parcel_pk, false),
            AccountMeta::new(rights_pk, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::id(), false),
        ],
        data,
    };
    process(&ctx, &payer, ix).await.expect("revoke right failed");
    assert!(
        ctx.banks_client
            .get_account(rights_pk)
            .await
            .unwrap()
            .is_none(),
        "rights account should be closed"
    );
}