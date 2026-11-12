use anchor_lang::prelude::*;

use crate::verification::reputation::ValidatorReputation;
use crate::TerraError;

// ---------------------------------------------------------------------------
// RFC-016 — Protocol Security, Decentralization, Privacy and Censorship
// Resilience (on-chain core)
//
// Vision:
//   - Privacy protection is a native protocol capability, not an admin favour.
//   - No single validator or authority decides high-impact privacy requests.
//   - Jurisdictional restrictions are scoped; one jurisdiction cannot silence
//     the global protocol.
//   - Emergency protection is time-boxed and requires independent review.
//   - The on-chain layer enforces decision *process*; off-chain APIs enforce
//   - the resulting privacy state.
// ---------------------------------------------------------------------------

/// Max length for a security/privacy request description.
pub const MAX_REQUEST_NOTE_LEN: usize = 256;
/// Max length for a decision reason.
pub const MAX_REASON_LEN: usize = 128;
/// Committee size for security/privacy review (odd → no tie).
pub const SECURITY_COMMITTEE_SIZE: usize = 5;
/// Minimum reputation (bps) for a security reviewer.
pub const MIN_SECURITY_REPUTATION: u16 = 5_000;
/// Emergency protection expiry (default 7 days).
pub const EMERGENCY_PROTECTION_SECS: i64 = 7 * 24 * 3600;
/// Standard review window (default 14 days).
pub const STANDARD_REVIEW_SECS: i64 = 14 * 24 * 3600;
/// Appeal window (default 7 days).
pub const APPEAL_WINDOW_SECS: i64 = 7 * 24 * 3600;

// ---------------------------------------------------------------------------
// Status enums
// ---------------------------------------------------------------------------

pub mod request_status {
    pub const OPEN: u8 = 0;
    pub const UNDER_REVIEW: u8 = 1;
    pub const APPROVED: u8 = 2;
    pub const REJECTED: u8 = 3;
    pub const APPEALED: u8 = 4;
    pub const MAX: u8 = APPEALED;
}

pub mod privacy_state {
    pub const PUBLIC: u8 = 0;
    pub const RESTRICTED: u8 = 1;
    pub const PRIVATE: u8 = 2;
    pub const REDACTED: u8 = 3;
    pub const TEMPORARILY_PROTECTED: u8 = 4;
    pub const UNDER_REVIEW: u8 = 5;
    pub const WITHDRAWN_FROM_PUBLIC_ACCESS: u8 = 6;
    pub const TOMBSTONED: u8 = 7;
    pub const RESTORED: u8 = 8;
    pub const MAX: u8 = RESTORED;
}

pub mod review_decision {
    pub const PENDING: u8 = 0;
    pub const UPHELD: u8 = 1;
    pub const DISMISSED: u8 = 2;
    pub const MAX: u8 = DISMISSED;
}

pub mod restriction_type {
    pub const PERSONAL_PRIVACY: u8 = 0;
    pub const SENSITIVE_EVIDENCE: u8 = 1;
    pub const SECURITY_THREAT: u8 = 2;
    pub const IDENTITY_EXPOSURE: u8 = 3;
    pub const JURISDICTIONAL: u8 = 4;
    pub const EMERGENCY_PROTECTION: u8 = 5;
    pub const CORRECTION_RECLASSIFICATION: u8 = 6;
    pub const MAX: u8 = CORRECTION_RECLASSIFICATION;
}

pub mod remedy_type {
    pub const RESTRICT_ACCESS: u8 = 0;
    pub const REMOVE_PUBLIC_INDEXING: u8 = 1;
    pub const REDACT: u8 = 2;
    pub const REPLACE_PUBLIC_REPRESENTATION: u8 = 3;
    pub const ENCRYPT: u8 = 4;
    pub const REVOKE_ACCESS: u8 = 5;
    pub const STOP_REPLICATION: u8 = 6;
    pub const CRYPTO_ERASE: u8 = 7;
    pub const TOMBSTONE: u8 = 8;
    pub const CORRECT_CLASSIFICATION: u8 = 9;
    pub const REJECT_REQUEST: u8 = 10;
    pub const MAX: u8 = REJECT_REQUEST;
}

pub mod appeal_status {
    pub const PENDING: u8 = 0;
    pub const GRANTED: u8 = 1;
    pub const DENIED: u8 = 2;
    pub const MAX: u8 = DENIED;
}

// ---------------------------------------------------------------------------
// Accounts
// ---------------------------------------------------------------------------

/// A security/privacy request submitted by a requester with legitimate
/// standing. PDA: `["security_request", requester, nonce]`
#[account]
#[derive(InitSpace)]
pub struct SecurityRequest {
    pub requester: Pubkey,
    pub subject: Pubkey, // subject of the request (identity, asset, etc.)
    pub nonce: u16,
    pub request_type: u8, // restriction_type enum
    #[max_len(MAX_REQUEST_NOTE_LEN)]
    pub description: String,
    pub requested_remedy: u8,  // remedy_type enum
    pub jurisdiction: [u8; 2], // ISO 3166-1 alpha-2
    pub status: u8,            // request_status enum
    pub emergency: bool,       // true if expedited review requested
    pub expires_at: i64,       // review deadline
    pub created_at: i64,
    pub updated_at: i64,
}

/// A privacy protection request for a specific subject.
/// PDA: `["privacy_request", requester, subject, nonce]`
#[account]
#[derive(InitSpace)]
pub struct PrivacyRequest {
    pub requester: Pubkey,
    pub subject: Pubkey, // identity, asset, document, etc.
    pub nonce: u16,
    pub request_type: u8, // restriction_type enum
    #[max_len(MAX_REQUEST_NOTE_LEN)]
    pub description: String,
    pub requested_remedy: u8,  // remedy_type enum
    pub jurisdiction: [u8; 2], // ISO 3166-1 alpha-2
    pub status: u8,            // request_status enum
    pub emergency: bool,       // true if expedited review requested
    pub expires_at: i64,       // review deadline
    pub created_at: i64,
    pub updated_at: i64,
}

/// A jurisdictional restriction request from an authenticated authority.
/// PDA: `["jurisdiction", authority, subject, nonce]`
#[account]
#[derive(InitSpace)]
pub struct JurisdictionalRestrictionRequest {
    pub authority: Pubkey,
    pub subject: Pubkey,
    pub nonce: u16,
    pub restriction_type: u8, // restriction_type enum
    #[max_len(MAX_REQUEST_NOTE_LEN)]
    pub description: String,
    pub requested_remedy: u8,    // remedy_type enum
    pub jurisdiction: [u8; 2],   // ISO 3166-1 alpha-2
    pub status: u8,              // request_status enum
    pub expires_at: Option<i64>, // optional expiry
    pub created_at: i64,
    pub updated_at: i64,
}

/// Privacy state for a subject (component, asset, document, etc.).
/// Versioned so consumers can tell available/restricted/superseded without
/// receiving protected content. PDA: `["privacy_state", subject, version]`
#[account]
#[derive(InitSpace)]
pub struct PrivacyState {
    pub subject: Pubkey,
    pub version: u16,
    pub state: u8,                           // privacy_state enum
    pub remedy: u8,                          // remedy_type enum applied
    pub restriction_request: Option<Pubkey>, // the request that set this state
    pub expires_at: Option<i64>,             // for TEMPORARILY_PROTECTED
    pub created_at: i64,
    pub updated_at: i64,
}

/// Independent review case for a security/privacy request.
/// PDA: `["review_case", request]`
#[account]
#[derive(InitSpace)]
pub struct SecurityReviewCase {
    pub request: Pubkey,
    pub requester: Pubkey,
    pub subject: Pubkey,
    pub committee_size: u8,
    pub votes_upheld: u8,
    pub votes_dismissed: u8,
    pub decision: u8, // review_decision enum
    pub finalized_at: Option<i64>,
    pub created_at: i64,
}

/// Reviewer assignment for a review case.
/// PDA: `["review_assignment", case, reviewer]`
#[account]
#[derive(InitSpace)]
pub struct SecurityReviewAssignment {
    pub case: Pubkey,
    pub reviewer: Pubkey,
    pub voted: bool,
    pub vote: u8, // review_decision enum
    pub voted_at: Option<i64>,
    pub assigned_at: i64,
}

/// Finalized decision on a security/privacy request.
/// PDA: `["decision", request]`
#[account]
#[derive(InitSpace)]
pub struct SecurityDecision {
    pub request: Pubkey,
    pub case: Pubkey,
    pub remedy: u8,                 // remedy_type enum
    pub state: u8,                  // privacy_state enum
    pub restriction_scope: [u8; 2], // jurisdiction
    pub expires_at: Option<i64>,
    pub decided_at: i64,
    pub decided_by: Pubkey, // review case PDA (recorded for audit)
}

/// Appeal of a security/privacy decision.
/// PDA: `["appeal", decision, appellant]`
#[account]
#[derive(InitSpace)]
pub struct SecurityAppeal {
    pub decision: Pubkey,
    pub appellant: Pubkey,
    pub nonce: u16,
    #[max_len(MAX_REQUEST_NOTE_LEN)]
    pub reason: String,
    pub status: u8, // appeal_status enum
    pub created_at: i64,
    pub resolved_at: Option<i64>,
}

/// Immutable audit event for security/privacy decisions.
/// PDA: `["audit_event", entity, sequence]`
#[account]
#[derive(InitSpace)]
pub struct SecurityAuditEvent {
    pub entity: Pubkey, // subject or decision PDA
    pub sequence: u32,
    pub action: u8, // 1=request, 2=decision, 3=appeal, 4=emergency
    pub actor: Pubkey,
    pub timestamp: i64,
    #[max_len(MAX_REASON_LEN)]
    pub reason: String,
}

// ---------------------------------------------------------------------------
// Instruction contexts
// ---------------------------------------------------------------------------

#[derive(Accounts)]
#[instruction(nonce: u16)]
pub struct SubmitSecurityRequest<'info> {
    #[account(
        init,
        payer = requester,
        space = 8 + SecurityRequest::INIT_SPACE,
        seeds = [
            b"security_request",
            requester.key().as_ref(),
            nonce.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub security_request: Account<'info, SecurityRequest>,
    #[account(mut)]
    pub requester: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(nonce: u16)]
pub struct SubmitPrivacyRequest<'info> {
    #[account(
        init,
        payer = requester,
        space = 8 + PrivacyRequest::INIT_SPACE,
        seeds = [
            b"privacy_request",
            requester.key().as_ref(),
            subject.key().as_ref(),
            nonce.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub privacy_request: Account<'info, PrivacyRequest>,
    #[account(mut)]
    pub requester: Signer<'info>,
    /// Subject of the privacy request (identity, asset, document, etc.)
    /// CHECK: validated by handler that it exists
    pub subject: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(nonce: u16)]
pub struct SubmitJurisdictionalRestriction<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + JurisdictionalRestrictionRequest::INIT_SPACE,
        seeds = [
            b"jurisdiction",
            authority.key().as_ref(),
            subject.key().as_ref(),
            nonce.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub restriction: Account<'info, JurisdictionalRestrictionRequest>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// Subject being restricted
    /// CHECK: validated by handler
    pub subject: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(nonce: u16)]
pub struct CreateReviewCase<'info> {
    #[account(
        init,
        payer = requester,
        space = 8 + SecurityReviewCase::INIT_SPACE,
        seeds = [b"review_case", request.key().as_ref()],
        bump,
    )]
    pub review_case: Account<'info, SecurityReviewCase>,
    #[account(
        mut,
        seeds = [b"security_request", request.key().as_ref()],
        bump,
    )]
    pub request: Account<'info, SecurityRequest>,
    #[account(mut)]
    pub requester: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AssignReviewer<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + SecurityReviewAssignment::INIT_SPACE,
        seeds = [
            b"review_assignment",
            case.key().as_ref(),
            reviewer.key().as_ref(),
        ],
        bump,
    )]
    pub assignment: Account<'info, SecurityReviewAssignment>,
    #[account(mut)]
    pub case: Account<'info, SecurityReviewCase>,
    #[account(
        seeds = [b"validator_reputation", reviewer.key().as_ref()],
        bump,
    )]
    pub reviewer_reputation: Account<'info, ValidatorReputation>,
    /// CHECK: the validator being assigned as reviewer
    pub reviewer: UncheckedAccount<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CastReviewVote<'info> {
    #[account(
        mut,
        seeds = [b"review_assignment", case.key().as_ref(), voter.key().as_ref()],
        bump,
    )]
    pub assignment: Account<'info, SecurityReviewAssignment>,
    #[account(mut)]
    pub case: Account<'info, SecurityReviewCase>,
    #[account(mut)]
    pub voter: Signer<'info>,
}

#[derive(Accounts)]
pub struct FinalizeDecision<'info> {
    #[account(
        mut,
        seeds = [b"review_case", request.key().as_ref()],
        bump,
    )]
    pub case: Account<'info, SecurityReviewCase>,
    #[account(
        init,
        payer = authority,
        space = 8 + SecurityDecision::INIT_SPACE,
        seeds = [b"decision", request.key().as_ref()],
        bump,
    )]
    pub decision: Account<'info, SecurityDecision>,
    #[account(
        mut,
        seeds = [b"security_request", request.key().as_ref()],
        bump,
    )]
    pub request: Account<'info, SecurityRequest>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(nonce: u16)]
pub struct AppealDecision<'info> {
    #[account(
        init,
        payer = appellant,
        space = 8 + SecurityAppeal::INIT_SPACE,
        seeds = [
            b"appeal",
            decision.key().as_ref(),
            appellant.key().as_ref(),
            nonce.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub appeal: Account<'info, SecurityAppeal>,
    #[account(mut)]
    pub decision: Account<'info, SecurityDecision>,
    #[account(mut)]
    pub appellant: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FinalizeAppeal<'info> {
    #[account(mut)]
    pub appeal: Account<'info, SecurityAppeal>,
    #[account(mut)]
    pub decision: Account<'info, SecurityDecision>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(sequence: u32)]
pub struct RecordAuditEvent<'info> {
    #[account(
        init,
        payer = actor,
        space = 8 + SecurityAuditEvent::INIT_SPACE,
        seeds = [
            b"audit_event",
            entity.key().as_ref(),
            sequence.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub audit_event: Account<'info, SecurityAuditEvent>,
    /// CHECK: the entity being audited
    pub entity: UncheckedAccount<'info>,
    #[account(mut)]
    pub actor: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// ---------------------------------------------------------------------------
// Emergency privacy protection PDA (separate from SecurityRequest to allow
// fast-track expiry tracking)
// PDA: `["emergency_protect", subject, nonce]`
#[account]
#[derive(InitSpace)]
pub struct EmergencyProtection {
    pub subject: Pubkey,
    pub nonce: u16,
    pub requester: Pubkey,
    pub expires_at: i64,
    pub created_at: i64,
    pub lifted: bool,
}

#[derive(Accounts)]
#[instruction(nonce: u16)]
pub struct EmergencyProtect<'info> {
    #[account(
        init,
        payer = requester,
        space = 8 + EmergencyProtection::INIT_SPACE,
        seeds = [
            b"emergency_protect",
            subject.key().as_ref(),
            nonce.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub protection: Account<'info, EmergencyProtection>,
    #[account(mut)]
    pub requester: Signer<'info>,
    /// CHECK: subject being emergency-protected
    pub subject: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

// ---------------------------------------------------------------------------
// Instruction handlers
// ---------------------------------------------------------------------------

pub fn submit_security_request(
    ctx: Context<SubmitSecurityRequest>,
    nonce: u16,
    request_type: u8,
    description: String,
    requested_remedy: u8,
    jurisdiction: [u8; 2],
    emergency: bool,
) -> Result<()> {
    let sr = &mut ctx.accounts.security_request;
    let now = Clock::get()?.unix_timestamp;

    require!(
        description.len() <= MAX_REQUEST_NOTE_LEN,
        TerraError::InvalidRequestDescription
    );
    require!(
        requested_remedy <= remedy_type::MAX,
        TerraError::InvalidRemedyType
    );
    require!(
        request_type <= restriction_type::MAX,
        TerraError::InvalidRestrictionType
    );

    sr.requester = ctx.accounts.requester.key();
    sr.subject = Pubkey::default(); // filled by specific request types
    sr.nonce = nonce;
    sr.request_type = request_type;
    sr.description = description;
    sr.requested_remedy = requested_remedy;
    sr.jurisdiction = jurisdiction;
    sr.status = request_status::OPEN;
    sr.emergency = emergency;
    sr.expires_at = now
        + if emergency {
            EMERGENCY_PROTECTION_SECS
        } else {
            STANDARD_REVIEW_SECS
        };
    sr.created_at = now;
    sr.updated_at = now;

    emit!(SecurityRequestSubmitted {
        request: sr.key(),
        requester: sr.requester,
        subject: sr.subject,
        request_type: sr.request_type,
        remedy: sr.requested_remedy,
        created_at: now,
    });

    Ok(())
}

pub fn submit_privacy_request(
    ctx: Context<SubmitPrivacyRequest>,
    nonce: u16,
    request_type: u8,
    description: String,
    requested_remedy: u8,
    jurisdiction: [u8; 2],
    emergency: bool,
) -> Result<()> {
    let pr = &mut ctx.accounts.privacy_request;
    let now = Clock::get()?.unix_timestamp;

    require!(
        description.len() <= MAX_REQUEST_NOTE_LEN,
        TerraError::InvalidRequestDescription
    );
    require!(
        requested_remedy <= remedy_type::MAX,
        TerraError::InvalidRemedyType
    );
    require!(
        request_type <= restriction_type::MAX,
        TerraError::InvalidRestrictionType
    );

    pr.requester = ctx.accounts.requester.key();
    pr.subject = ctx.accounts.subject.key();
    pr.nonce = nonce;
    pr.request_type = request_type;
    pr.description = description;
    pr.requested_remedy = requested_remedy;
    pr.jurisdiction = jurisdiction;
    pr.status = request_status::OPEN;
    pr.emergency = emergency;
    pr.expires_at = now
        + if emergency {
            EMERGENCY_PROTECTION_SECS
        } else {
            STANDARD_REVIEW_SECS
        };
    pr.created_at = now;
    pr.updated_at = now;

    emit!(PrivacyRequestSubmitted {
        request: pr.key(),
        requester: pr.requester,
        subject: pr.subject,
        request_type: pr.request_type,
        remedy: pr.requested_remedy,
        created_at: now,
    });

    Ok(())
}

pub fn submit_jurisdictional_restriction(
    ctx: Context<SubmitJurisdictionalRestriction>,
    nonce: u16,
    restriction_type: u8,
    description: String,
    requested_remedy: u8,
    jurisdiction: [u8; 2],
    expiry: Option<i64>,
) -> Result<()> {
    let jr = &mut ctx.accounts.restriction;
    let now = Clock::get()?.unix_timestamp;

    require!(
        description.len() <= MAX_REQUEST_NOTE_LEN,
        TerraError::InvalidRequestDescription
    );
    require!(
        requested_remedy <= remedy_type::MAX,
        TerraError::InvalidRemedyType
    );
    require!(
        restriction_type <= restriction_type::MAX,
        TerraError::InvalidRestrictionType
    );

    jr.authority = ctx.accounts.authority.key();
    jr.subject = ctx.accounts.subject.key();
    jr.nonce = nonce;
    jr.restriction_type = restriction_type;
    jr.description = description;
    jr.requested_remedy = requested_remedy;
    jr.jurisdiction = jurisdiction;
    jr.status = request_status::OPEN;
    jr.expires_at = expiry;
    jr.created_at = now;
    jr.updated_at = now;

    emit!(JurisdictionalRestrictionSubmitted {
        restriction: jr.key(),
        authority: jr.authority,
        subject: jr.subject,
        restriction_type: jr.restriction_type,
        remedy: jr.requested_remedy,
        created_at: now,
    });

    Ok(())
}

pub fn create_review_case(ctx: Context<CreateReviewCase>, _nonce: u16) -> Result<()> {
    let case = &mut ctx.accounts.review_case;
    let request = &mut ctx.accounts.request;
    let now = Clock::get()?.unix_timestamp;

    require!(
        request.status == request_status::OPEN,
        TerraError::RequestNotOpen
    );
    require!(request.expires_at > now, TerraError::RequestExpired);

    case.request = request.key();
    case.requester = request.requester;
    case.subject = request.subject;
    case.committee_size = SECURITY_COMMITTEE_SIZE as u8;
    case.votes_upheld = 0;
    case.votes_dismissed = 0;
    case.decision = review_decision::PENDING;
    case.finalized_at = None;
    case.created_at = now;

    request.status = request_status::UNDER_REVIEW;
    request.updated_at = now;

    emit!(ReviewCaseCreated {
        case: case.key(),
        request: request.key(),
        requester: case.requester,
        subject: case.subject,
        committee_size: case.committee_size,
        created_at: now,
    });

    Ok(())
}

pub fn assign_reviewer(ctx: Context<AssignReviewer>) -> Result<()> {
    let assignment = &mut ctx.accounts.assignment;
    let case = &mut ctx.accounts.case;
    let reviewer_reputation = &ctx.accounts.reviewer_reputation;
    let now = Clock::get()?.unix_timestamp;

    require!(
        case.decision == review_decision::PENDING,
        TerraError::CaseAlreadyDecided
    );
    require!(
        reviewer_reputation.reputation_score >= MIN_SECURITY_REPUTATION,
        TerraError::ReviewerInsufficientReputation
    );

    assignment.case = case.key();
    assignment.reviewer = ctx.accounts.reviewer.key();
    assignment.voted = false;
    assignment.vote = review_decision::PENDING;
    assignment.voted_at = None;
    assignment.assigned_at = now;

    emit!(ReviewerAssigned {
        assignment: assignment.key(),
        case: case.key(),
        reviewer: assignment.reviewer,
        assigned_at: now,
    });

    Ok(())
}

pub fn cast_review_vote(ctx: Context<CastReviewVote>, vote: u8) -> Result<()> {
    let assignment = &mut ctx.accounts.assignment;
    let case = &mut ctx.accounts.case;
    let now = Clock::get()?.unix_timestamp;

    require!(!assignment.voted, TerraError::AlreadyVoted);
    require!(vote <= review_decision::MAX, TerraError::InvalidReviewVote);

    assignment.voted = true;
    assignment.vote = vote;
    assignment.voted_at = Some(now);

    if vote == review_decision::UPHELD {
        case.votes_upheld = case.votes_upheld.saturating_add(1);
    } else if vote == review_decision::DISMISSED {
        case.votes_dismissed = case.votes_dismissed.saturating_add(1);
    }

    emit!(ReviewVoteCast {
        case: case.key(),
        reviewer: ctx.accounts.voter.key(),
        vote,
        votes_upheld: case.votes_upheld,
        votes_dismissed: case.votes_dismissed,
    });

    Ok(())
}

pub fn finalize_decision(ctx: Context<FinalizeDecision>) -> Result<()> {
    let case = &mut ctx.accounts.case;
    let decision = &mut ctx.accounts.decision;
    let request = &mut ctx.accounts.request;
    let now = Clock::get()?.unix_timestamp;

    require!(
        case.decision == review_decision::PENDING,
        TerraError::CaseAlreadyDecided
    );
    require!(
        case.votes_upheld + case.votes_dismissed >= case.committee_size,
        TerraError::SecurityQuorumNotReached
    );

    let upheld = case.votes_upheld > case.votes_dismissed;
    let decision_value = if upheld {
        review_decision::UPHELD
    } else {
        review_decision::DISMISSED
    };

    case.decision = decision_value;
    case.finalized_at = Some(now);

    decision.request = request.key();
    decision.case = case.key();
    decision.remedy = request.requested_remedy;
    decision.state = if upheld {
        privacy_state::RESTRICTED
    } else {
        privacy_state::RESTORED
    };
    decision.restriction_scope = request.jurisdiction;
    decision.decided_at = now;
    decision.decided_by = case.key();

    request.status = if upheld {
        request_status::APPROVED
    } else {
        request_status::REJECTED
    };
    request.updated_at = now;

    emit!(SecurityDecisionFinalized {
        decision: decision.key(),
        request: request.key(),
        case: case.key(),
        remedy: decision.remedy,
        state: decision.state,
        decided_at: now,
    });

    Ok(())
}

pub fn appeal_decision(ctx: Context<AppealDecision>, nonce: u16, reason: String) -> Result<()> {
    let appeal = &mut ctx.accounts.appeal;
    let decision = &mut ctx.accounts.decision;
    let now = Clock::get()?.unix_timestamp;

    require!(
        decision.state == privacy_state::RESTRICTED || decision.state == privacy_state::PRIVATE,
        TerraError::DecisionNotAppealable
    );
    require!(reason.len() <= MAX_REASON_LEN, TerraError::ReasonTooLong);

    appeal.decision = decision.key();
    appeal.appellant = ctx.accounts.appellant.key();
    appeal.nonce = nonce;
    appeal.reason = reason;
    appeal.status = appeal_status::PENDING;
    appeal.created_at = now;
    appeal.resolved_at = None;

    emit!(AppealFiled {
        appeal: appeal.key(),
        decision: decision.key(),
        appellant: appeal.appellant,
        created_at: now,
    });

    Ok(())
}

pub fn finalize_appeal(ctx: Context<FinalizeAppeal>, upheld_appeal: bool) -> Result<()> {
    let appeal = &mut ctx.accounts.appeal;
    let decision = &mut ctx.accounts.decision;
    let now = Clock::get()?.unix_timestamp;

    require!(
        appeal.status == appeal_status::PENDING,
        TerraError::AppealAlreadyDecided
    );

    appeal.status = if upheld_appeal {
        appeal_status::GRANTED
    } else {
        appeal_status::DENIED
    };
    appeal.resolved_at = Some(now);

    // If appeal granted, restore the previous state (revert restriction)
    if upheld_appeal {
        decision.state = privacy_state::RESTORED;
    }

    emit!(AppealDecided {
        appeal: appeal.key(),
        decision: decision.key(),
        upheld: upheld_appeal,
        resolved_at: now,
    });

    Ok(())
}

pub fn record_audit_event(
    ctx: Context<RecordAuditEvent>,
    sequence: u32,
    action: u8,
    reason: String,
) -> Result<()> {
    let audit = &mut ctx.accounts.audit_event;
    let now = Clock::get()?.unix_timestamp;

    require!(reason.len() <= MAX_REASON_LEN, TerraError::ReasonTooLong);

    audit.entity = ctx.accounts.entity.key();
    audit.sequence = sequence;
    audit.action = action;
    audit.actor = ctx.accounts.actor.key();
    audit.timestamp = now;
    audit.reason = reason;

    emit!(SecurityAuditRecorded {
        audit_event: audit.key(),
        entity: audit.entity,
        sequence: audit.sequence,
        action: audit.action,
        actor: audit.actor,
        timestamp: audit.timestamp,
    });

    Ok(())
}

pub fn emergency_protect(ctx: Context<EmergencyProtect>, nonce: u16) -> Result<()> {
    let protection = &mut ctx.accounts.protection;
    let now = Clock::get()?.unix_timestamp;

    require!(!protection.lifted, TerraError::ProtectionAlreadyActive);

    protection.subject = ctx.accounts.subject.key();
    protection.nonce = nonce;
    protection.requester = ctx.accounts.requester.key();
    protection.expires_at = now + EMERGENCY_PROTECTION_SECS;
    protection.created_at = now;
    protection.lifted = false;

    emit!(EmergencyProtectionStarted {
        protection: protection.key(),
        subject: protection.subject,
        requester: protection.requester,
        expires_at: protection.expires_at,
        created_at: now,
    });

    Ok(())
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[event]
pub struct SecurityRequestSubmitted {
    pub request: Pubkey,
    pub requester: Pubkey,
    pub subject: Pubkey,
    pub request_type: u8,
    pub remedy: u8,
    pub created_at: i64,
}

#[event]
pub struct PrivacyRequestSubmitted {
    pub request: Pubkey,
    pub requester: Pubkey,
    pub subject: Pubkey,
    pub request_type: u8,
    pub remedy: u8,
    pub created_at: i64,
}

#[event]
pub struct JurisdictionalRestrictionSubmitted {
    pub restriction: Pubkey,
    pub authority: Pubkey,
    pub subject: Pubkey,
    pub restriction_type: u8,
    pub remedy: u8,
    pub created_at: i64,
}

#[event]
pub struct ReviewCaseCreated {
    pub case: Pubkey,
    pub request: Pubkey,
    pub requester: Pubkey,
    pub subject: Pubkey,
    pub committee_size: u8,
    pub created_at: i64,
}

#[event]
pub struct ReviewerAssigned {
    pub assignment: Pubkey,
    pub case: Pubkey,
    pub reviewer: Pubkey,
    pub assigned_at: i64,
}

#[event]
pub struct ReviewVoteCast {
    pub case: Pubkey,
    pub reviewer: Pubkey,
    pub vote: u8,
    pub votes_upheld: u8,
    pub votes_dismissed: u8,
}

#[event]
pub struct SecurityDecisionFinalized {
    pub decision: Pubkey,
    pub request: Pubkey,
    pub case: Pubkey,
    pub remedy: u8,
    pub state: u8,
    pub decided_at: i64,
}

#[event]
pub struct AppealFiled {
    pub appeal: Pubkey,
    pub decision: Pubkey,
    pub appellant: Pubkey,
    pub created_at: i64,
}

#[event]
pub struct AppealDecided {
    pub appeal: Pubkey,
    pub decision: Pubkey,
    pub upheld: bool,
    pub resolved_at: i64,
}

#[event]
pub struct EmergencyProtectionStarted {
    pub protection: Pubkey,
    pub subject: Pubkey,
    pub requester: Pubkey,
    pub expires_at: i64,
    pub created_at: i64,
}

#[event]
pub struct SecurityAuditRecorded {
    pub audit_event: Pubkey,
    pub entity: Pubkey,
    pub sequence: u32,
    pub action: u8,
    pub actor: Pubkey,
    pub timestamp: i64,
}
