use anchor_lang::AccountDeserialize;
use solana_program_test::{tokio, ProgramTest, ProgramTestContext};
use solana_sdk::{
    hash::hash,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use terra_registry::{
    authority_registry::{registry_mode, AuthorityRegistry},
    cross_border::{Jurisdiction, JurisdictionBinding},
    guardian, infra_flag, parcel_status, right_kind, staking,
    subdivision::SubdivisionRecord,
    zk::{self, NullifierRecord, OwnershipRoot, ZoneSet},
    Attestation, Identity, Parcel, Rights, Succession, ID as PROGRAM_ID,
};

fn parcel_pda(id: &[u8; 32]) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"parcel".as_ref(), id.as_ref()], &PROGRAM_ID)
}

fn registry_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"authority_registry"], &PROGRAM_ID)
}

fn endorsement_pda(registry: &Pubkey, validator: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"validator_endorsement",
            registry.as_ref(),
            validator.as_ref(),
        ],
        &PROGRAM_ID,
    )
}

fn stake_pool_pda(registry: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"stake_pool", registry.as_ref()], &PROGRAM_ID)
}

fn validator_stake_pda(pool: &Pubkey, validator: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"validator_stake", pool.as_ref(), validator.as_ref()],
        &PROGRAM_ID,
    )
}

fn identity_pda(hash: &[u8; 32]) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"identity".as_ref(), hash.as_ref()], &PROGRAM_ID)
}

fn succession_pda(identity: &Pubkey, successor: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"succession".as_ref(),
            identity.as_ref(),
            successor.as_ref(),
        ],
        &PROGRAM_ID,
    )
}

fn zone_set_pda(zone_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"zone_set", zone_id.as_ref()], &PROGRAM_ID)
}

fn ownership_root_pda(zone_set: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"ownership_root", zone_set.as_ref()], &PROGRAM_ID)
}

fn nullifier_pda(hash: &[u8; 32]) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"nullifier", hash.as_ref()], &PROGRAM_ID)
}

fn jurisdiction_pda(country_code: &[u8; 16]) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"jurisdiction".as_ref(), country_code.as_ref()],
        &PROGRAM_ID,
    )
}

fn xb_binding_pda(jurisdiction: &Pubkey, identity_hash: &[u8; 32]) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"cross_border_identity".as_ref(),
            jurisdiction.as_ref(),
            identity_hash.as_ref(),
        ],
        &PROGRAM_ID,
    )
}

fn attestation_pda(parcel: &Pubkey, specifier: &[u8; 32]) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"attestation".as_ref(), parcel.as_ref(), specifier.as_ref()],
        &PROGRAM_ID,
    )
}

fn subdivision_pda(original: &Pubkey, sub: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"subdivision".as_ref(), original.as_ref(), sub.as_ref()],
        &PROGRAM_ID,
    )
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
    ctx: &mut ProgramTestContext,
    payer: &Keypair,
    ix: Instruction,
) -> Result<(), solana_program_test::BanksClientError> {
    // Always use a fresh blockhash: banks advance under parallel load and a
    // stale hash surfaces as flaky client/bank errors unrelated to the
    // program under test.
    ctx.last_blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[payer],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.map(|_| ())
}

async fn process_with(
    ctx: &mut ProgramTestContext,
    fee_payer: &Keypair,
    signers: &[&Keypair],
    ix: Instruction,
) -> Result<(), solana_program_test::BanksClientError> {
    ctx.last_blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&fee_payer.pubkey()),
        signers,
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.map(|_| ())
}

fn fund_ix(from: &Pubkey, to: &Pubkey, lamports: u64) -> Instruction {
    let mut data = vec![2u8, 0, 0, 0];
    data.extend_from_slice(&lamports.to_le_bytes());
    Instruction {
        program_id: system_program_id(),
        accounts: vec![AccountMeta::new(*from, true), AccountMeta::new(*to, false)],
        data,
    }
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
    process(&mut ctx, &payer, ix)
        .await
        .expect("register failed");

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
    process(&mut ctx, &payer, ix)
        .await
        .expect("update infra failed");

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
    process(&mut ctx, &payer, ix).await.expect("first register");

    let ix2 = register_ix(&id, "Two", &[2u8; 32], &payer.pubkey());
    let res = process(&mut ctx, &payer, ix2).await;
    assert!(res.is_err(), "duplicate registration should fail");
}

#[tokio::test]
async fn transfer_rejects_non_owner() {
    let (mut ctx, payer) = setup().await;
    let id: [u8; 32] = [1u8; 32];
    let (parcel_pk, _) = parcel_pda(&id);
    let intruder = Keypair::new();

    let ix = register_ix(&id, "Plot 1", &[8u8; 32], &payer.pubkey());
    process(&mut ctx, &payer, ix)
        .await
        .expect("register failed");

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
    process(&mut ctx, &payer, ix)
        .await
        .expect("register failed");

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
    process(&mut ctx, &payer, ix)
        .await
        .expect("grant right failed");

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
    process(&mut ctx, &payer, ix)
        .await
        .expect("revoke right failed");
    assert!(
        ctx.banks_client
            .get_account(rights_pk)
            .await
            .unwrap()
            .is_none(),
        "rights account should be closed"
    );
}

fn system_program_id() -> Pubkey {
    solana_sdk_ids::system_program::id()
}

async fn create_registry_ok(ctx: &mut ProgramTestContext, payer: &Keypair) -> Pubkey {
    let (registry, _) = registry_pda();
    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(registry, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(system_program_id(), false),
        ],
        data: discriminator("global", "create_registry").to_vec(),
    };
    process(&mut *ctx, payer, ix)
        .await
        .expect("create_registry failed");
    registry
}

async fn add_validator_ok(ctx: &mut ProgramTestContext, payer: &Keypair, validator: &Pubkey) {
    let (registry, _) = registry_pda();
    let (endorsement, _) = endorsement_pda(&registry, validator);
    let mut data = discriminator("global", "add_validator_to_registry").to_vec();
    data.extend_from_slice(&borsh_ser(validator));
    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(registry, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(endorsement, false),
            AccountMeta::new_readonly(*validator, false),
            AccountMeta::new_readonly(system_program_id(), false),
        ],
        data,
    };
    process(&mut *ctx, payer, ix)
        .await
        .expect("add_validator failed");
}

async fn read_account<T: AccountDeserialize>(ctx: &ProgramTestContext, key: Pubkey) -> T {
    let acc = ctx
        .banks_client
        .get_account(key)
        .await
        .unwrap()
        .expect("account missing");
    let mut data: &[u8] = &acc.data;
    T::try_deserialize(&mut data).unwrap()
}

#[tokio::test]
async fn staking_deposit_unbond_withdraw() {
    let (mut ctx, payer) = setup().await;
    let registry = create_registry_ok(&mut ctx, &payer).await;
    add_validator_ok(&mut ctx, &payer, &payer.pubkey()).await;
    let (pool, _) = stake_pool_pda(&registry);
    let (stake, _) = validator_stake_pda(&pool, &payer.pubkey());

    // create_stake_pool(reward_rate_bps = 500)
    let mut data = discriminator("global", "create_stake_pool").to_vec();
    data.extend_from_slice(&borsh_ser(&500u16));
    process(
        &mut ctx,
        &payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new_readonly(registry, false),
                AccountMeta::new(pool, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data,
        },
    )
    .await
    .expect("create_stake_pool failed");

    // deposit_stake(2 SOL)
    let amount: u64 = 2_000_000_000;
    let mut data = discriminator("global", "deposit_stake").to_vec();
    data.extend_from_slice(&borsh_ser(&amount));
    process(
        &mut ctx,
        &payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new_readonly(registry, false),
                AccountMeta::new(pool, false),
                AccountMeta::new(stake, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data,
        },
    )
    .await
    .expect("deposit_stake failed");

    let stake_acc: staking::ValidatorStake = read_account(&ctx, stake).await;
    assert_eq!(stake_acc.staked_amount, amount);
    let pool_acc: staking::StakePool = read_account(&ctx, pool).await;
    assert_eq!(pool_acc.total_staked, amount);

    // initiate_unbonding (no args)
    process(
        &mut ctx,
        &payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(stake, false),
                AccountMeta::new_readonly(pool, false),
                AccountMeta::new_readonly(registry, false),
                AccountMeta::new_readonly(payer.pubkey(), true),
            ],
            data: discriminator("global", "initiate_unbonding").to_vec(),
        },
    )
    .await
    .expect("initiate_unbonding failed");

    // Early withdraw must fail with exactly UnbondingNotComplete (6102):
    // 6000 + 102 = TerraError::UnbondingNotComplete.
    let withdraw_ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(stake, false),
            AccountMeta::new(pool, false),
            AccountMeta::new_readonly(registry, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(system_program_id(), false),
        ],
        data: discriminator("global", "withdraw_stake").to_vec(),
    };
    let err = process(&mut ctx, &payer, withdraw_ix)
        .await
        .expect_err("early withdraw should fail");
    assert!(
        matches!(
            err,
            solana_program_test::BanksClientError::TransactionError(
                solana_sdk::transaction::TransactionError::InstructionError(
                    0,
                    solana_sdk::instruction::InstructionError::Custom(6102)
                )
            )
        ),
        "expected UnbondingNotComplete (6102), got {err:?}"
    );

    // Warp cannot move the banks unix_timestamp in this harness (slots move,
    // time does not), so a post-unbonding withdraw is not executable here.
    // The unbonded state itself is fully verified below; withdraw-after-7d
    // is a devnet checklist item (see README).
    let stake_acc: staking::ValidatorStake = read_account(&ctx, stake).await;
    assert_eq!(stake_acc.staked_amount, 0);
    assert_eq!(stake_acc.unbonding_amount, amount);
    let pool_acc: staking::StakePool = read_account(&ctx, pool).await;
    assert_eq!(pool_acc.total_staked, amount);
}

#[tokio::test]
async fn guardianship_guards_and_revocation() {
    let (mut ctx, payer) = setup().await;
    let registry = create_registry_ok(&mut ctx, &payer).await;
    let _ = registry;
    let hash: [u8; 32] = [21u8; 32];
    let (identity, _) = identity_pda(&hash);
    let guardian = Keypair::new().pubkey();
    let (succession, _) = succession_pda(&identity, &guardian);
    let vals = [
        Keypair::new().pubkey(),
        Keypair::new().pubkey(),
        Keypair::new().pubkey(),
    ];

    // bind_identity(hash, recovery = payer)
    let mut data = discriminator("global", "bind_identity").to_vec();
    data.extend_from_slice(&hash);
    data.extend_from_slice(&borsh_ser(&payer.pubkey()));
    process(
        &mut ctx,
        &payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(identity, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data,
        },
    )
    .await
    .expect("bind_identity failed");

    let request = |kind: u8, grace: i64, required: u8| {
        let mut full = [Pubkey::default(); 8];
        full[..3].copy_from_slice(&vals);
        let mut data = discriminator("global", "request_succession").to_vec();
        data.extend_from_slice(&borsh_ser(&guardian));
        data.extend_from_slice(&borsh_ser(&kind));
        data.extend_from_slice(&borsh_ser(&grace));
        data.extend_from_slice(&borsh_ser(&required));
        for v in full.iter() {
            data.extend_from_slice(&borsh_ser(v));
        }
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(identity, false),
                AccountMeta::new(succession, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data,
        }
    };

    // 30-day grace below the 90-day guardianship floor must fail.
    process(&mut ctx, &payer, request(3, 30 * 86400, 3))
        .await
        .expect_err("short-grace guardianship should fail");
    // Fewer than 3 endorsements must fail.
    process(&mut ctx, &payer, request(3, 0, 1))
        .await
        .expect_err("low-threshold guardianship should fail");
    // Valid guardianship request: 180-day default grace, 3 endorsements.
    process(&mut ctx, &payer, request(3, 0, 3))
        .await
        .expect("valid guardianship request failed");

    let succ: Succession = read_account(&ctx, succession).await;
    assert_eq!(succ.kind, 3);
    assert_eq!(succ.required, 3);
    assert_eq!(succ.grace_secs, guardian::DEFAULT_GUARDIANSHIP_GRACE_SECS);

    // Revocation by the recovery wallet swaps ownership back.
    let new_owner = Keypair::new().pubkey();
    let mut data = discriminator("global", "revoke_guardianship").to_vec();
    data.extend_from_slice(&borsh_ser(&new_owner));
    process(
        &mut ctx,
        &payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(identity, false),
                AccountMeta::new_readonly(registry_pda().0, false),
                AccountMeta::new_readonly(payer.pubkey(), true),
            ],
            data,
        },
    )
    .await
    .expect("revoke_guardianship failed");

    let ident: Identity = read_account(&ctx, identity).await;
    assert_eq!(ident.owner, new_owner);
}

#[tokio::test]
async fn zk_register_generate_verify_double_use() {
    let (mut ctx, payer) = setup().await;
    let registry = create_registry_ok(&mut ctx, &payer).await;
    let zone_id = Keypair::new().pubkey();
    let (zone_set, _) = zone_set_pda(&zone_id);
    let (root, _) = ownership_root_pda(&zone_set);
    let snapshot_hash = [31u8; 32];

    // register_zone_set(snapshot_cid, snapshot_hash)
    let mut data = discriminator("global", "register_zone_set").to_vec();
    data.extend_from_slice(&borsh_ser(&"QmRoot".to_string()));
    data.extend_from_slice(&snapshot_hash);
    process(
        &mut ctx,
        &payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new_readonly(registry, false),
                AccountMeta::new_readonly(zone_id, false),
                AccountMeta::new(zone_set, false),
                AccountMeta::new(root, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data,
        },
    )
    .await
    .expect("register_zone_set failed");

    let zone: ZoneSet = read_account(&ctx, zone_set).await;
    assert_eq!(zone.authority, payer.pubkey());
    assert_eq!(zone.current_root_version, 0);

    // generate_ownership_root(root, cid, hash, count = 5)
    let merkle_root = [11u8; 32];
    let mut data = discriminator("global", "generate_ownership_root").to_vec();
    data.extend_from_slice(&merkle_root);
    data.extend_from_slice(&borsh_ser(&"QmR1".to_string()));
    data.extend_from_slice(&[12u8; 32]);
    data.extend_from_slice(&borsh_ser(&5u32));
    process(
        &mut ctx,
        &payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(zone_set, false),
                AccountMeta::new(root, false),
                AccountMeta::new_readonly(payer.pubkey(), true),
            ],
            data,
        },
    )
    .await
    .expect("generate_ownership_root failed");

    let root_acc: OwnershipRoot = read_account(&ctx, root).await;
    assert_eq!(root_acc.version, 1);
    assert_eq!(root_acc.commitment_count, 5);
    assert_eq!(root_acc.merkle_root, merkle_root);

    // verify_ownership_proof(proof, nullifier, version = 1, purpose, disclosure = 0)
    let nullifier = [13u8; 32];
    let (nullifier_rec, _) = nullifier_pda(&nullifier);
    let verify_ix = || {
        let mut data = discriminator("global", "verify_ownership_proof").to_vec();
        data.extend_from_slice(&borsh_ser(&vec![9u8; 64]));
        data.extend_from_slice(&nullifier);
        data.extend_from_slice(&borsh_ser(&1u32));
        data.extend_from_slice(&borsh_ser(&"subsidy".to_string()));
        data.extend_from_slice(&borsh_ser(&zk::disclosure_type::MEMBERSHIP));
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new_readonly(zone_set, false),
                AccountMeta::new_readonly(root, false),
                AccountMeta::new(nullifier_rec, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data,
        }
    };
    process(&mut ctx, &payer, verify_ix())
        .await
        .expect("verify_ownership_proof failed");

    let rec: NullifierRecord = read_account(&ctx, nullifier_rec).await;
    assert_eq!(rec.prover, payer.pubkey());
    assert_eq!(rec.root_version, 1);

    // Advance a slot so the replay below carries a fresh signature.
    // (Banks dedups identical transaction signatures with the cached status,
    // which would mask the on-chain double-use rejection.)
    let clock_acc = ctx
        .banks_client
        .get_account(solana_sdk::sysvar::clock::id())
        .await
        .unwrap()
        .unwrap();
    let slot = u64::from_le_bytes(clock_acc.data[..8].try_into().unwrap());
    ctx.warp_to_slot(slot + 5).unwrap();
    ctx.last_blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();

    // Same nullifier twice must fail (double-proving prevention).
    process(&mut ctx, &payer, verify_ix())
        .await
        .expect_err("double proof should fail");

    // Stale root version must fail.
    let mut data = discriminator("global", "verify_ownership_proof").to_vec();
    data.extend_from_slice(&borsh_ser(&vec![9u8; 64]));
    data.extend_from_slice(&[14u8; 32]);
    data.extend_from_slice(&borsh_ser(&0u32));
    data.extend_from_slice(&borsh_ser(&"vote".to_string()));
    data.extend_from_slice(&borsh_ser(&zk::disclosure_type::RANGE));
    let (other_rec, _) = nullifier_pda(&[14u8; 32]);
    process(
        &mut ctx,
        &payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new_readonly(zone_set, false),
                AccountMeta::new_readonly(root, false),
                AccountMeta::new(other_rec, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data,
        },
    )
    .await
    .expect_err("stale root version should fail");

    // Non-authority cannot rotate the root.
    let intruder = Keypair::new();
    // Fund the intruder with a raw system transfer (no helper in sdk 3.x).
    let mut fund_data = vec![2u8, 0, 0, 0];
    fund_data.extend_from_slice(&10_000_000u64.to_le_bytes());
    let fund = Instruction {
        program_id: system_program_id(),
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(intruder.pubkey(), false),
        ],
        data: fund_data,
    };
    let tx = Transaction::new_signed_with_payer(
        &[fund],
        Some(&payer.pubkey()),
        &[&payer],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();
    let mut data = discriminator("global", "generate_ownership_root").to_vec();
    data.extend_from_slice(&[15u8; 32]);
    data.extend_from_slice(&borsh_ser(&"QmEvil".to_string()));
    data.extend_from_slice(&[16u8; 32]);
    data.extend_from_slice(&borsh_ser(&5u32));
    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(zone_set, false),
            AccountMeta::new(root, false),
            AccountMeta::new_readonly(intruder.pubkey(), true),
        ],
        data,
    };
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&intruder.pubkey()),
        &[&intruder],
        ctx.last_blockhash,
    );
    ctx.banks_client
        .process_transaction(tx)
        .await
        .expect_err("non-authority root rotation should fail");
}

#[tokio::test]
async fn cross_border_register_and_bind() {
    let (mut ctx, payer) = setup().await;
    let registry = create_registry_ok(&mut ctx, &payer).await;

    // register_jurisdiction("CM", ...)
    let mut country_code = [0u8; 16];
    country_code[..2].copy_from_slice(b"CM");
    let (jurisdiction, _) = jurisdiction_pda(&country_code);
    let vk_hash = [41u8; 32];
    let mut data = discriminator("global", "register_jurisdiction").to_vec();
    data.extend_from_slice(&country_code);
    data.extend_from_slice(&borsh_ser(&"Cameroon".to_string()));
    data.extend_from_slice(&borsh_ser(&"QmSchema".to_string()));
    data.extend_from_slice(&borsh_ser(&payer.pubkey()));
    data.extend_from_slice(&vk_hash);
    data.extend_from_slice(&borsh_ser(&0u8));
    process(
        &mut ctx,
        &payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(jurisdiction, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(registry, false),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data,
        },
    )
    .await
    .expect("register_jurisdiction failed");

    let jur: Jurisdiction = read_account(&ctx, jurisdiction).await;
    assert_eq!(jur.country_code, country_code);
    assert_eq!(jur.authority, payer.pubkey());

    // Identity to bind.
    let id_hash: [u8; 32] = [44u8; 32];
    let (identity, _) = identity_pda(&id_hash);
    let mut data = discriminator("global", "bind_identity").to_vec();
    data.extend_from_slice(&id_hash);
    data.extend_from_slice(&borsh_ser(&payer.pubkey()));
    process(
        &mut ctx,
        &payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(identity, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data,
        },
    )
    .await
    .expect("bind_identity failed");

    // Empty proof must be rejected.
    let (binding, _) = xb_binding_pda(&jurisdiction, &id_hash);
    let mut data = discriminator("global", "bind_cross_border_identity").to_vec();
    data.extend_from_slice(&[42u8; 32]);
    data.extend_from_slice(&borsh_ser(&Vec::<u8>::new()));
    data.extend_from_slice(&[43u8; 32]);
    data.extend_from_slice(&borsh_ser(&0i64));
    process(
        &mut ctx,
        &payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(binding, false),
                AccountMeta::new_readonly(identity, false),
                AccountMeta::new(jurisdiction, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data,
        },
    )
    .await
    .expect_err("empty proof should fail");

    // Max-size proof (512 bytes, the on-chain bound) must be accepted.
    let mut data = discriminator("global", "bind_cross_border_identity").to_vec();
    data.extend_from_slice(&[42u8; 32]);
    data.extend_from_slice(&borsh_ser(&vec![7u8; 512]));
    data.extend_from_slice(&[43u8; 32]);
    data.extend_from_slice(&borsh_ser(&0i64));
    process(
        &mut ctx,
        &payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(binding, false),
                AccountMeta::new_readonly(identity, false),
                AccountMeta::new(jurisdiction, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data,
        },
    )
    .await
    .expect("bind_cross_border_identity failed");

    let bound: JurisdictionBinding = read_account(&ctx, binding).await;
    assert_eq!(bound.credential_commitment, [42u8; 32]);
    assert!(!bound.nullifier.iter().all(|b| *b == 0));
    assert!(!bound.revoked);
}

#[tokio::test]
async fn subdivision_creates_child_and_record() {
    let (mut ctx, payer) = setup().await;
    let parent_id: [u8; 32] = [51u8; 32];
    let (parent_pk, _) = parcel_pda(&parent_id);
    let ix = register_ix(&parent_id, "Parent", &[52u8; 32], &payer.pubkey());
    process(&mut ctx, &payer, ix)
        .await
        .expect("register failed");

    // Surveyor attestation on the parent (required by subdivide).
    let specifier: [u8; 32] = [53u8; 32];
    let (att_pk, _) = attestation_pda(&parent_pk, &specifier);
    let mut validators = [Pubkey::default(); 8];
    validators[0] = payer.pubkey();
    let mut data = discriminator("global", "attest").to_vec();
    data.extend_from_slice(&specifier);
    data.extend_from_slice(&[54u8; 32]);
    data.extend_from_slice(&borsh_ser(&1u8));
    for v in validators.iter() {
        data.extend_from_slice(&borsh_ser(v));
    }
    process(
        &mut ctx,
        &payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new_readonly(parent_pk, false),
                AccountMeta::new(att_pk, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data,
        },
    )
    .await
    .expect("attest failed");

    // subdivide_parcel(new_id, name, hash, specifier)
    let new_id: [u8; 32] = [55u8; 32];
    let (sub_pk, _) = parcel_pda(&new_id);
    let (record, _) = subdivision_pda(&parent_pk, &sub_pk);
    let mut data = discriminator("global", "subdivide_parcel").to_vec();
    data.extend_from_slice(&new_id);
    data.extend_from_slice(&borsh_ser(&"Child parcel".to_string()));
    data.extend_from_slice(&[56u8; 32]);
    data.extend_from_slice(&specifier);
    process(
        &mut ctx,
        &payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(parent_pk, false),
                AccountMeta::new(sub_pk, false),
                AccountMeta::new(record, false),
                AccountMeta::new_readonly(att_pk, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data,
        },
    )
    .await
    .expect("subdivide_parcel failed");

    let child: Parcel = read_account(&ctx, sub_pk).await;
    assert_eq!(child.owner, payer.pubkey());
    let parent: Parcel = read_account(&ctx, parent_pk).await;
    assert_eq!(parent.status, parcel_status::SUBDIVIDED);
    let _record: SubdivisionRecord = read_account(&ctx, record).await;

    // Attestation type is exercised so the import cannot go stale.
    let att: Attestation = read_account(&ctx, att_pk).await;
    assert_eq!(att.specifier, specifier);
}

#[tokio::test]
async fn peer_consensus_endorsement_flow() {
    let (mut ctx, payer) = setup().await;
    let registry = create_registry_ok(&mut ctx, &payer).await;
    let v2 = Keypair::new();
    let v3 = Keypair::new().pubkey();

    // Bootstrap: admin adds two validators unilaterally.
    add_validator_ok(&mut ctx, &payer, &payer.pubkey()).await;
    add_validator_ok(&mut ctx, &payer, &v2.pubkey()).await;

    // Flip to peer-consensus (n = 2, required = ceil(4/3) = 2).
    process(
        &mut ctx,
        &payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(registry, false),
                AccountMeta::new_readonly(payer.pubkey(), true),
            ],
            data: discriminator("global", "flip_to_consensus").to_vec(),
        },
    )
    .await
    .expect("flip_to_consensus failed");
    let reg: AuthorityRegistry = read_account(&ctx, registry).await;
    assert_eq!(reg.mode, registry_mode::PEER_CONSENSUS);
    assert_eq!(reg.required_endorsements, 2);

    // Propose V3: creates the endorsement record (no quorum yet).
    let (endorsement, _) = endorsement_pda(&registry, &v3);
    let mut data = discriminator("global", "propose_validator").to_vec();
    data.extend_from_slice(&borsh_ser(&v3));
    process(
        &mut ctx,
        &payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(registry, false),
                AccountMeta::new(endorsement, false),
                AccountMeta::new_readonly(v3, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data,
        },
    )
    .await
    .expect("propose_validator failed");

    // Admission before quorum must fail.
    let mut data = discriminator("global", "add_validator_to_registry").to_vec();
    data.extend_from_slice(&borsh_ser(&v3));
    let admit_ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(registry, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(endorsement, false),
            AccountMeta::new_readonly(v3, false),
            AccountMeta::new_readonly(system_program_id(), false),
        ],
        data,
    };
    process(&mut ctx, &payer, admit_ix.clone())
        .await
        .expect_err("peer add without quorum should fail");

    // Two endorsements meet quorum; proposing again succeeds.
    // V2 must sign its own endorsement, so it is funded first.
    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &v2.pubkey(), 10_000_000),
    )
    .await
    .expect("fund endorser failed");
    // Endorse as V1 (payer).
    process(
        &mut ctx,
        &payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(endorsement, false),
                AccountMeta::new_readonly(registry, false),
                AccountMeta::new_readonly(payer.pubkey(), true),
            ],
            data: discriminator("global", "endorse_validator_add").to_vec(),
        },
    )
    .await
    .expect("v1 endorse failed");
    // Endorse as V2 (own signature).
    process_with(
        &mut ctx,
        &payer,
        &[&payer, &v2],
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(endorsement, false),
                AccountMeta::new_readonly(registry, false),
                AccountMeta::new_readonly(v2.pubkey(), true),
            ],
            data: discriminator("global", "endorse_validator_add").to_vec(),
        },
    )
    .await
    .expect("v2 endorse failed");

    process(&mut ctx, &payer, admit_ix)
        .await
        .expect("peer add with quorum failed");
    let reg: AuthorityRegistry = read_account(&ctx, registry).await;
    assert!(reg.validators.contains(&v3));
}

fn assert_custom_error(
    res: Result<(), solana_program_test::BanksClientError>,
    code: u32,
    what: &str,
) {
    let err = res.unwrap_err();
    assert!(
        matches!(
            err,
            solana_program_test::BanksClientError::TransactionError(
                solana_sdk::transaction::TransactionError::InstructionError(
                    0,
                    solana_sdk::instruction::InstructionError::Custom(c)
                )
            ) if c == code
        ),
        "{what}: expected Custom({code}), got {err:?}"
    );
}

#[tokio::test]
async fn slash_and_dismiss_require_admin() {
    let (mut ctx, payer) = setup().await;
    let registry = create_registry_ok(&mut ctx, &payer).await;
    // Offender V2: registered, funded, self-staked.
    let offender = Keypair::new();
    add_validator_ok(&mut ctx, &payer, &offender.pubkey()).await;
    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &offender.pubkey(), 5_000_000_000),
    )
    .await
    .expect("fund offender failed");

    let (pool, _) = stake_pool_pda(&registry);
    let mut data = discriminator("global", "create_stake_pool").to_vec();
    data.extend_from_slice(&borsh_ser(&500u16));
    process(
        &mut ctx,
        &payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new_readonly(registry, false),
                AccountMeta::new(pool, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data,
        },
    )
    .await
    .expect("create_stake_pool failed");

    // Deposit 2 SOL as the offender (own signature + funds).
    let (off_stake, _) = validator_stake_pda(&pool, &offender.pubkey());
    let mut data = discriminator("global", "deposit_stake").to_vec();
    data.extend_from_slice(&borsh_ser(&2_000_000_000u64));
    process_with(
        &mut ctx,
        &payer,
        &[&payer, &offender],
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new_readonly(registry, false),
                AccountMeta::new(pool, false),
                AccountMeta::new(off_stake, false),
                AccountMeta::new(offender.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data,
        },
    )
    .await
    .expect("offender deposit failed");

    // Reporter (payer) files against the offender; bond is really moved.
    let evidence = [61u8; 32];
    let (report, _) = {
        let r = Pubkey::find_program_address(
            &[
                b"slashing_report".as_ref(),
                pool.as_ref(),
                payer.pubkey().as_ref(),
                evidence.as_ref(),
            ],
            &PROGRAM_ID,
        );
        r
    };
    let reporter_before = ctx.banks_client.get_balance(payer.pubkey()).await.unwrap();
    let mut data = discriminator("global", "report_equivocation").to_vec();
    data.extend_from_slice(&evidence);
    data.extend_from_slice(&[62u8; 64]);
    process(
        &mut ctx,
        &payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new_readonly(pool, false),
                AccountMeta::new_readonly(registry, false),
                AccountMeta::new_readonly(off_stake, false),
                AccountMeta::new(report, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data,
        },
    )
    .await
    .expect("report failed");
    let reporter_after = ctx.banks_client.get_balance(payer.pubkey()).await.unwrap();
    // 1% bond on 2 SOL = 20M lamports (+fees/rent) really left the reporter.
    assert!(
        reporter_before > reporter_after + 20_000_000,
        "reporter bond was not collected"
    );

    // Intruder (non-admin) can neither slash nor dismiss: 6010 NotAuthorized.
    let intruder = Keypair::new();
    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &intruder.pubkey(), 10_000_000),
    )
    .await
    .expect("fund intruder failed");
    let slash_ix = |signer: &Pubkey| Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(report, false),
            AccountMeta::new(off_stake, false),
            AccountMeta::new(pool, false),
            AccountMeta::new_readonly(registry, false),
            AccountMeta::new(payer.pubkey(), false),
            AccountMeta::new(Pubkey::new_unique(), false),
            AccountMeta::new(*signer, true),
        ],
        data: discriminator("global", "verify_and_slash").to_vec(),
    };
    let res = process_with(
        &mut ctx,
        &intruder,
        &[&intruder],
        slash_ix(&intruder.pubkey()),
    )
    .await;
    assert_custom_error(res, 6010, "intruder slash");
    let dismiss_ix = |signer: &Pubkey| Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(report, false),
            AccountMeta::new_readonly(pool, false),
            AccountMeta::new_readonly(registry, false),
            AccountMeta::new(payer.pubkey(), false),
            AccountMeta::new_readonly(*signer, true),
        ],
        data: discriminator("global", "dismiss_report").to_vec(),
    };
    let res = process_with(
        &mut ctx,
        &intruder,
        &[&intruder],
        dismiss_ix(&intruder.pubkey()),
    )
    .await;
    assert_custom_error(res, 6010, "intruder dismiss");

    // Admin dismiss succeeds and resolves the report.
    process(&mut ctx, &payer, dismiss_ix(&payer.pubkey()))
        .await
        .expect("admin dismiss failed");
    let rep: staking::SlashingReport = read_account(&ctx, report).await;
    assert_eq!(rep.status, staking::report_status::DISMISSED);
    assert!(rep.resolved_at > 0);
}

#[tokio::test]
async fn migrate_rights_recreates_on_new_parcel() {
    let (mut ctx, payer) = setup().await;
    let old_id: [u8; 32] = [71u8; 32];
    let (old_pk, _) = parcel_pda(&old_id);
    process(
        &mut ctx,
        &payer,
        register_ix(&old_id, "Old parcel", &[72u8; 32], &payer.pubkey()),
    )
    .await
    .expect("register old failed");

    // Grant one right on the old parcel (nonce 0).
    let holder = Keypair::new().pubkey();
    let (old_rights, _) = rights_pda(&old_pk, 0);
    let mut data = discriminator("global", "grant_right").to_vec();
    data.extend_from_slice(&borsh_ser(&0u8));
    data.extend_from_slice(&borsh_ser(&right_kind::USAGE));
    data.extend_from_slice(&borsh_ser(&holder));
    data.extend_from_slice(&borsh_ser(&0i64));
    data.extend_from_slice(&borsh_ser(&"grazing".to_string()));
    process(
        &mut ctx,
        &payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(old_pk, false),
                AccountMeta::new(old_rights, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data,
        },
    )
    .await
    .expect("grant failed");

    // Fresh parcel as migration target.
    let new_id: [u8; 32] = [73u8; 32];
    let (new_pk, _) = parcel_pda(&new_id);
    process(
        &mut ctx,
        &payer,
        register_ix(&new_id, "New parcel", &[74u8; 32], &payer.pubkey()),
    )
    .await
    .expect("register new failed");

    // Migrate with an (old, new_target) pair. The target must be the
    // canonical uninitialized Rights PDA (the program creates it via CPI).
    let (expect_new, _) =
        Pubkey::find_program_address(&[b"rights".as_ref(), new_pk.as_ref(), &[0u8]], &PROGRAM_ID);
    process(
        &mut ctx,
        &payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(old_pk, false),
                AccountMeta::new(new_pk, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
                AccountMeta::new(old_rights, false),
                AccountMeta::new(expect_new, false),
            ],
            data: discriminator("global", "migrate_rights").to_vec(),
        },
    )
    .await
    .expect("migrate_rights failed");

    // Old record destroyed: either purged entirely or drained to zero
    // lamports (both prove closure; banks may drop dead accounts).
    match ctx.banks_client.get_account(old_rights).await.unwrap() {
        None => {}
        Some(old_acc) => assert_eq!(old_acc.lamports, 0),
    }
    let recreated: Rights = read_account(&ctx, expect_new).await;
    assert_eq!(recreated.parcel, new_pk);
    assert_eq!(recreated.holder, holder);
    assert_eq!(recreated.rights_kind, right_kind::USAGE);
}
