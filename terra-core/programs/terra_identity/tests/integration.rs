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

fn bind_identity_ix(
    identity_hash: &[u8; 32],
    recovery: &Pubkey,
    owner: &Pubkey,
) -> Instruction {
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

fn execute_revoke_guardianship_ix(
    identity: &Pubkey,
    new_owner: &Keypair,
) -> Instruction {
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
    assert!(is_guardianship_kind(succession_kind::COURT_APPOINTED_GUARDIAN));
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
    process(&mut ctx, &payer, fund_ix(&payer.pubkey(), &v1.pubkey(), 10_000_000))
        .await
        .unwrap();
    process(&mut ctx, &payer, fund_ix(&payer.pubkey(), &v2.pubkey(), 10_000_000))
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
    let clock = ctx.banks_client.get_sysvar::<solana_sdk::sysvar::clock::Clock>().await.unwrap();
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
    process(&mut ctx, &payer, fund_ix(&payer.pubkey(), &v1.pubkey(), 10_000_000))
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
    process(&mut ctx, &payer, fund_ix(&payer.pubkey(), &attacker.pubkey(), 10_000_000))
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
    process(&mut ctx, &payer, fund_ix(&payer.pubkey(), &v1.pubkey(), 10_000_000))
        .await
        .unwrap();
    process(&mut ctx, &payer, fund_ix(&payer.pubkey(), &v2_not_listed.pubkey(), 10_000_000))
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
    process(&mut ctx, &payer, fund_ix(&payer.pubkey(), &v1.pubkey(), 10_000_000))
        .await
        .unwrap();
    process(&mut ctx, &payer, fund_ix(&payer.pubkey(), &v2.pubkey(), 10_000_000))
        .await
        .unwrap();
    process(&mut ctx, &payer, fund_ix(&payer.pubkey(), &v3.pubkey(), 10_000_000))
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

    process(&mut ctx, &payer, fund_ix(&payer.pubkey(), &guardian_owner.pubkey(), 10_000_000))
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
    process(&mut ctx, &payer, fund_ix(&payer.pubkey(), &new_owner.pubkey(), 10_000_000))
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
    let clock = ctx.banks_client.get_sysvar::<solana_sdk::sysvar::clock::Clock>().await.unwrap();
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
    process(&mut ctx, &payer, fund_ix(&payer.pubkey(), &v1.pubkey(), 10_000_000))
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
    process(&mut ctx, &payer, fund_ix(&payer.pubkey(), &v1.pubkey(), 10_000_000))
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
