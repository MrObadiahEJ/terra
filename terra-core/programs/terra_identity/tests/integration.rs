use anchor_lang::AccountDeserialize;
use solana_program_test::{tokio, ProgramTest, ProgramTestContext};
use solana_sdk::{
    hash::hash,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use terra_identity::{
    constants::*,
    helpers::*,
    state::{Identity, Succession},
    ID as PROGRAM_ID,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn identity_pda(identity_hash: &[u8; 32]) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"identity".as_ref(), identity_hash.as_ref()], &PROGRAM_ID)
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

fn system_program_id() -> Pubkey {
    solana_sdk_ids::system_program::id()
}

async fn setup() -> (ProgramTestContext, Keypair) {
    let mut pt = ProgramTest::new("terra_identity", PROGRAM_ID, None);
    pt.set_compute_max_units(200_000);
    let ctx = pt.start_with_context().await;
    let payer = ctx.payer.insecure_clone();
    (ctx, payer)
}

async fn process(
    ctx: &mut ProgramTestContext,
    payer: &Keypair,
    ix: Instruction,
) -> Result<(), solana_program_test::BanksClientError> {
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
    let mut all = vec![fee_payer];
    all.extend(signers);
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&fee_payer.pubkey()),
        &all,
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

// ---------------------------------------------------------------------------
// Instruction builders
// ---------------------------------------------------------------------------

fn bind_identity_ix(identity_hash: &[u8; 32], recovery: &Pubkey, owner: &Pubkey) -> Instruction {
    let (pk, _) = identity_pda(identity_hash);
    let mut data = discriminator("global", "bind_identity").to_vec();
    data.extend_from_slice(identity_hash);
    data.extend_from_slice(&recovery.to_bytes());
    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(pk, false),
            AccountMeta::new(*owner, true),
            AccountMeta::new_readonly(system_program_id(), false),
        ],
        data,
    }
}

fn request_succession_ix(
    identity: &Pubkey,
    identity_hash: &[u8; 32],
    successor: &Pubkey,
    kind: u8,
    grace_secs: i64,
    required_validations: u8,
    validators: &[Pubkey],
    signer: &Pubkey,
) -> Instruction {
    let (succ_pk, _) = succession_pda(identity, successor);
    let mut data = discriminator("global", "request_succession").to_vec();
    data.extend_from_slice(&successor.to_bytes());
    data.push(kind);
    data.extend_from_slice(&grace_secs.to_le_bytes());
    data.push(required_validations);
    // Pad validators to MAX_VALIDATORS (8) with default Pubkey.
    let mut vld = [Pubkey::default(); MAX_VALIDATORS];
    for (i, v) in validators.iter().enumerate() {
        vld[i] = *v;
    }
    for v in &vld {
        data.extend_from_slice(&v.to_bytes());
    }
    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*identity, false),
            AccountMeta::new(succ_pk, false),
            AccountMeta::new(*signer, true),
            AccountMeta::new_readonly(system_program_id(), false),
        ],
        data,
    }
}

fn endorse_succession_ix(
    identity: &Pubkey,
    identity_hash: &[u8; 32],
    successor: &Pubkey,
    validator: &Keypair,
) -> Instruction {
    let (succ_pk, _) = succession_pda(identity, successor);
    let data = discriminator("global", "endorse_succession").to_vec();
    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*identity, false),
            AccountMeta::new(succ_pk, false),
            AccountMeta::new(validator.pubkey(), true),
            AccountMeta::new_readonly(system_program_id(), false),
        ],
        data,
    }
}

fn cancel_succession_ix(
    identity: &Pubkey,
    identity_hash: &[u8; 32],
    successor: &Pubkey,
    signer: &Pubkey,
) -> Instruction {
    let (succ_pk, _) = succession_pda(identity, successor);
    let data = discriminator("global", "cancel_succession").to_vec();
    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*identity, false),
            AccountMeta::new(succ_pk, false),
            AccountMeta::new(*signer, true),
            AccountMeta::new_readonly(system_program_id(), false),
        ],
        data,
    }
}

fn claim_succession_ix(
    identity: &Pubkey,
    identity_hash: &[u8; 32],
    successor: &Keypair,
) -> Instruction {
    let (succ_pk, _) = succession_pda(identity, &successor.pubkey());
    let data = discriminator("global", "claim_succession").to_vec();
    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*identity, false),
            AccountMeta::new(succ_pk, false),
            AccountMeta::new(successor.pubkey(), true),
            AccountMeta::new_readonly(system_program_id(), false),
        ],
        data,
    }
}

fn request_court_guardianship_ix(
    identity: &Pubkey,
    identity_hash: &[u8; 32],
    successor: &Pubkey,
    grace_secs: i64,
    required_validations: u8,
    validators: &[Pubkey],
    case_hash: &[u8; 32],
    scope_notes: &str,
    signer: &Pubkey,
) -> Instruction {
    let (succ_pk, _) = succession_pda(identity, successor);
    let mut data = discriminator("global", "request_court_guardianship").to_vec();
    data.extend_from_slice(&successor.to_bytes());
    data.extend_from_slice(&grace_secs.to_le_bytes());
    data.push(required_validations);
    let mut vld = [Pubkey::default(); MAX_VALIDATORS];
    for (i, v) in validators.iter().enumerate() {
        vld[i] = *v;
    }
    for v in &vld {
        data.extend_from_slice(&v.to_bytes());
    }
    data.extend_from_slice(case_hash);
    data.extend_from_slice(&borsh_ser(&scope_notes.to_string()));
    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*identity, false),
            AccountMeta::new(succ_pk, false),
            AccountMeta::new(*signer, true),
            AccountMeta::new_readonly(system_program_id(), false),
        ],
        data,
    }
}

fn revoke_guardianship_ix(
    identity: &Pubkey,
    identity_hash: &[u8; 32],
    new_owner: &Pubkey,
    revoker: &Pubkey,
) -> Instruction {
    let mut data = discriminator("global", "revoke_guardianship").to_vec();
    data.extend_from_slice(&new_owner.to_bytes());
    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*identity, false),
            AccountMeta::new(*revoker, true),
            AccountMeta::new(*new_owner, false),
        ],
        data,
    }
}

fn execute_revoke_guardianship_ix(identity: &Pubkey, new_owner: &Keypair) -> Instruction {
    let data = discriminator("global", "execute_revoke_guardianship").to_vec();
    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*identity, false),
            AccountMeta::new(new_owner.pubkey(), true),
        ],
        data,
    }
}

// ===========================================================================
// Unit tests (moved from constants.rs and helpers.rs)
// ===========================================================================

#[test]
fn succession_kind_reserved_variants_are_contiguous() {
    assert_eq!(succession_kind::SUCCESSOR, 0);
    assert_eq!(succession_kind::RECOVERY, 1);
    assert_eq!(succession_kind::TRANSFER, 2);
    assert_eq!(succession_kind::GUARDIANSHIP, 3);
    assert_eq!(succession_kind::COURT_APPOINTED_GUARDIAN, 4);
    assert_eq!(
        succession_kind::MAX,
        succession_kind::COURT_APPOINTED_GUARDIAN
    );
}

#[test]
fn is_guardianship_kind_works() {
    assert!(!is_guardianship_kind(succession_kind::SUCCESSOR));
    assert!(!is_guardianship_kind(succession_kind::RECOVERY));
    assert!(!is_guardianship_kind(succession_kind::TRANSFER));
    assert!(is_guardianship_kind(succession_kind::GUARDIANSHIP));
    assert!(is_guardianship_kind(
        succession_kind::COURT_APPOINTED_GUARDIAN
    ));
}

#[test]
fn normalize_guardianship_grace_defaults_to_180d() {
    assert_eq!(
        normalize_guardianship_grace(0),
        DEFAULT_GUARDIANSHIP_GRACE_SECS
    );
}

#[test]
fn normalize_guardianship_grace_clamps() {
    assert_eq!(
        normalize_guardianship_grace(120 * 24 * 3600),
        120 * 24 * 3600
    );
    assert_eq!(
        normalize_guardianship_grace(50 * 24 * 3600),
        MIN_GUARDIANSHIP_GRACE_SECS
    );
    assert_eq!(
        normalize_guardianship_grace(200 * 24 * 3600),
        MAX_SUCCESSION_GRACE_SECS
    );
}

#[test]
fn validate_guardianship_threshold_works() {
    assert!(validate_guardianship_threshold(3, 3));
    assert!(validate_guardianship_threshold(3, 5));
    assert!(!validate_guardianship_threshold(2, 3));
    assert!(!validate_guardianship_threshold(4, 3));
}

// ===========================================================================
// Integration tests — bind_identity
// ===========================================================================

#[tokio::test]
async fn bind_identity_ok() {
    let (mut ctx, payer) = setup().await;
    let hash: [u8; 32] = [1u8; 32];
    let recovery = Keypair::new();
    let (pk, _) = identity_pda(&hash);

    process(
        &mut ctx,
        &payer,
        bind_identity_ix(&hash, &recovery.pubkey(), &payer.pubkey()),
    )
    .await
    .expect("bind_identity failed");

    let id: Identity = read_account(&ctx, pk).await;
    assert_eq!(id.identity_hash, hash);
    assert_eq!(id.owner, payer.pubkey());
    assert_eq!(id.recovery, recovery.pubkey());
    assert_eq!(id.parcel_count, 0);
}

#[tokio::test]
async fn bind_identity_empty_hash_fails() {
    let (mut ctx, payer) = setup().await;
    let zero_hash = [0u8; 32];
    let recovery = Keypair::new();

    let result = process(
        &mut ctx,
        &payer,
        bind_identity_ix(&zero_hash, &recovery.pubkey(), &payer.pubkey()),
    )
    .await;

    assert!(result.is_err());
}

// ===========================================================================
// Integration tests — succession
// ===========================================================================

#[tokio::test]
async fn succession_request_endorse_claim_ok() {
    let (mut ctx, payer) = setup().await;

    // Bind identity.
    let id_hash: [u8; 32] = [2u8; 32];
    let recovery = Keypair::new();
    let (identity_pk, _) = identity_pda(&id_hash);
    process(
        &mut ctx,
        &payer,
        bind_identity_ix(&id_hash, &recovery.pubkey(), &payer.pubkey()),
    )
    .await
    .expect("bind_identity failed");

    // Two validators.
    let v1 = Keypair::new();
    let v2 = Keypair::new();
    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &v1.pubkey(), 10_000_000),
    )
    .await
    .unwrap();
    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &v2.pubkey(), 10_000_000),
    )
    .await
    .unwrap();

    let successor = Keypair::new();

    // Request succession (TRANSFER, required=2, grace=7d).
    process(
        &mut ctx,
        &payer,
        request_succession_ix(
            &identity_pk,
            &id_hash,
            &successor.pubkey(),
            succession_kind::TRANSFER,
            7 * 24 * 3600,
            2,
            &[v1.pubkey(), v2.pubkey()],
            &payer.pubkey(),
        ),
    )
    .await
    .expect("request_succession failed");

    let (succ_pk, _) = succession_pda(&identity_pk, &successor.pubkey());
    let succ: Succession = read_account(&ctx, succ_pk).await;
    assert_eq!(succ.validations_count, 0);
    assert_eq!(succ.required, 2);

    // Endorse from v1.
    process_with(
        &mut ctx,
        &payer,
        &[&v1],
        endorse_succession_ix(&identity_pk, &id_hash, &successor.pubkey(), &v1),
    )
    .await
    .expect("endorse v1 failed");

    let succ_after_v1: Succession = read_account(&ctx, succ_pk).await;
    assert_eq!(succ_after_v1.validations_count, 1);

    // Endorse from v2.
    process_with(
        &mut ctx,
        &payer,
        &[&v2],
        endorse_succession_ix(&identity_pk, &id_hash, &successor.pubkey(), &v2),
    )
    .await
    .expect("endorse v2 failed");

    let succ_after_v2: Succession = read_account(&ctx, succ_pk).await;
    assert_eq!(succ_after_v2.validations_count, 2);

    // Fast-forward past grace period (7 days = 604800s).
    let clock = ctx
        .banks_client
        .get_sysvar::<solana_sdk::sysvar::clock::Clock>()
        .await
        .unwrap();
    ctx.set_sysvar(&solana_sdk::sysvar::clock::Clock {
        slot: clock.slot + 1_000_000,
        epoch_start_timestamp: clock.unix_timestamp + 604800 + 1,
        epoch: clock.epoch,
        leader_schedule_epoch: clock.leader_schedule_epoch,
        unix_timestamp: clock.unix_timestamp + 604800 + 1,
    });

    // Claim succession.
    process_with(
        &mut ctx,
        &payer,
        &[&successor],
        claim_succession_ix(&identity_pk, &id_hash, &successor),
    )
    .await
    .expect("claim_succession failed");

    let id_after: Identity = read_account(&ctx, identity_pk).await;
    assert_eq!(id_after.owner, successor.pubkey());
    assert_eq!(id_after.recovery, Pubkey::default());
}

#[tokio::test]
async fn succession_cancel_ok() {
    let (mut ctx, payer) = setup().await;

    let id_hash: [u8; 32] = [3u8; 32];
    let recovery = Keypair::new();
    let (identity_pk, _) = identity_pda(&id_hash);
    process(
        &mut ctx,
        &payer,
        bind_identity_ix(&id_hash, &recovery.pubkey(), &payer.pubkey()),
    )
    .await
    .expect("bind_identity failed");

    let v1 = Keypair::new();
    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &v1.pubkey(), 10_000_000),
    )
    .await
    .unwrap();

    let successor = Keypair::new();

    process(
        &mut ctx,
        &payer,
        request_succession_ix(
            &identity_pk,
            &id_hash,
            &successor.pubkey(),
            succession_kind::SUCCESSOR,
            0, // default grace
            1,
            &[v1.pubkey()],
            &payer.pubkey(),
        ),
    )
    .await
    .expect("request_succession failed");

    // Cancel.
    process(
        &mut ctx,
        &payer,
        cancel_succession_ix(&identity_pk, &id_hash, &successor.pubkey(), &payer.pubkey()),
    )
    .await
    .expect("cancel_succession failed");

    // Verify the succession account is closed (gone).
    let (succ_pk, _) = succession_pda(&identity_pk, &successor.pubkey());
    let gone = ctx.banks_client.get_account(succ_pk).await.unwrap();
    assert!(gone.is_none(), "succession account should be closed");
}

#[tokio::test]
async fn succession_unauthorized_requester_fails() {
    let (mut ctx, payer) = setup().await;

    let id_hash: [u8; 32] = [4u8; 32];
    let recovery = Keypair::new();
    let (identity_pk, _) = identity_pda(&id_hash);
    process(
        &mut ctx,
        &payer,
        bind_identity_ix(&id_hash, &recovery.pubkey(), &payer.pubkey()),
    )
    .await
    .expect("bind_identity failed");

    let v1 = Keypair::new();
    let attacker = Keypair::new();
    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &attacker.pubkey(), 10_000_000),
    )
    .await
    .unwrap();

    let successor = Keypair::new();

    let result = process(
        &mut ctx,
        &attacker,
        request_succession_ix(
            &identity_pk,
            &id_hash,
            &successor.pubkey(),
            succession_kind::SUCCESSOR,
            0,
            1,
            &[v1.pubkey()],
            &attacker.pubkey(),
        ),
    )
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn succession_wrong_validator_endorsement_fails() {
    let (mut ctx, payer) = setup().await;

    let id_hash: [u8; 32] = [5u8; 32];
    let recovery = Keypair::new();
    let (identity_pk, _) = identity_pda(&id_hash);
    process(
        &mut ctx,
        &payer,
        bind_identity_ix(&id_hash, &recovery.pubkey(), &payer.pubkey()),
    )
    .await
    .expect("bind_identity failed");

    let v1 = Keypair::new();
    let v2_not_listed = Keypair::new();
    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &v1.pubkey(), 10_000_000),
    )
    .await
    .unwrap();
    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &v2_not_listed.pubkey(), 10_000_000),
    )
    .await
    .unwrap();

    let successor = Keypair::new();

    process(
        &mut ctx,
        &payer,
        request_succession_ix(
            &identity_pk,
            &id_hash,
            &successor.pubkey(),
            succession_kind::SUCCESSOR,
            0,
            1,
            &[v1.pubkey()],
            &payer.pubkey(),
        ),
    )
    .await
    .expect("request_succession failed");

    // v2_not_listed was NOT in the validators list — should fail.
    let result = process_with(
        &mut ctx,
        &payer,
        &[&v2_not_listed],
        endorse_succession_ix(&identity_pk, &id_hash, &successor.pubkey(), &v2_not_listed),
    )
    .await;

    assert!(result.is_err());
}

// ===========================================================================
// Integration tests — guardianship
// ===========================================================================

#[tokio::test]
async fn guardianship_request_ok() {
    let (mut ctx, payer) = setup().await;

    let id_hash: [u8; 32] = [6u8; 32];
    let recovery = Keypair::new();
    let (identity_pk, _) = identity_pda(&id_hash);
    process(
        &mut ctx,
        &payer,
        bind_identity_ix(&id_hash, &recovery.pubkey(), &payer.pubkey()),
    )
    .await
    .expect("bind_identity failed");

    let v1 = Keypair::new();
    let v2 = Keypair::new();
    let v3 = Keypair::new();
    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &v1.pubkey(), 10_000_000),
    )
    .await
    .unwrap();
    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &v2.pubkey(), 10_000_000),
    )
    .await
    .unwrap();
    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &v3.pubkey(), 10_000_000),
    )
    .await
    .unwrap();

    let guardian = Keypair::new();
    let case_hash = [7u8; 32];
    let scope_notes = "Court case #12345";

    process(
        &mut ctx,
        &payer,
        request_court_guardianship_ix(
            &identity_pk,
            &id_hash,
            &guardian.pubkey(),
            0, // default 180d grace
            3, // min guardianship threshold
            &[v1.pubkey(), v2.pubkey(), v3.pubkey()],
            &case_hash,
            scope_notes,
            &payer.pubkey(),
        ),
    )
    .await
    .expect("request_court_guardianship failed");

    let (succ_pk, _) = succession_pda(&identity_pk, &guardian.pubkey());
    let succ: Succession = read_account(&ctx, succ_pk).await;
    assert_eq!(succ.kind, succession_kind::COURT_APPOINTED_GUARDIAN);
    assert_eq!(succ.required, 3);
    assert_eq!(succ.grace_secs, DEFAULT_GUARDIANSHIP_GRACE_SECS);
}

#[tokio::test]
async fn guardianship_revoke_and_execute_ok() {
    let (mut ctx, payer) = setup().await;

    // Bind identity with a guardian as owner (simulate an active guardianship).
    let id_hash: [u8; 32] = [8u8; 32];
    let recovery = Keypair::new();
    let guardian_owner = Keypair::new();
    let (identity_pk, _) = identity_pda(&id_hash);

    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &guardian_owner.pubkey(), 10_000_000),
    )
    .await
    .unwrap();

    process(
        &mut ctx,
        &guardian_owner,
        bind_identity_ix(&id_hash, &recovery.pubkey(), &guardian_owner.pubkey()),
    )
    .await
    .expect("bind_identity failed");

    // Recovery requests revocation.
    let new_owner = Keypair::new();
    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &new_owner.pubkey(), 10_000_000),
    )
    .await
    .unwrap();

    // Actually call as recovery.
    process_with(
        &mut ctx,
        &payer,
        &[&recovery],
        revoke_guardianship_ix(
            &identity_pk,
            &id_hash,
            &new_owner.pubkey(),
            &recovery.pubkey(),
        ),
    )
    .await
    .expect("revoke_guardianship failed");

    let id_after_revoke: Identity = read_account(&ctx, identity_pk).await;
    assert!(id_after_revoke.pending_revocation);
    assert_eq!(id_after_revoke.pending_new_owner, new_owner.pubkey());

    // Execute before timelock — should fail.
    let result = process_with(
        &mut ctx,
        &payer,
        &[&new_owner],
        execute_revoke_guardianship_ix(&identity_pk, &new_owner),
    )
    .await;
    assert!(result.is_err(), "execute before timelock should fail");

    // Fast-forward past timelock (48h = 172800s).
    let clock = ctx
        .banks_client
        .get_sysvar::<solana_sdk::sysvar::clock::Clock>()
        .await
        .unwrap();
    ctx.set_sysvar(&solana_sdk::sysvar::clock::Clock {
        slot: clock.slot + 1_000_000,
        epoch_start_timestamp: clock.unix_timestamp + 172800 + 1,
        epoch: clock.epoch,
        leader_schedule_epoch: clock.leader_schedule_epoch,
        unix_timestamp: clock.unix_timestamp + 172800 + 1,
    });

    // Execute after timelock.
    process_with(
        &mut ctx,
        &payer,
        &[&new_owner],
        execute_revoke_guardianship_ix(&identity_pk, &new_owner),
    )
    .await
    .expect("execute_revoke_guardianship failed");

    let id_final: Identity = read_account(&ctx, identity_pk).await;
    assert_eq!(id_final.owner, new_owner.pubkey());
    assert_eq!(id_final.recovery, Pubkey::default());
    assert!(!id_final.pending_revocation);
}

#[tokio::test]
async fn guardianship_scope_notes_too_long_fails() {
    let (mut ctx, payer) = setup().await;

    let id_hash: [u8; 32] = [9u8; 32];
    let recovery = Keypair::new();
    let (identity_pk, _) = identity_pda(&id_hash);
    process(
        &mut ctx,
        &payer,
        bind_identity_ix(&id_hash, &recovery.pubkey(), &payer.pubkey()),
    )
    .await
    .expect("bind_identity failed");

    let v1 = Keypair::new();
    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &v1.pubkey(), 10_000_000),
    )
    .await
    .unwrap();

    let guardian = Keypair::new();
    let case_hash = [10u8; 32];
    let long_notes = "x".repeat(MAX_SCOPE_NOTES_LEN + 1);

    let result = process(
        &mut ctx,
        &payer,
        request_court_guardianship_ix(
            &identity_pk,
            &id_hash,
            &guardian.pubkey(),
            0,
            1,
            &[v1.pubkey()],
            &case_hash,
            &long_notes,
            &payer.pubkey(),
        ),
    )
    .await;

    assert!(result.is_err(), "scope_notes > 128 should fail");
}

#[tokio::test]
async fn guardianship_empty_case_hash_fails() {
    let (mut ctx, payer) = setup().await;

    let id_hash: [u8; 32] = [11u8; 32];
    let recovery = Keypair::new();
    let (identity_pk, _) = identity_pda(&id_hash);
    process(
        &mut ctx,
        &payer,
        bind_identity_ix(&id_hash, &recovery.pubkey(), &payer.pubkey()),
    )
    .await
    .expect("bind_identity failed");

    let v1 = Keypair::new();
    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &v1.pubkey(), 10_000_000),
    )
    .await
    .unwrap();

    let guardian = Keypair::new();
    let zero_hash = [0u8; 32];

    let result = process(
        &mut ctx,
        &payer,
        request_court_guardianship_ix(
            &identity_pk,
            &id_hash,
            &guardian.pubkey(),
            0,
            1,
            &[v1.pubkey()],
            &zero_hash,
            "ok",
            &payer.pubkey(),
        ),
    )
    .await;

    assert!(result.is_err(), "empty case_hash should fail");
}

// ===========================================================================
// Integration tests — duplicate validators / duplicate endorsements
// ===========================================================================

#[tokio::test]
async fn succession_duplicate_validators_fails() {
    let (mut ctx, payer) = setup().await;

    let id_hash: [u8; 32] = [20u8; 32];
    let recovery = Keypair::new();
    let (identity_pk, _) = identity_pda(&id_hash);
    process(
        &mut ctx,
        &payer,
        bind_identity_ix(&id_hash, &recovery.pubkey(), &payer.pubkey()),
    )
    .await
    .expect("bind_identity failed");

    let v1 = Keypair::new();
    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &v1.pubkey(), 10_000_000),
    )
    .await
    .unwrap();

    let successor = Keypair::new();

    // [v1, v1, ...] — duplicate must be rejected with DuplicateValidator.
    let result = process(
        &mut ctx,
        &payer,
        request_succession_ix(
            &identity_pk,
            &id_hash,
            &successor.pubkey(),
            succession_kind::SUCCESSOR,
            0,
            2, // would be "reachable" only if duplicates counted as unique
            &[v1.pubkey(), v1.pubkey()],
            &payer.pubkey(),
        ),
    )
    .await;

    assert!(
        result.is_err(),
        "duplicate validators in request_succession must fail"
    );
}

#[tokio::test]
async fn succession_duplicate_endorsement_fails() {
    let (mut ctx, payer) = setup().await;

    let id_hash: [u8; 32] = [21u8; 32];
    let recovery = Keypair::new();
    let (identity_pk, _) = identity_pda(&id_hash);
    process(
        &mut ctx,
        &payer,
        bind_identity_ix(&id_hash, &recovery.pubkey(), &payer.pubkey()),
    )
    .await
    .expect("bind_identity failed");

    let v1 = Keypair::new();
    let v2 = Keypair::new();
    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &v1.pubkey(), 10_000_000),
    )
    .await
    .unwrap();
    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &v2.pubkey(), 10_000_000),
    )
    .await
    .unwrap();

    let successor = Keypair::new();

    // required=2 with two distinct validators so the first endorse can succeed
    // and the second (duplicate) hits AlreadyEndorsed, not ValidationLimitReached.
    process(
        &mut ctx,
        &payer,
        request_succession_ix(
            &identity_pk,
            &id_hash,
            &successor.pubkey(),
            succession_kind::SUCCESSOR,
            0,
            2,
            &[v1.pubkey(), v2.pubkey()],
            &payer.pubkey(),
        ),
    )
    .await
    .expect("request_succession failed");

    // First endorsement succeeds.
    process_with(
        &mut ctx,
        &payer,
        &[&v1],
        endorse_succession_ix(&identity_pk, &id_hash, &successor.pubkey(), &v1),
    )
    .await
    .expect("first endorse should succeed");

    // Second endorsement from the same validator must fail.
    let result = process_with(
        &mut ctx,
        &payer,
        &[&v1],
        endorse_succession_ix(&identity_pk, &id_hash, &successor.pubkey(), &v1),
    )
    .await;

    assert!(
        result.is_err(),
        "duplicate endorsement from same validator must fail"
    );
}

#[tokio::test]
async fn guardianship_duplicate_validators_fails() {
    let (mut ctx, payer) = setup().await;

    let id_hash: [u8; 32] = [22u8; 32];
    let recovery = Keypair::new();
    let (identity_pk, _) = identity_pda(&id_hash);
    process(
        &mut ctx,
        &payer,
        bind_identity_ix(&id_hash, &recovery.pubkey(), &payer.pubkey()),
    )
    .await
    .expect("bind_identity failed");

    let v1 = Keypair::new();
    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &v1.pubkey(), 10_000_000),
    )
    .await
    .unwrap();

    let guardian = Keypair::new();
    let case_hash = [7u8; 32];

    // Duplicate v1 must be rejected.
    let result = process(
        &mut ctx,
        &payer,
        request_court_guardianship_ix(
            &identity_pk,
            &id_hash,
            &guardian.pubkey(),
            0,
            3,
            &[v1.pubkey(), v1.pubkey(), v1.pubkey()],
            &case_hash,
            "ok",
            &payer.pubkey(),
        ),
    )
    .await;

    assert!(
        result.is_err(),
        "duplicate validators in request_court_guardianship must fail"
    );
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

// ===========================================================================
// B6: identity-subject tests moved from terra_registry's integration suite
// ===========================================================================

// ===========================================================================
// Batch 2 (moved): endorse_succession, claim_succession, cancel_succession
// ===========================================================================

// ===========================================================================
// Batch 2: endorse_succession, claim_succession, cancel_succession
// ===========================================================================

#[tokio::test]
async fn succession_endorse_and_cancel() {
    let (mut ctx, payer) = setup().await;

    // Bind identity: owner = payer, recovery = some other wallet.
    let identity_hash = [10u8; 32];
    let (id_pda, _) = identity_pda(&identity_hash);
    let recovery = Keypair::new();
    process(
        &mut ctx,
        &payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(id_pda, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data: {
                let mut d = discriminator("global", "bind_identity").to_vec();
                d.extend_from_slice(&identity_hash);
                d.extend_from_slice(&recovery.pubkey().to_bytes());
                d
            },
        },
    )
    .await
    .expect("bind_identity failed");

    let id: Identity = read_account(&ctx, id_pda).await;
    assert_eq!(id.owner, payer.pubkey());
    assert_eq!(id.recovery, recovery.pubkey());

    // Request succession: owner transfers to successor.
    let successor = Keypair::new();
    let validator1 = Keypair::new();
    let validator2 = Keypair::new();
    let (succession_pda, _) = succession_pda(&id_pda, &successor.pubkey());
    let mut validators = [Pubkey::default(); 8]; // MAX_VALIDATORS = 8
    validators[0] = validator1.pubkey();
    validators[1] = validator2.pubkey();

    process(
        &mut ctx,
        &payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(id_pda, false),
                AccountMeta::new(succession_pda, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data: {
                let mut d = discriminator("global", "request_succession").to_vec();
                d.extend_from_slice(&successor.pubkey().to_bytes());
                d.push(0u8); // kind = SUCCESSOR
                d.extend_from_slice(&0i64.to_le_bytes()); // grace_secs = 0 (default 30d)
                d.push(2u8); // required_validations = 2
                for v in &validators {
                    d.extend_from_slice(&v.to_bytes());
                }
                d
            },
        },
    )
    .await
    .expect("request_succession failed");

    let s: Succession = read_account(&ctx, succession_pda).await;
    assert_eq!(s.validations_count, 0);
    assert_eq!(s.required, 2);

    // Fund validators so they can sign transactions.
    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &validator1.pubkey(), 10_000_000),
    )
    .await
    .expect("fund validator1 failed");
    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &validator2.pubkey(), 10_000_000),
    )
    .await
    .expect("fund validator2 failed");

    // Validator1 endorses the succession.
    process(
        &mut ctx,
        &validator1,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(id_pda, false),
                AccountMeta::new(succession_pda, false),
                AccountMeta::new(validator1.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data: discriminator("global", "endorse_succession").to_vec(),
        },
    )
    .await
    .expect("endorse_succession failed");

    let s: Succession = read_account(&ctx, succession_pda).await;
    assert_eq!(s.validations_count, 1);

    // Owner cancels the succession before it becomes effective.
    process(
        &mut ctx,
        &payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(id_pda, false),
                AccountMeta::new(succession_pda, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data: discriminator("global", "cancel_succession").to_vec(),
        },
    )
    .await
    .expect("cancel_succession failed");

    // Succession account should be closed (close = signer).
    let acc = ctx.banks_client.get_account(succession_pda).await.unwrap();
    assert!(
        acc.is_none(),
        "succession account should be closed after cancel"
    );
}

#[tokio::test]
async fn claim_succession_not_yet_effective() {
    let (mut ctx, payer) = setup().await;

    // Bind identity.
    let identity_hash = [11u8; 32];
    let (id_pda, _) = identity_pda(&identity_hash);
    let recovery = Keypair::new();
    process(
        &mut ctx,
        &payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(id_pda, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data: {
                let mut d = discriminator("global", "bind_identity").to_vec();
                d.extend_from_slice(&identity_hash);
                d.extend_from_slice(&recovery.pubkey().to_bytes());
                d
            },
        },
    )
    .await
    .expect("bind_identity failed");

    // Request succession with default grace (30 days).
    let successor = Keypair::new();
    let validator1 = Keypair::new();
    let (succession_pda, _) = succession_pda(&id_pda, &successor.pubkey());
    let mut validators = [Pubkey::default(); 8];
    validators[0] = validator1.pubkey();

    process(
        &mut ctx,
        &payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(id_pda, false),
                AccountMeta::new(succession_pda, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data: {
                let mut d = discriminator("global", "request_succession").to_vec();
                d.extend_from_slice(&successor.pubkey().to_bytes());
                d.push(0u8); // kind = SUCCESSOR
                d.extend_from_slice(&0i64.to_le_bytes()); // grace_secs = 0 (default)
                d.push(1u8); // required_validations = 1
                for v in &validators {
                    d.extend_from_slice(&v.to_bytes());
                }
                d
            },
        },
    )
    .await
    .expect("request_succession failed");

    // Fund validator and endorse.
    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &validator1.pubkey(), 10_000_000),
    )
    .await
    .expect("fund validator1 failed");
    process(
        &mut ctx,
        &validator1,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(id_pda, false),
                AccountMeta::new(succession_pda, false),
                AccountMeta::new(validator1.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data: discriminator("global", "endorse_succession").to_vec(),
        },
    )
    .await
    .expect("endorse_succession failed");

    // Claim fails: grace period not elapsed (30 days default).
    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &successor.pubkey(), 10_000_000),
    )
    .await
    .expect("fund successor failed");
    let res = process(
        &mut ctx,
        &successor,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(id_pda, false),
                AccountMeta::new(succession_pda, false),
                AccountMeta::new(successor.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data: discriminator("global", "claim_succession").to_vec(),
        },
    )
    .await;
    assert!(res.is_err(), "claim should fail — not yet effective");
}

// ============================================================================
// P0-1: Succession Endorsement Duplication Prevention
// ============================================================================

/// Helper: bind identity + request succession, return (identity_pda, succession_pda).
async fn setup_succession(
    ctx: &mut ProgramTestContext,
    payer: &Keypair,
    identity_hash: [u8; 32],
    successor: &Pubkey,
    validators: &[Pubkey; 8],
    required: u8,
) -> (Pubkey, Pubkey) {
    let (id_pda, _) = identity_pda(&identity_hash);
    let recovery = Keypair::new();

    // Bind identity
    process(
        ctx,
        payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(id_pda, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data: {
                let mut d = discriminator("global", "bind_identity").to_vec();
                d.extend_from_slice(&identity_hash);
                d.extend_from_slice(&recovery.pubkey().to_bytes());
                d
            },
        },
    )
    .await
    .expect("bind_identity");

    let (succ_pda, _) = succession_pda(&id_pda, successor);

    // Request succession
    process(
        ctx,
        payer,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(id_pda, false),
                AccountMeta::new(succ_pda, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data: {
                let mut d = discriminator("global", "request_succession").to_vec();
                d.extend_from_slice(&successor.to_bytes());
                d.push(0u8); // kind = SUCCESSOR
                d.extend_from_slice(&0i64.to_le_bytes()); // grace_secs = 0 (default)
                d.push(required);
                for v in validators {
                    d.extend_from_slice(&v.to_bytes());
                }
                d
            },
        },
    )
    .await
    .expect("request_succession");

    (id_pda, succ_pda)
}

/// Helper: endorse a succession.
async fn endorse(
    ctx: &mut ProgramTestContext,
    payer: &Keypair,
    id_pda: Pubkey,
    succ_pda: Pubkey,
    validator: &Keypair,
) {
    process(
        ctx,
        validator,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(id_pda, false),
                AccountMeta::new(succ_pda, false),
                AccountMeta::new(validator.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data: discriminator("global", "endorse_succession").to_vec(),
        },
    )
    .await
    .expect("endorse_succession");
}

// P0-1 Test 1: Duplicate endorsement by same validator is rejected.
#[tokio::test]
async fn p0_1_succession_duplicate_endorsement_rejected() {
    let (mut ctx, payer) = setup().await;
    let identity_hash = [0xA1u8; 32];
    let successor = Keypair::new();
    let v1 = Keypair::new();
    let v2 = Keypair::new();
    let mut validators = [Pubkey::default(); 8];
    validators[0] = v1.pubkey();
    validators[1] = v2.pubkey();

    // required=2, 2 declared validators — valid setup.
    let (id_pda, succ_pda) = setup_succession(
        &mut ctx,
        &payer,
        identity_hash,
        &successor.pubkey(),
        &validators,
        2,
    )
    .await;

    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &v1.pubkey(), 10_000_000),
    )
    .await
    .unwrap();

    // First endorsement succeeds.
    endorse(&mut ctx, &payer, id_pda, succ_pda, &v1).await;

    let s: Succession = read_account(&ctx, succ_pda).await;
    assert_eq!(s.validations_count, 1);
    assert_eq!(s.endorsers_count, 1);
    assert_eq!(s.endorsers[0], v1.pubkey());

    // Second endorsement by same validator fails.
    let res = process(
        &mut ctx,
        &v1,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(id_pda, false),
                AccountMeta::new(succ_pda, false),
                AccountMeta::new(v1.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data: discriminator("global", "endorse_succession").to_vec(),
        },
    )
    .await;
    assert_custom_error(res, 6027, "duplicate endorsement rejected");

    // Count unchanged.
    let s: Succession = read_account(&ctx, succ_pda).await;
    assert_eq!(s.validations_count, 1);
    assert_eq!(s.endorsers_count, 1);
}

// P0-1 Test 2: Two distinct validators reach quorum.
#[tokio::test]
async fn p0_1_succession_two_distinct_validators_reach_quorum() {
    let (mut ctx, payer) = setup().await;
    let identity_hash = [0xA2u8; 32];
    let successor = Keypair::new();
    let v1 = Keypair::new();
    let v2 = Keypair::new();
    let mut validators = [Pubkey::default(); 8];
    validators[0] = v1.pubkey();
    validators[1] = v2.pubkey();

    let (id_pda, succ_pda) = setup_succession(
        &mut ctx,
        &payer,
        identity_hash,
        &successor.pubkey(),
        &validators,
        2,
    )
    .await;

    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &v1.pubkey(), 10_000_000),
    )
    .await
    .unwrap();
    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &v2.pubkey(), 10_000_000),
    )
    .await
    .unwrap();

    // V1 endorses.
    endorse(&mut ctx, &payer, id_pda, succ_pda, &v1).await;
    let s: Succession = read_account(&ctx, succ_pda).await;
    assert_eq!(s.validations_count, 1);

    // V2 endorses — quorum met.
    endorse(&mut ctx, &payer, id_pda, succ_pda, &v2).await;
    let s: Succession = read_account(&ctx, succ_pda).await;
    assert_eq!(s.validations_count, 2);
    assert_eq!(s.endorsers_count, 2);
}

// P0-1 Test 3: Single validator cannot reach quorum via duplicate endorsement.
#[tokio::test]
async fn p0_1_succession_quorum_cannot_be_reached_by_one_validator() {
    let (mut ctx, payer) = setup().await;
    let identity_hash = [0xA3u8; 32];
    let successor = Keypair::new();
    let v1 = Keypair::new();
    let v2 = Keypair::new();
    let mut validators = [Pubkey::default(); 8];
    validators[0] = v1.pubkey();
    validators[1] = v2.pubkey();

    // required=2, 2 validators declared. Only v1 will endorse.
    let (id_pda, succ_pda) = setup_succession(
        &mut ctx,
        &payer,
        identity_hash,
        &successor.pubkey(),
        &validators,
        2,
    )
    .await;

    process(
        &mut ctx,
        &payer,
        fund_ix(&payer.pubkey(), &v1.pubkey(), 10_000_000),
    )
    .await
    .unwrap();

    // First endorsement succeeds.
    endorse(&mut ctx, &payer, id_pda, succ_pda, &v1).await;

    // Second endorsement by v1 rejected (duplicate).
    let res = process(
        &mut ctx,
        &v1,
        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(id_pda, false),
                AccountMeta::new(succ_pda, false),
                AccountMeta::new(v1.pubkey(), true),
                AccountMeta::new_readonly(system_program_id(), false),
            ],
            data: discriminator("global", "endorse_succession").to_vec(),
        },
    )
    .await;
    assert_custom_error(
        res,
        6027,
        "single validator cannot reach quorum via duplication",
    );

    // Quorum not met — only 1 endorsement for required=2.
    let s: Succession = read_account(&ctx, succ_pda).await;
    assert_eq!(s.validations_count, 1);
    assert!(s.validations_count < s.required);
}
