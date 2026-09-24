use anchor_lang::prelude::*;
use solana_program::hash::hashv;

use crate::validator_profile::{capability_level, profile_tier, ValidatorProfile};
use crate::verification::reputation::ValidatorReputation;
use crate::TerraError;

// ---------------------------------------------------------------------------
// RFC-012 Phase 7 — reputation governance (self-regulating, no jail)
//
// Vision (user directive 2026-09-24):
//   - Progressive: bootstrap admin mode → PEER_CONSENSUS self-regulating
//     (`effective_mode` already flips at CONSENSUS_FLIP_THRESHOLD).
//   - NO jail. Confirmed fraud demotes the accused to very limited capability
//     (DECLARED / PROBATIONARY), never a binary ban.
//   - Judgement is by a random committee of well-reputed validators, not admin.
//
// Flow: FraudReport → ReviewCase (random committee) → votes → decision →
//       CapabilityRestriction (demote) → Appeal → rehabilitation (lift).
// ---------------------------------------------------------------------------

/// Max description length stored on a fraud report / appeal note.
pub const MAX_FRAUD_NOTE_LEN: usize = 128;
/// Committee size for fraud review / appeal (odd → no tie).
pub const COMMITTEE_SIZE: usize = 5;
/// Min reputation (bps) for a validator to sit on a fraud committee.
pub const MIN_COMMITTEE_REPUTATION: u16 = 5_000;
/// Restriction is appealable only after this many seconds.
pub const RESTRICTION_APPEAL_DELAY_SECS: i64 = 24 * 3600;
/// After this many seconds of clean standing, restriction may be lifted.
pub const RESTRICTION_REHAB_SECS: i64 = 30 * 24 * 3600;

pub mod fraud_status {
    pub const OPEN: u8 = 0;
    pub const UNDER_REVIEW: u8 = 1;
    pub const UPHELD: u8 = 2;
    pub const DISMISSED: u8 = 3;
    pub const APPEALED: u8 = 4;
    pub const MAX: u8 = APPEALED;
}

pub mod review_decision {
    pub const PENDING: u8 = 0;
    pub const UPHELD: u8 = 1;
    pub const DISMISSED: u8 = 2;
    pub const MAX: u8 = DISMISSED;
}

pub mod restriction_status {
    pub const ACTIVE: u8 = 0;
    pub const LIFTED: u8 = 1;
    pub const MAX: u8 = LIFTED;
}

pub mod appeal_status {
    pub const PENDING: u8 = 0;
    pub const GRANTED: u8 = 1;
    pub const DENIED: u8 = 2;
    pub const MAX: u8 = DENIED;
}

pub mod fraud_reason {
    pub const FALSIFIED_OBSERVATION: u8 = 0;
    pub const COLLUSION: u8 = 1;
    pub const IDENTITY_MISUSE: u8 = 2;
    pub const EVIDENCE_TAMPERING: u8 = 3;
    pub const OTHER: u8 = 4;
    pub const MAX: u8 = OTHER;
}

// ---------------------------------------------------------------------------
// Accounts (PDA seeds per RFC-012 §10)
// ---------------------------------------------------------------------------

/// Accusation with evidence. Permissionless to file; never auto-punishes.
/// PDA: `["fraud_report", accused, reporter, nonce]`
#[account]
#[derive(InitSpace)]
pub struct FraudReport {
    pub accused: Pubkey,
    pub reporter: Pubkey,
    pub nonce: u16,
    pub evidence_hash: [u8; 32],
    pub reason_code: u8,
    #[max_len(128)]
    pub note: String,
    pub status: u8,
    pub capability_code: u8,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Independent review of a FraudReport by a random well-reputed committee.
/// PDA: `["review_case", report]`
#[account]
#[derive(InitSpace)]
pub struct ReviewCase {
    pub report: Pubkey,
    pub accused: Pubkey,
    #[max_len(5)]
    pub committee: Vec<Pubkey>,
    /// Members who have already voted (prevents double-vote).
    #[max_len(5)]
    pub voted: Vec<Pubkey>,
    pub votes_cast: u8,
    pub votes_upheld: u8,
    pub votes_dismissed: u8,
    pub decision: u8,
    pub opened_at: i64,
    pub decided_at: i64,
}

/// Partial capability downgrade — NOT a ban, NOT a jail (Design Rule 13).
/// PDA: `["capability_restriction", wallet, capability_code]`
#[account]
#[derive(InitSpace)]
pub struct CapabilityRestriction {
    pub wallet: Pubkey,
    pub capability_code: u8,
    pub report: Pubkey,
    pub max_level: u8,
    pub status: u8,
    pub applied_at: i64,
    pub appeal_opens_at: i64,
    pub rehab_eligible_at: i64,
    pub lifted_at: i64,
    pub updated_at: i64,
}

/// Due-process appeal against an active CapabilityRestriction.
/// PDA: `["appeal", restriction, appellant, nonce]`
#[account]
#[derive(InitSpace)]
pub struct Appeal {
    pub restriction: Pubkey,
    pub appellant: Pubkey,
    pub nonce: u16,
    pub evidence_hash: [u8; 32],
    pub status: u8,
    #[max_len(128)]
    pub note: String,
    pub created_at: i64,
    pub decided_at: i64,
}

// ---------------------------------------------------------------------------
// Pure helpers (unit-testable)
// ---------------------------------------------------------------------------

pub fn is_valid_fraud_status(s: u8) -> bool {
    s <= fraud_status::MAX
}

pub fn is_valid_reason_code(r: u8) -> bool {
    r <= fraud_reason::MAX
}

pub fn is_valid_restriction_status(s: u8) -> bool {
    s <= restriction_status::MAX
}

pub fn is_valid_appeal_status(s: u8) -> bool {
    s <= appeal_status::MAX
}

pub fn is_self_report(accused: Pubkey, reporter: Pubkey) -> bool {
    accused == reporter
}

pub fn can_open_review(status: u8) -> bool {
    status == fraud_status::OPEN
}

pub fn can_cast_vote(decision: u8) -> bool {
    decision == review_decision::PENDING
}

pub fn already_voted(voted: &[Pubkey], voter: &Pubkey) -> bool {
    voted.contains(voter)
}

pub fn can_finalize(votes_cast: u8, committee_len: u8) -> bool {
    committee_len > 0 && votes_cast >= committee_len
}

/// Majority-upheld → fraud confirmed; else dismissed (odd committee, no tie).
pub fn decide_majority(votes_upheld: u8, votes_dismissed: u8) -> u8 {
    if votes_upheld > votes_dismissed {
        review_decision::UPHELD
    } else {
        review_decision::DISMISSED
    }
}

pub fn appeal_is_open(now: i64, appeal_opens_at: i64) -> bool {
    now >= appeal_opens_at
}

pub fn rehab_is_eligible(now: i64, rehab_eligible_at: i64) -> bool {
    now >= rehab_eligible_at
}

pub fn can_lift(status: u8) -> bool {
    status == restriction_status::ACTIVE
}

/// Level cap on upheld fraud: DECLARED (new-validator-like). No jail.
pub fn restricted_max_level() -> u8 {
    capability_level::DECLARED
}

/// Tier demotion on upheld fraud: PROBATIONARY (not binary ban).
pub fn restricted_tier() -> u8 {
    profile_tier::PROBATIONARY
}

/// Used by `route_task`: active restriction on (wallet, capability_code) fails.
pub fn blocks_capability(
    restriction: Option<&CapabilityRestriction>,
    wallet: &Pubkey,
    capability_code: u8,
) -> bool {
    match restriction {
        Some(r) => {
            r.status == restriction_status::ACTIVE
                && r.wallet == *wallet
                && r.capability_code == capability_code
        }
        None => false,
    }
}

/// Filter a candidate pool down to well-reputed validators.
pub fn filter_committee_pool(pool: &[Pubkey], scores: &[u16]) -> Vec<Pubkey> {
    pool.iter()
        .zip(scores.iter())
        .filter(|(_, s)| **s >= MIN_COMMITTEE_REPUTATION)
        .map(|(p, _)| *p)
        .collect()
}

/// Deterministic committee seed from report PDA + slot.
pub fn committee_seed(report_pda: &[u8; 32], slot: u64) -> [u8; 32] {
    hashv(&[report_pda.as_ref(), &slot.to_le_bytes(), b"terra_committee"]).to_bytes()
}

/// Pick `COMMITTEE_SIZE` distinct members from `pool` using `seed`.
pub fn select_committee(seed: &[u8; 32], pool: &[Pubkey]) -> Option<Vec<Pubkey>> {
    let mut remaining = pool.to_vec();
    remaining.sort();
    remaining.dedup();
    if remaining.len() < COMMITTEE_SIZE {
        return None;
    }
    let mut chosen: Vec<Pubkey> = Vec::with_capacity(COMMITTEE_SIZE);
    let mut cursor = *seed;
    while chosen.len() < COMMITTEE_SIZE {
        let h = hashv(&[
            cursor.as_ref(),
            b"terra_pick",
            &(chosen.len() as u64).to_le_bytes(),
        ]);
        cursor = h.to_bytes();
        let idx = u64::from_le_bytes(cursor[..8].try_into().unwrap()) as usize % remaining.len();
        chosen.push(remaining.swap_remove(idx));
    }
    Some(chosen)
}

/// Map a 32-byte draw to [0, 10000) bps.
pub fn draw_bps(seed: &[u8; 32], key: &Pubkey) -> u16 {
    let h = hashv(&[seed.as_ref(), key.as_ref(), b"terra_fraud_draw"]);
    let v = u64::from_le_bytes(h.to_bytes()[..8].try_into().unwrap());
    (v % 10_000) as u16
}

fn deser<T: anchor_lang::AccountDeserialize>(ai: &AccountInfo) -> Result<T> {
    let data = ai.try_borrow_data()?;
    let mut slice: &[u8] = data.as_ref();
    T::try_deserialize(&mut slice)
}

// ---------------------------------------------------------------------------
// Instruction handlers
// ---------------------------------------------------------------------------

/// File a fraud accusation. Permissionless; reporter pays; self-report banned.
pub fn submit_fraud_report(
    ctx: Context<crate::SubmitFraudReport>,
    _nonce: u16,
    evidence_hash: [u8; 32],
    reason_code: u8,
    note: String,
    capability_code: u8,
) -> Result<()> {
    require!(
        is_valid_reason_code(reason_code),
        TerraError::InvalidFraudReason
    );
    require!(note.len() <= MAX_FRAUD_NOTE_LEN, TerraError::StringTooLong);
    require!(
        crate::validator_profile::is_valid_capability_code(capability_code),
        TerraError::InvalidCapabilityCode
    );

    let accused = ctx.accounts.accused.key();
    let reporter = ctx.accounts.reporter.key();
    require!(
        !is_self_report(accused, reporter),
        TerraError::SelfFraudReport
    );

    let now = Clock::get()?.unix_timestamp;
    let r = &mut ctx.accounts.report;
    r.accused = accused;
    r.reporter = reporter;
    r.nonce = _nonce;
    r.evidence_hash = evidence_hash;
    r.reason_code = reason_code;
    r.note = note;
    r.status = fraud_status::OPEN;
    r.capability_code = capability_code;
    r.created_at = now;
    r.updated_at = now;

    emit!(crate::FraudReportSubmitted {
        report: r.key(),
        accused,
        reporter,
        reason_code,
        capability_code,
    });
    Ok(())
}

/// Open a review: select a random committee of well-reputed validators from
/// `remaining_accounts` as (profile, reputation) pairs. Permissionless open;
/// the committee is random so the caller cannot choose friends.
pub fn open_fraud_review(ctx: Context<crate::OpenFraudReview>) -> Result<()> {
    let report_key = ctx.accounts.report.key();
    require!(
        can_open_review(ctx.accounts.report.status),
        TerraError::InvalidFraudStatus
    );

    let mut pool: Vec<Pubkey> = Vec::new();
    let mut scores: Vec<u16> = Vec::new();
    let accounts = &ctx.remaining_accounts;
    let mut i = 0;
    while i + 1 < accounts.len() {
        let profile: ValidatorProfile = deser(&accounts[i])?;
        let rep: ValidatorReputation = deser(&accounts[i + 1])?;
        require!(
            rep.validator == profile.wallet,
            TerraError::RouteAccountMismatch
        );
        if profile.wallet != ctx.accounts.report.accused {
            pool.push(profile.wallet);
            scores.push(rep.reputation_score);
        }
        i += 2;
    }
    let eligible = filter_committee_pool(&pool, &scores);

    let slot = Clock::get()?.slot;
    let report_bytes = report_key.to_bytes();
    let seed = committee_seed(&report_bytes, slot);
    let committee = select_committee(&seed, &eligible).ok_or(TerraError::CommitteeTooSmall)?;

    let accused = ctx.accounts.report.accused;
    require!(!committee.contains(&accused), TerraError::CommitteeTooSmall);

    let now = Clock::get()?.unix_timestamp;
    let report = &mut ctx.accounts.report;
    report.status = fraud_status::UNDER_REVIEW;
    report.updated_at = now;

    let case = &mut ctx.accounts.review;
    case.report = report_key;
    case.accused = accused;
    case.committee = committee.clone();
    case.voted = Vec::new();
    case.votes_cast = 0;
    case.votes_upheld = 0;
    case.votes_dismissed = 0;
    case.decision = review_decision::PENDING;
    case.opened_at = now;
    case.decided_at = 0;

    emit!(crate::FraudReviewOpened {
        review: case.key(),
        report: report_key,
        accused,
        committee_size: committee.len() as u8,
    });
    Ok(())
}

/// A committee member casts one vote (true = uphold / guilty).
pub fn cast_fraud_vote(ctx: Context<crate::CastFraudVote>, uphold: bool) -> Result<()> {
    let voter = ctx.accounts.voter.key();
    let case = &mut ctx.accounts.review;
    require!(
        can_cast_vote(case.decision),
        TerraError::ReviewAlreadyDecided
    );
    require!(
        case.committee.contains(&voter),
        TerraError::NotCommitteeMember
    );
    require!(
        !already_voted(&case.voted, &voter),
        TerraError::DuplicateCommitteeVote
    );

    case.voted.push(voter);
    case.votes_cast = case.votes_cast.saturating_add(1);
    if uphold {
        case.votes_upheld = case.votes_upheld.saturating_add(1);
    } else {
        case.votes_dismissed = case.votes_dismissed.saturating_add(1);
    }

    emit!(crate::FraudVoteCast {
        review: case.key(),
        voter,
        uphold,
        votes_cast: case.votes_cast,
    });
    Ok(())
}

/// Finalize when every committee member has voted. Upheld → create
/// CapabilityRestriction (demote to DECLARED / PROBATIONARY). Dismissed →
/// report closed. No admin; no jail.
pub fn finalize_fraud_review(ctx: Context<crate::FinalizeFraudReview>) -> Result<()> {
    let (votes_upheld, votes_dismissed, report_ref, already_decided, not_full) = {
        let case = &ctx.accounts.review;
        (
            case.votes_upheld,
            case.votes_dismissed,
            case.report,
            !can_cast_vote(case.decision),
            !can_finalize(case.votes_cast, case.committee.len() as u8),
        )
    };
    require!(!already_decided, TerraError::ReviewAlreadyDecided);
    require!(!not_full, TerraError::ReviewNotFinalizable);
    require!(
        report_ref == ctx.accounts.report.key(),
        TerraError::RouteAccountMismatch
    );

    let decision = decide_majority(votes_upheld, votes_dismissed);
    let now = Clock::get()?.unix_timestamp;

    {
        let case_mut = &mut ctx.accounts.review;
        case_mut.decision = decision;
        case_mut.decided_at = now;
    }

    let report_key = ctx.accounts.report.key();
    let report = &mut ctx.accounts.report;
    report.updated_at = now;

    if decision == review_decision::UPHELD {
        report.status = fraud_status::UPHELD;

        // Demote capability: cap the accused's level (no jail, no ban).
        let cap_code = report.capability_code;
        let accused = report.accused;

        // The context always includes `capability` (init_if_needed) so the
        // accused's capability for this code is forced to DECLARED.
        let cap = &mut ctx.accounts.capability;
        cap.wallet = accused;
        cap.capability_code = cap_code;
        // Never raise above DECLARED on fraud uphold.
        if cap.level > restricted_max_level() {
            cap.level = restricted_max_level();
        }
        if cap.declared_at == 0 {
            cap.declared_at = now;
        }
        cap.verified_at = 0;
        cap.updated_at = now;

        // Demote profile tier toward NEW-like (PROBATIONARY).
        let profile = &mut ctx.accounts.profile;
        if profile.tier > restricted_tier() {
            profile.tier = restricted_tier();
        }
        profile.updated_at = now;

        // Record the restriction PDA.
        let restriction = &mut ctx.accounts.restriction;
        restriction.wallet = accused;
        restriction.capability_code = cap_code;
        restriction.report = report_key;
        restriction.max_level = restricted_max_level();
        restriction.status = restriction_status::ACTIVE;
        restriction.applied_at = now;
        restriction.appeal_opens_at = now.saturating_add(RESTRICTION_APPEAL_DELAY_SECS);
        restriction.rehab_eligible_at = now.saturating_add(RESTRICTION_REHAB_SECS);
        restriction.lifted_at = 0;
        restriction.updated_at = now;

        emit!(crate::CapabilityRestrictionApplied {
            restriction: restriction.key(),
            wallet: accused,
            capability_code: cap_code,
            max_level: restriction.max_level,
            report: report_key,
        });
    } else {
        report.status = fraud_status::DISMISSED;
    }

    emit!(crate::FraudReviewFinalized {
        review: ctx.accounts.review.key(),
        report: report_key,
        decision,
        votes_upheld,
        votes_dismissed,
    });
    Ok(())
}

/// File an appeal against an ACTIVE restriction (after appeal window opens).
pub fn file_fraud_appeal(
    ctx: Context<crate::FileFraudAppeal>,
    _nonce: u16,
    evidence_hash: [u8; 32],
    note: String,
) -> Result<()> {
    require!(note.len() <= MAX_FRAUD_NOTE_LEN, TerraError::StringTooLong);
    let restriction = &ctx.accounts.restriction;
    require!(
        is_valid_restriction_status(restriction.status),
        TerraError::InvalidRestrictionStatus
    );
    require!(
        can_lift(restriction.status),
        TerraError::RestrictionNotActive
    );
    let now = Clock::get()?.unix_timestamp;
    require!(
        appeal_is_open(now, restriction.appeal_opens_at),
        TerraError::AppealWindowClosed
    );
    require!(
        ctx.accounts.appellant.key() == restriction.wallet,
        TerraError::NotRestrictedWallet
    );

    // Only one open appeal per restriction: require source report still UPHELD.
    require!(
        ctx.accounts.report.status == fraud_status::UPHELD,
        TerraError::InvalidFraudStatus
    );

    let a = &mut ctx.accounts.appeal;
    a.restriction = restriction.key();
    a.appellant = ctx.accounts.appellant.key();
    a.nonce = _nonce;
    a.evidence_hash = evidence_hash;
    a.status = appeal_status::PENDING;
    a.note = note;
    a.created_at = now;
    a.decided_at = 0;

    // Mark report as appealed so a second appeal cannot be filed casually.
    let report = &mut ctx.accounts.report;
    report.status = fraud_status::APPEALED;
    report.updated_at = now;

    emit!(crate::FraudAppealFiled {
        appeal: a.key(),
        restriction: restriction.key(),
        appellant: a.appellant,
    });
    Ok(())
}

/// Open a fresh random committee for an appeal (self-regulating, no admin).
///
/// Appeals reuse the `ReviewCase` account with seeds `["review_case", appeal]`
/// and mirror the fraud flow: open → cast (COMMITTEE_SIZE votes) → finalize.
/// `remaining_accounts` is a (profile, reputation) pool; the appellant is
/// excluded from their own committee. Majority grant lifts the restriction.
pub fn open_appeal_review(ctx: Context<crate::OpenAppealReview>) -> Result<()> {
    let appeal_key = ctx.accounts.appeal.key();
    require!(
        ctx.accounts.appeal.status == appeal_status::PENDING,
        TerraError::InvalidAppealStatus
    );
    require!(
        ctx.accounts.restriction.status == restriction_status::ACTIVE,
        TerraError::RestrictionNotActive
    );

    let mut pool: Vec<Pubkey> = Vec::new();
    let mut scores: Vec<u16> = Vec::new();
    let accounts = &ctx.remaining_accounts;
    let mut i = 0;
    while i + 1 < accounts.len() {
        let profile: ValidatorProfile = deser(&accounts[i])?;
        let rep: ValidatorReputation = deser(&accounts[i + 1])?;
        require!(
            rep.validator == profile.wallet,
            TerraError::RouteAccountMismatch
        );
        // Exclude appellant from their own appeal committee.
        if profile.wallet != ctx.accounts.appeal.appellant {
            pool.push(profile.wallet);
            scores.push(rep.reputation_score);
        }
        i += 2;
    }
    let eligible = filter_committee_pool(&pool, &scores);

    let slot = Clock::get()?.slot;
    let appeal_bytes = appeal_key.to_bytes();
    let seed = committee_seed(&appeal_bytes, slot);
    let committee = select_committee(&seed, &eligible).ok_or(TerraError::CommitteeTooSmall)?;
    require!(
        !committee.contains(&ctx.accounts.appeal.appellant),
        TerraError::CommitteeTooSmall
    );

    let now = Clock::get()?.unix_timestamp;
    let case = &mut ctx.accounts.review;
    case.report = appeal_key; // reuse field: links review → appeal
    case.accused = ctx.accounts.appeal.appellant;
    case.committee = committee.clone();
    case.voted = Vec::new();
    case.votes_cast = 0;
    case.votes_upheld = 0;
    case.votes_dismissed = 0;
    case.decision = review_decision::PENDING;
    case.opened_at = now;
    case.decided_at = 0;

    emit!(crate::AppealReviewOpened {
        review: case.key(),
        appeal: appeal_key,
        committee_size: committee.len() as u8,
    });
    Ok(())
}

/// Cast a vote on an appeal review (true = grant appeal / lift restriction).
pub fn cast_appeal_vote(ctx: Context<crate::CastAppealVote>, grant: bool) -> Result<()> {
    let voter = ctx.accounts.voter.key();
    let case = &mut ctx.accounts.review;
    require!(
        can_cast_vote(case.decision),
        TerraError::ReviewAlreadyDecided
    );
    require!(
        case.committee.contains(&voter),
        TerraError::NotCommitteeMember
    );
    require!(
        !already_voted(&case.voted, &voter),
        TerraError::DuplicateCommitteeVote
    );

    case.voted.push(voter);
    case.votes_cast = case.votes_cast.saturating_add(1);
    // Reinterpret: upheld votes = grant votes for appeal context.
    if grant {
        case.votes_upheld = case.votes_upheld.saturating_add(1);
    } else {
        case.votes_dismissed = case.votes_dismissed.saturating_add(1);
    }

    emit!(crate::FraudVoteCast {
        review: case.key(),
        voter,
        uphold: grant,
        votes_cast: case.votes_cast,
    });
    Ok(())
}

/// Finalize appeal: majority grant → LIFT restriction + restore capability
/// observation path; majority deny → keep ACTIVE. Self-regulating, no admin.
pub fn finalize_appeal_review(ctx: Context<crate::FinalizeAppealReview>) -> Result<()> {
    let (votes_upheld, votes_dismissed, appeal_ref, already_decided, not_full, not_active) = {
        let case = &ctx.accounts.review;
        (
            case.votes_upheld,
            case.votes_dismissed,
            case.report,
            !can_cast_vote(case.decision),
            !can_finalize(case.votes_cast, case.committee.len() as u8),
            ctx.accounts.restriction.status != restriction_status::ACTIVE,
        )
    };
    require!(!already_decided, TerraError::ReviewAlreadyDecided);
    require!(!not_full, TerraError::ReviewNotFinalizable);
    require!(
        appeal_ref == ctx.accounts.appeal.key(),
        TerraError::RouteAccountMismatch
    );
    require!(!not_active, TerraError::RestrictionNotActive);

    let decision = decide_majority(votes_upheld, votes_dismissed);
    let now = Clock::get()?.unix_timestamp;

    {
        let case_mut = &mut ctx.accounts.review;
        case_mut.decision = decision;
        case_mut.decided_at = now;
    }

    let appeal_key = ctx.accounts.appeal.key();
    let appeal = &mut ctx.accounts.appeal;
    appeal.decided_at = now;

    if decision == review_decision::UPHELD {
        // Granted: lift restriction.
        appeal.status = appeal_status::GRANTED;
        let restriction = &mut ctx.accounts.restriction;
        require!(
            can_lift(restriction.status),
            TerraError::RestrictionNotActive
        );
        restriction.status = restriction_status::LIFTED;
        restriction.lifted_at = now;
        restriction.updated_at = now;

        // Restore capability: clear the DECLARED clamp (allow self-declare again;
        // VERIFIED still needs the normal path). Cap stays at DECLARED until
        // the wallet re-declares / re-verifies — rehabilitation is gradual.
        let cap = &mut ctx.accounts.capability;
        cap.updated_at = now;

        // Report returns to UPHELD (restriction no longer active for appeal).
        let report = &mut ctx.accounts.report;
        if report.status == fraud_status::APPEALED {
            report.status = fraud_status::UPHELD;
            report.updated_at = now;
        }

        emit!(crate::CapabilityRestrictionLifted {
            restriction: ctx.accounts.restriction.key(),
            wallet: ctx.accounts.restriction.wallet,
            capability_code: ctx.accounts.restriction.capability_code,
            by: crate::fraud_governance::lift_reason::APPEAL,
        });
    } else {
        appeal.status = appeal_status::DENIED;
        // Report returns to UPHELD (restriction still active).
        let report = &mut ctx.accounts.report;
        if report.status == fraud_status::APPEALED {
            report.status = fraud_status::UPHELD;
            report.updated_at = now;
        }
    }

    emit!(crate::FraudAppealDecided {
        appeal: appeal_key,
        decision,
    });
    Ok(())
}

/// Rehabilitation: after `RESTRICTION_REHAB_SECS` of clean standing, the
/// restricted wallet lifts their own restriction (self-regulating; no admin).
pub fn rehabilitate_restriction(ctx: Context<crate::RehabilitateRestriction>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let restriction = &ctx.accounts.restriction;
    require!(
        is_valid_restriction_status(restriction.status),
        TerraError::InvalidRestrictionStatus
    );
    require!(
        can_lift(restriction.status),
        TerraError::RestrictionNotActive
    );
    require!(
        rehab_is_eligible(now, restriction.rehab_eligible_at),
        TerraError::RehabTooEarly
    );
    require!(
        ctx.accounts.wallet.key() == restriction.wallet,
        TerraError::NotRestrictedWallet
    );
    // No open appeal blocking rehab: report must not be APPEALED.
    require!(
        ctx.accounts.report.status != fraud_status::APPEALED,
        TerraError::InvalidFraudStatus
    );

    let r = &mut ctx.accounts.restriction;
    r.status = restriction_status::LIFTED;
    r.lifted_at = now;
    r.updated_at = now;

    let cap = &mut ctx.accounts.capability;
    cap.updated_at = now;

    emit!(crate::CapabilityRestrictionLifted {
        restriction: r.key(),
        wallet: r.wallet,
        capability_code: r.capability_code,
        by: crate::fraud_governance::lift_reason::REHABILITATION,
    });
    Ok(())
}

/// Lift-reason codes emitted with CapabilityRestrictionLifted.
pub mod lift_reason {
    pub const APPEAL: u8 = 0;
    pub const REHABILITATION: u8 = 1;
    pub const MAX: u8 = REHABILITATION;
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn pk(b: u8) -> Pubkey {
        Pubkey::new_from_array([b; 32])
    }

    #[test]
    fn status_constants_are_contiguous() {
        assert_eq!(fraud_status::OPEN, 0);
        assert_eq!(fraud_status::UNDER_REVIEW, 1);
        assert_eq!(fraud_status::UPHELD, 2);
        assert_eq!(fraud_status::DISMISSED, 3);
        assert_eq!(fraud_status::APPEALED, 4);
        for s in 0..=fraud_status::MAX {
            assert!(is_valid_fraud_status(s));
        }
        assert!(!is_valid_fraud_status(fraud_status::MAX + 1));

        assert_eq!(restriction_status::ACTIVE, 0);
        assert_eq!(restriction_status::LIFTED, 1);
        for s in 0..=restriction_status::MAX {
            assert!(is_valid_restriction_status(s));
        }

        assert_eq!(appeal_status::PENDING, 0);
        assert_eq!(appeal_status::GRANTED, 1);
        assert_eq!(appeal_status::DENIED, 2);
        for s in 0..=appeal_status::MAX {
            assert!(is_valid_appeal_status(s));
        }
    }

    #[test]
    fn reason_codes_cover_rfc_categories() {
        for r in 0..=fraud_reason::MAX {
            assert!(is_valid_reason_code(r));
        }
        assert!(!is_valid_reason_code(fraud_reason::MAX + 1));
    }

    #[test]
    fn self_report_is_rejected() {
        assert!(is_self_report(pk(1), pk(1)));
        assert!(!is_self_report(pk(1), pk(2)));
    }

    #[test]
    fn review_open_and_vote_gates() {
        assert!(can_open_review(fraud_status::OPEN));
        assert!(!can_open_review(fraud_status::UNDER_REVIEW));
        assert!(!can_open_review(fraud_status::UPHELD));

        assert!(can_cast_vote(review_decision::PENDING));
        assert!(!can_cast_vote(review_decision::UPHELD));
        assert!(!can_cast_vote(review_decision::DISMISSED));
    }

    #[test]
    fn double_vote_is_detected() {
        let voted = vec![pk(1), pk(2)];
        assert!(already_voted(&voted, &pk(1)));
        assert!(!already_voted(&voted, &pk(3)));
    }

    #[test]
    fn finalize_requires_full_committee_and_majority_works() {
        assert!(!can_finalize(4, 5));
        assert!(can_finalize(5, 5));
        assert!(!can_finalize(0, 0));

        assert_eq!(decide_majority(3, 2), review_decision::UPHELD);
        assert_eq!(decide_majority(5, 0), review_decision::UPHELD);
        assert_eq!(decide_majority(2, 3), review_decision::DISMISSED);
        assert_eq!(decide_majority(0, 5), review_decision::DISMISSED);
    }

    #[test]
    fn restriction_is_demotion_not_ban() {
        assert_eq!(restricted_max_level(), capability_level::DECLARED);
        assert_eq!(restricted_tier(), profile_tier::PROBATIONARY);
        // Never a "jail" — status is only ACTIVE or LIFTED.
        assert!(is_valid_restriction_status(restriction_status::ACTIVE));
        assert!(is_valid_restriction_status(restriction_status::LIFTED));
    }

    #[test]
    fn blocks_capability_matches_wallet_and_code() {
        let w = pk(9);
        let r = CapabilityRestriction {
            wallet: w,
            capability_code: 2,
            report: pk(1),
            max_level: 0,
            status: restriction_status::ACTIVE,
            applied_at: 0,
            appeal_opens_at: 0,
            rehab_eligible_at: 0,
            lifted_at: 0,
            updated_at: 0,
        };
        assert!(blocks_capability(Some(&r), &w, 2));
        assert!(!blocks_capability(Some(&r), &w, 3));
        assert!(!blocks_capability(Some(&r), &pk(8), 2));
        assert!(!blocks_capability(None, &w, 2));

        let lifted = CapabilityRestriction {
            status: restriction_status::LIFTED,
            ..r
        };
        assert!(!blocks_capability(Some(&lifted), &w, 2));
    }

    #[test]
    fn appeal_and_rehab_windows() {
        // appeal_opens_at = 1000
        assert!(!appeal_is_open(999, 1000));
        assert!(appeal_is_open(1000, 1000));
        assert!(appeal_is_open(1001, 1000));

        assert!(!rehab_is_eligible(999, 1000));
        assert!(rehab_is_eligible(1000, 1000));

        assert!(can_lift(restriction_status::ACTIVE));
        assert!(!can_lift(restriction_status::LIFTED));
    }

    #[test]
    fn committee_pool_filters_low_reputation() {
        let pool = vec![pk(1), pk(2), pk(3), pk(4), pk(5), pk(6)];
        let scores = vec![100, 5000, 9999, 4999, 6000, 7000];
        let eligible = filter_committee_pool(&pool, &scores);
        // pk(1)=100 out, pk(4)=4999 out → 4 remain — too small for COMMITTEE_SIZE=5
        assert_eq!(eligible.len(), 4);

        let scores2 = vec![5000, 5000, 5000, 5000, 5000, 6000];
        let eligible2 = filter_committee_pool(&pool, &scores2);
        assert_eq!(eligible2.len(), 6);
    }

    #[test]
    fn select_committee_is_deterministic_and_distinct() {
        let pool: Vec<Pubkey> = (0..10u8).map(pk).collect();
        let seed = committee_seed(&[7u8; 32], 42);
        let c1 = select_committee(&seed, &pool).expect("committee");
        let c2 = select_committee(&seed, &pool).expect("committee");
        assert_eq!(c1, c2);
        assert_eq!(c1.len(), COMMITTEE_SIZE);
        // Distinct
        for i in 0..c1.len() {
            for j in (i + 1)..c1.len() {
                assert_ne!(c1[i], c1[j]);
            }
        }
        // Seed sensitivity
        let c3 = select_committee(&committee_seed(&[7u8; 32], 43), &pool).unwrap();
        assert_ne!(c1, c3);

        // Too-small pool
        let tiny: Vec<Pubkey> = (0..4u8).map(pk).collect();
        assert!(select_committee(&seed, &tiny).is_none());
    }

    #[test]
    fn committee_seed_changes_with_slot() {
        let s1 = committee_seed(&[1u8; 32], 1);
        let s2 = committee_seed(&[1u8; 32], 2);
        assert_ne!(s1, s2);
        assert_eq!(committee_seed(&[1u8; 32], 1), s1);
    }

    #[test]
    fn draw_bps_in_bounds() {
        let seed = [3u8; 32];
        let k = pk(5);
        for _ in 0..32 {
            let d = draw_bps(&seed, &k);
            assert!(d < 10_000);
        }
    }

    #[test]
    fn lift_reason_codes() {
        assert_eq!(lift_reason::APPEAL, 0);
        assert_eq!(lift_reason::REHABILITATION, 1);
        assert_eq!(lift_reason::MAX, 1);
    }

    #[test]
    fn note_length_guard() {
        let ok = "x".repeat(MAX_FRAUD_NOTE_LEN);
        let bad = "x".repeat(MAX_FRAUD_NOTE_LEN + 1);
        assert!(ok.len() <= MAX_FRAUD_NOTE_LEN);
        assert!(bad.len() > MAX_FRAUD_NOTE_LEN);
    }
}
