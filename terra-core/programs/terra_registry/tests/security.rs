use anchor_lang::AccountDeserialize;
use solana_program_test::{tokio, ProgramTest, ProgramTestContext};
use solana_sdk::{
    hash::hash,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use terra_registry::security::{
    request_status, restriction_type, review_decision, remedy_type,
    SecurityRequest, SecurityReviewAssignment, SecurityReviewCase,
    SecurityAuditEvent,
    EmergencyProtection, JurisdictionalRestrictionRequest, PrivacyRequest,
};
use terra_registry::ID as PROGRAM_ID;

fn security_request_pda(requester: &Pubkey, nonce: u16) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"security_request", requester.as_ref(), &nonce.to_le_bytes()],
        &PROGRAM_ID,
    )
}

fn privacy_request_pda(requester: &Pubkey, subject: &Pubkey, nonce: u16) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"privacy_request",
            requester.as_ref(),
            subject.as_ref(),
            &nonce.to_le_bytes(),
        ],
        &PROGRAM_ID,
    )
}

fn jurisdictional_restriction_pda(
    authority: &Pubkey,
    subject: &Pubkey,
    nonce: u16,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"jurisdiction",
            authority.as_ref(),
            subject.as_ref(),
            &nonce.to_le_bytes(),
        ],
        &PROGRAM_ID,
    )
}

fn review_case_pda(request: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"review_case", request.as_ref()], &PROGRAM_ID)
}

fn review_assignment_pda(case: &Pubkey, reviewer: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"review_assignment", case.as_ref(), reviewer.as_ref()],
        &PROGRAM_ID,
    )
}

fn security_decision_pda(request: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"decision", request.as_ref()], &PROGRAM_ID)
}

fn appeal_pda(decision: &Pubkey, appellant: &Pubkey, nonce: u16) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"appeal", decision.as_ref(), appellant.as_ref(), &nonce.to_le_bytes()],
        &PROGRAM_ID,
    )
}

fn audit_event_pda(entity: &Pubkey, sequence: u32) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"audit_event", entity.as_ref(), &sequence.to_le_bytes()],
        &PROGRAM_ID,
    )
}

fn emergency_protection_pda(subject: &Pubkey, nonce: u16) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"emergency_protect", subject.as_ref(), &nonce.to_le_bytes()],
        &PROGRAM_ID,
    )
}

fn registry_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"validator_registry"], &PROGRAM_ID)
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

fn build_ix(
    program_id: &Pubkey,
    accounts: Vec<AccountMeta>,
    data: Vec<u8>,
) -> Instruction {
    Instruction {
        program_id: *program_id,
        accounts,
        data,
    }
}

async fn send_tx(
    ctx: &mut ProgramTestContext,
    fee_payer: &Pubkey,
    signers: &[&Keypair],
    ixs: &[Instruction],
) -> Result<(), solana_program_test::BanksClientError> {
    use solana_sdk::signature::Signature;
    use std::collections::HashSet;
    use std::sync::Mutex;

    static SEEN: Mutex<Option<HashSet<Signature>>> = Mutex::new(None);

    let mut blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();
    let mut tx = Transaction::new_signed_with_payer(ixs, Some(fee_payer), signers, blockhash);

    let duplicate = {
        let mut guard = SEEN.lock().unwrap();
        let set = guard.get_or_insert_with(HashSet::new);
        if set.len() > 16_384 {
            set.clear();
        }
        let dup = set.contains(&tx.signatures[0]);
        set.insert(tx.signatures[0]);
        dup
    };
    if duplicate {
        for _ in 0..64 {
            tokio::time::sleep(std::time::Duration::from_millis(2)).await;
            let next = ctx.banks_client.get_latest_blockhash().await.unwrap();
            if next != blockhash {
                blockhash = next;
                break;
            }
        }
        tx = Transaction::new_signed_with_payer(ixs, Some(fee_payer), signers, blockhash);
        SEEN.lock()
            .unwrap()
            .get_or_insert_with(HashSet::new)
            .insert(tx.signatures[0]);
    }

    ctx.last_blockhash = blockhash;
    ctx.banks_client.process_transaction(tx).await.map(|_| ())
}

async fn process(
    ctx: &mut ProgramTestContext,
    payer: &Keypair,
    ix: Instruction,
) -> Result<(), solana_program_test::BanksClientError> {
    send_tx(ctx, &payer.pubkey(), &[payer], &[ix]).await
}

async fn process_with(
    ctx: &mut ProgramTestContext,
    fee_payer: &Keypair,
    signers: &[&Keypair],
    ix: Instruction,
) -> Result<(), solana_program_test::BanksClientError> {
    send_tx(ctx, &fee_payer.pubkey(), signers, &[ix]).await
}

async fn setup() -> (ProgramTestContext, Keypair) {
    let mut pt = ProgramTest::new("terra_registry", PROGRAM_ID, None);
    pt.add_program("terra_registry", PROGRAM_ID, None);
    pt.set_compute_max_units(500_000);
    let ctx = pt.start_with_context().await;
    let payer = ctx.payer.insecure_clone();
    (ctx, payer)
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
async fn submit_security_request_ok() {
    let (mut ctx, payer) = setup().await;

    let nonce: u16 = 1;
    let requester = payer.pubkey();

    let (request_pda, _) = security_request_pda(&requester, nonce);

    let mut data = discriminator("global", "submit_security_request").to_vec();
    data.extend_from_slice(&nonce.to_le_bytes());
    data.extend_from_slice(&[restriction_type::PERSONAL_PRIVACY]);
    data.extend_from_slice(&borsh_ser(&"Test privacy request description".to_string()));
    data.extend_from_slice(&[remedy_type::RESTRICT_ACCESS]);
    data.extend_from_slice(&[67, 65]); // CA
    data.extend_from_slice(&[0]); // emergency = false

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(request_pda, false),
            AccountMeta::new(requester, true),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::id(), false),
        ],
        data,
    };

    process_with(&mut ctx, &payer, &[&payer], ix).await.unwrap();

    let sr: SecurityRequest = read_account(&ctx, request_pda).await;

    assert_eq!(sr.requester, requester);
    assert_eq!(sr.request_type, restriction_type::PERSONAL_PRIVACY);
    assert_eq!(sr.requested_remedy, remedy_type::RESTRICT_ACCESS);
    assert_eq!(sr.jurisdiction, [67, 65]);
    assert_eq!(sr.status, request_status::OPEN);
    assert!(!sr.emergency);
}

#[tokio::test]
async fn submit_privacy_request_ok() {
    let (mut ctx, payer) = setup().await;

    let requester = payer.pubkey();
    let subject = payer.pubkey();
    let nonce: u16 = 1;

    let (request_pda, _) = privacy_request_pda(&requester, &subject, nonce);

    let mut data = discriminator("global", "submit_privacy_request").to_vec();
    data.extend_from_slice(&nonce.to_le_bytes());
    data.extend_from_slice(&[restriction_type::SENSITIVE_EVIDENCE]);
    data.extend_from_slice(&borsh_ser(&"Sensitive document should be restricted".to_string()));
    data.extend_from_slice(&[remedy_type::REDACT]);
    data.extend_from_slice(&[85, 83]); // US
    data.extend_from_slice(&[1]); // emergency = true

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(request_pda, false),
            AccountMeta::new(requester, true),
            AccountMeta::new_readonly(subject, false),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::id(), false),
        ],
        data,
    };

    process_with(&mut ctx, &payer, &[&payer], ix).await.unwrap();

    let pr: PrivacyRequest = read_account(&ctx, request_pda).await;

    assert_eq!(pr.requester, requester);
    assert_eq!(pr.subject, subject);
    assert_eq!(pr.request_type, restriction_type::SENSITIVE_EVIDENCE);
    assert_eq!(pr.requested_remedy, remedy_type::REDACT);
    assert_eq!(pr.jurisdiction, [85, 83]);
    assert_eq!(pr.status, request_status::OPEN);
    assert!(pr.emergency);
}

#[tokio::test]
async fn submit_jurisdictional_restriction_ok() {
    let (mut ctx, payer) = setup().await;

    let authority = payer.pubkey();
    let subject = payer.pubkey();
    let nonce: u16 = 1;

    let (restriction_pda, _) =
        jurisdictional_restriction_pda(&authority, &subject, nonce);

    let mut data = discriminator("global", "submit_jurisdictional_restriction").to_vec();
    data.extend_from_slice(&nonce.to_le_bytes());
    data.extend_from_slice(&[restriction_type::JURISDICTIONAL]);
    data.extend_from_slice(&borsh_ser(&"Court order to restrict access".to_string()));
    data.extend_from_slice(&[remedy_type::RESTRICT_ACCESS]);
    data.extend_from_slice(&[85, 83]); // US
    data.extend_from_slice(&[1, 0, 0, 0, 0, 0, 0, 0]); // expiry = 1_000_000 as i64 LE

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(restriction_pda, false),
            AccountMeta::new(authority, true),
            AccountMeta::new_readonly(subject, false),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::id(), false),
        ],
        data,
    };

    process_with(&mut ctx, &payer, &[&payer], ix).await.unwrap();

    let jr: JurisdictionalRestrictionRequest = read_account(&ctx, restriction_pda).await;

    assert_eq!(jr.authority, authority);
    assert_eq!(jr.subject, subject);
    assert_eq!(jr.restriction_type, restriction_type::JURISDICTIONAL);
    assert_eq!(jr.requested_remedy, remedy_type::RESTRICT_ACCESS);
    assert_eq!(jr.jurisdiction, [85, 83]);
    assert_eq!(jr.status, request_status::OPEN);
    assert_eq!(jr.expires_at, Some(1_000_000));
}

#[tokio::test]
async fn create_review_case_ok() {
    let (mut ctx, payer) = setup().await;

    let requester = payer.pubkey();
    let nonce: u16 = 1;

    // First submit a security request
    let mut data = discriminator("global", "submit_security_request").to_vec();
    data.extend_from_slice(&nonce.to_le_bytes());
    data.extend_from_slice(&[restriction_type::SECURITY_THREAT]);
    data.extend_from_slice(&borsh_ser(&"Test threat report".to_string()));
    data.extend_from_slice(&[remedy_type::ENCRYPT]);
    data.extend_from_slice(&[67, 65]); // CA
    data.extend_from_slice(&[0]); // emergency = false

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(security_request_pda(&requester, nonce).0, false),
            AccountMeta::new(requester, true),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::id(), false),
        ],
        data,
    };
    process_with(&mut ctx, &payer, &[&payer], ix).await.unwrap();

    let (request_pda, _) = security_request_pda(&requester, nonce);

    // Now create review case
    let mut data = discriminator("global", "create_review_case").to_vec();
    data.extend_from_slice(&nonce.to_le_bytes());

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(review_case_pda(&request_pda).0, false),
            AccountMeta::new(request_pda, false),
            AccountMeta::new(requester, true),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::id(), false),
        ],
        data,
    };
    process_with(&mut ctx, &payer, &[&payer], ix).await.unwrap();

    let (case_pda, _) = review_case_pda(&request_pda);
    let rc: SecurityReviewCase = read_account(&ctx, case_pda).await;

    assert_eq!(rc.request, request_pda);
    assert_eq!(rc.requester, requester);
    assert_eq!(rc.committee_size, 5);
    assert_eq!(rc.decision, review_decision::PENDING);
    assert_eq!(rc.votes_upheld, 0);
    assert_eq!(rc.votes_dismissed, 0);
}

#[tokio::test]
async fn assign_reviewer_ok() {
    let (mut ctx, payer) = setup().await;

    let requester = payer.pubkey();
    let reviewer = payer.pubkey();
    let nonce: u16 = 1;

    // Create registry (required by initialize_validator_reputation)
    let (registry_pda, _) = registry_pda();
    let mut data = discriminator("global", "create_registry").to_vec();
    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(registry_pda, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::id(), false),
        ],
        data,
    };
    process_with(&mut ctx, &payer, &[&payer], ix).await.unwrap();

    // Submit security request
    let mut data = discriminator("global", "submit_security_request").to_vec();
    data.extend_from_slice(&nonce.to_le_bytes());
    data.extend_from_slice(&[restriction_type::SECURITY_THREAT]);
    data.extend_from_slice(&borsh_ser(&"Test threat".to_string()));
    data.extend_from_slice(&[remedy_type::RESTRICT_ACCESS]);
    data.extend_from_slice(&[67, 65]); // CA
    data.extend_from_slice(&[0]); // emergency = false

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(security_request_pda(&requester, nonce).0, false),
            AccountMeta::new(requester, true),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::id(), false),
        ],
        data,
    };
    process_with(&mut ctx, &payer, &[&payer], ix).await.unwrap();
    let (request_pda, _) = security_request_pda(&requester, nonce);

    // Create review case
    let mut data = discriminator("global", "create_review_case").to_vec();
    data.extend_from_slice(&nonce.to_le_bytes());

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(review_case_pda(&request_pda).0, false),
            AccountMeta::new(request_pda, false),
            AccountMeta::new(requester, true),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::id(), false),
        ],
        data,
    };
    process_with(&mut ctx, &payer, &[&payer], ix).await.unwrap();
    let (case_pda, _) = review_case_pda(&request_pda);

    // Initialize validator reputation for reviewer
    let reviewer_reputation_pda = Pubkey::find_program_address(
        &[b"validator_reputation", reviewer.as_ref()],
        &PROGRAM_ID,
    ).0;
    let mut data = discriminator("global", "initialize_validator_reputation").to_vec();
    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(reviewer_reputation_pda, false),
            AccountMeta::new_readonly(reviewer, false),
            AccountMeta::new_readonly(registry_pda, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::id(), false),
        ],
        data,
    };
    process_with(&mut ctx, &payer, &[&payer], ix).await.unwrap();

    // Assign reviewer
    let mut data = discriminator("global", "assign_reviewer").to_vec();

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(review_assignment_pda(&case_pda, &reviewer).0, false),
            AccountMeta::new(case_pda, false),
            AccountMeta::new(reviewer_reputation_pda, false),
            AccountMeta::new_readonly(reviewer, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::id(), false),
        ],
        data,
    };
    process_with(&mut ctx, &payer, &[&payer], ix).await.unwrap();

    let (assignment_pda, _) = review_assignment_pda(&case_pda, &reviewer);
    let ra: SecurityReviewAssignment = read_account(&ctx, assignment_pda).await;

    assert_eq!(ra.case, case_pda);
    assert_eq!(ra.reviewer, reviewer);
    assert!(!ra.voted);
    assert_eq!(ra.vote, review_decision::PENDING);
}

#[tokio::test]
async fn emergency_protect_ok() {
    let (mut ctx, payer) = setup().await;

    let requester = payer.pubkey();
    let subject = payer.pubkey();
    let nonce: u16 = 1;

    let (protection_pda, _) = emergency_protection_pda(&subject, nonce);

    let mut data = discriminator("global", "emergency_protect").to_vec();
    data.extend_from_slice(&nonce.to_le_bytes());

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(protection_pda, false),
            AccountMeta::new(requester, true),
            AccountMeta::new_readonly(subject, false),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::id(), false),
        ],
        data,
    };

    process_with(&mut ctx, &payer, &[&payer], ix).await.unwrap();

    let ep: EmergencyProtection = read_account(&ctx, protection_pda).await;

    assert_eq!(ep.subject, subject);
    assert_eq!(ep.requester, requester);
    assert!(!ep.lifted);
    assert!(ep.expires_at > 0);
}

#[tokio::test]
async fn record_audit_event_ok() {
    let (mut ctx, payer) = setup().await;

    let actor = payer.pubkey();
    let entity = payer.pubkey();
    let sequence: u32 = 1;

    let mut data = discriminator("global", "record_audit_event").to_vec();
    data.extend_from_slice(&sequence.to_le_bytes());
    data.extend_from_slice(&[1]); // action: request
    data.extend_from_slice(&borsh_ser(&"Test audit event".to_string()));

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(audit_event_pda(&entity, sequence).0, false),
            AccountMeta::new_readonly(entity, false),
            AccountMeta::new(actor, true),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::id(), false),
        ],
        data,
    };

    process_with(&mut ctx, &payer, &[&payer], ix).await.unwrap();

    let (audit_pda, _) = audit_event_pda(&entity, sequence);
    let ae: SecurityAuditEvent = read_account(&ctx, audit_pda).await;

    assert_eq!(ae.entity, entity);
    assert_eq!(ae.sequence, sequence);
    assert_eq!(ae.action, 1);
    assert_eq!(ae.actor, actor);
    assert_eq!(ae.reason, "Test audit event");
}

#[tokio::test]
async fn submit_security_request_invalid_request_type() {
    let (mut ctx, payer) = setup().await;

    let requester = payer.pubkey();
    let nonce: u16 = 1;

    let (request_pda, _) = security_request_pda(&requester, nonce);

    let mut data = discriminator("global", "submit_security_request").to_vec();
    data.extend_from_slice(&nonce.to_le_bytes());
    data.extend_from_slice(&[99u8]); // request_type = 99 which is > MAX (6)
    data.extend_from_slice(&borsh_ser(&"Test".to_string()));
    data.extend_from_slice(&[remedy_type::RESTRICT_ACCESS]);
    data.extend_from_slice(&[67, 65]); // CA
    data.extend_from_slice(&[0]); // emergency = false

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(request_pda, false),
            AccountMeta::new(requester, true),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::id(), false),
        ],
        data,
    };

    let result = process_with(&mut ctx, &payer, &[&payer], ix).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn submit_security_request_description_too_long() {
    let (mut ctx, payer) = setup().await;

    let requester = payer.pubkey();
    let nonce: u16 = 1;

    let (request_pda, _) = security_request_pda(&requester, nonce);

    let long_desc = "a".repeat(257);

    let mut data = discriminator("global", "submit_security_request").to_vec();
    data.extend_from_slice(&nonce.to_le_bytes());
    data.extend_from_slice(&[restriction_type::PERSONAL_PRIVACY]);
    data.extend_from_slice(&borsh_ser(&long_desc));
    data.extend_from_slice(&[remedy_type::RESTRICT_ACCESS]);
    data.extend_from_slice(&[67, 65]); // CA
    data.extend_from_slice(&[0]); // emergency = false

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(request_pda, false),
            AccountMeta::new(requester, true),
            AccountMeta::new_readonly(solana_sdk_ids::system_program::id(), false),
        ],
        data,
    };

    let result = process_with(&mut ctx, &payer, &[&payer], ix).await;
    assert!(result.is_err());
}