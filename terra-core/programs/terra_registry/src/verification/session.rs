use anchor_lang::prelude::*;

use crate::TerraError;

// ---------------------------------------------------------------------------
// Session status
// ---------------------------------------------------------------------------

pub mod session_status {
    pub const OPEN: u8 = 0;
    pub const CLOSED: u8 = 1;
    pub const TIMED_OUT: u8 = 2;
    pub const QUORUM_REACHED: u8 = 3;
    pub const CHALLENGED: u8 = 4;
    pub const MAX: u8 = CHALLENGED;
}

// ---------------------------------------------------------------------------
// Session timeout
// ---------------------------------------------------------------------------

/// Verification sessions expire after 30 days if not resolved.
pub const SESSION_TIMEOUT_SECS: i64 = 30 * 24 * 3600;

// ---------------------------------------------------------------------------
// VerificationSession account
// ---------------------------------------------------------------------------

/// A stateful session that tracks a claim through its full verification
/// lifecycle. Links evidence, observations, and attestations. Manages
/// timeout and quorum progression.
///
/// PDA: `["verification_session", claim, session_id]`
#[account]
#[derive(InitSpace)]
pub struct VerificationSession {
    /// The claim being verified.
    pub claim: Pubkey,
    /// Unique session identifier (caller-provided).
    pub session_id: [u8; 32],
    /// Current session status.
    pub status: u8,
    /// Participant who opened the session.
    pub opened_by: Pubkey,
    /// Number of evidence items attached.
    pub evidence_count: u8,
    /// Number of observations submitted.
    pub observation_count: u8,
    /// Number of confirmatory attestations.
    pub attestation_count: u8,
    /// Number of disconfirmatory attestations.
    pub dispute_count: u8,
    /// Required attestations for quorum.
    pub required_attestations: u8,
    pub created_at: i64,
    pub expires_at: i64,
    pub closed_at: i64,
    pub updated_at: i64,
}

impl VerificationSession {
    pub fn is_terminal(status: u8) -> bool {
        matches!(
            status,
            session_status::CLOSED
                | session_status::TIMED_OUT
                | session_status::CHALLENGED
        )
    }
}

// ---------------------------------------------------------------------------
// Instruction handlers
// ---------------------------------------------------------------------------

/// Open a new verification session for a claim. Anyone may open a session.
/// Only one session may be open per claim at a time (enforced by PDA seed
/// including session_id — the caller must ensure uniqueness).
pub fn open_verification_session(
    ctx: Context<crate::OpenVerificationSession>,
    session_id: [u8; 32],
    required_attestations: u8,
) -> Result<()> {
    require!(
        !session_id.iter().all(|b| *b == 0),
        TerraError::EmptyClaimId
    );
    require!(
        required_attestations >= 1,
        TerraError::InvalidRequiredAttestations
    );

    let now = Clock::get()?.unix_timestamp;
    let session = &mut ctx.accounts.session;
    session.claim = ctx.accounts.claim.key();
    session.session_id = session_id;
    session.status = session_status::OPEN;
    session.opened_by = ctx.accounts.opener.key();
    session.evidence_count = 0;
    session.observation_count = 0;
    session.attestation_count = 0;
    session.dispute_count = 0;
    session.required_attestations = required_attestations;
    session.created_at = now;
    session.expires_at = now.saturating_add(SESSION_TIMEOUT_SECS);
    session.closed_at = 0;
    session.updated_at = now;

    emit!(crate::VerificationSessionOpened {
        session: session.key(),
        claim: session.claim,
        session_id,
        opened_by: session.opened_by,
        expires_at: session.expires_at,
    });
    Ok(())
}

/// Close a verification session. Only the opener may close it.
/// Sessions can be closed when quorum is reached or when the opener
/// decides to withdraw the claim.
pub fn close_verification_session(
    ctx: Context<crate::CloseVerificationSession>,
    final_status: u8,
) -> Result<()> {
    require!(
        VerificationSession::is_terminal(final_status),
        TerraError::InvalidClaimStatus
    );
    require!(
        ctx.accounts.session.opened_by == ctx.accounts.opener.key(),
        TerraError::NotClaimSubmitter
    );

    let now = Clock::get()?.unix_timestamp;
    let session = &mut ctx.accounts.session;
    session.status = final_status;
    session.closed_at = now;

    emit!(crate::VerificationSessionClosed {
        session: session.key(),
        claim: session.claim,
        status: final_status,
        closed_at: now,
    });
    Ok(())
}

/// Record an evidence addition to the session (bumps evidence_count).
pub fn record_session_evidence(ctx: Context<crate::RecordSessionEvidence>) -> Result<()> {
    let session = &mut ctx.accounts.session;
    require!(
        !VerificationSession::is_terminal(session.status),
        TerraError::InvalidClaimStatus
    );

    session.evidence_count = session.evidence_count.saturating_add(1);
    session.updated_at = Clock::get()?.unix_timestamp;
    Ok(())
}

/// Record an observation to the session.
pub fn record_session_observation(ctx: Context<crate::RecordSessionObservation>) -> Result<()> {
    let session = &mut ctx.accounts.session;
    require!(
        !VerificationSession::is_terminal(session.status),
        TerraError::InvalidClaimStatus
    );

    session.observation_count = session.observation_count.saturating_add(1);
    session.updated_at = Clock::get()?.unix_timestamp;
    Ok(())
}

/// Record an attestation to the session. Automatically transitions to
/// QUORUM_REACHED when the threshold is met.
pub fn record_session_attestation(
    ctx: Context<crate::RecordSessionAttestation>,
    is_confirmatory: bool,
) -> Result<()> {
    let session = &mut ctx.accounts.session;
    require!(
        !VerificationSession::is_terminal(session.status),
        TerraError::InvalidClaimStatus
    );

    if is_confirmatory {
        session.attestation_count = session.attestation_count.saturating_add(1);
    } else {
        session.dispute_count = session.dispute_count.saturating_add(1);
    }

    if session.attestation_count >= session.required_attestations {
        session.status = session_status::QUORUM_REACHED;
        session.closed_at = Clock::get()?.unix_timestamp;

        emit!(crate::VerificationSessionClosed {
            session: session.key(),
            claim: session.claim,
            status: session_status::QUORUM_REACHED,
            closed_at: session.closed_at,
        });
    }

    session.updated_at = Clock::get()?.unix_timestamp;
    Ok(())
}
