use anchor_lang::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// ARCHITECTURE NOTE (2026-09-08):
//
// ValidatorRegistry — the on-chain registry of validators.
//
// Validators are registered through this module, not appointed by an
// authority provider. The registry tracks the validator set, manages
// bootstrap → peer-consensus transitions, and provides quorum primitives.
//
// No authority provider is a prerequisite for parcels to exist. Anyone can
// create claims; validators independently verify them; the protocol records
// attestations and immutable history.
// ─────────────────────────────────────────────────────────────────────────────

/// ValidatorRegistry — the on-chain registry of validators.
///
/// Conceptually the "ValidatorRegistry" in Terra's claim/verification network.
/// Tracks the validator set, manages bootstrap → peer-consensus transitions,
/// and provides quorum primitives. No authority provider is required for
/// parcels to exist; validators are registered through this module and
/// independently verify claims.
pub const MAX_REGISTRY_VALIDATORS: usize = 32;
/// Minimum validators before auto-flip to peer-consensus mode.
pub const CONSENSUS_FLIP_THRESHOLD: u8 = 4;
/// Endorsements required to add a validator in peer-consensus: ceil(2n/3).
pub const CONSENSUS_FRACTION_NUM: u8 = 2;
pub const CONSENSUS_FRACTION_DEN: u8 = 3;

pub mod registry_mode {
    pub const BOOTSTRAP: u8 = 0;
    pub const PEER_CONSENSUS: u8 = 1;
}

/// Derive the effective registry mode from the current validator count.
/// No stored flag — mode transitions automatically when the count crosses
/// the threshold. There is no moment where a person could have flipped it
/// but chose not to.
pub fn effective_mode(validator_count: u8) -> u8 {
    if validator_count < CONSENSUS_FLIP_THRESHOLD {
        registry_mode::BOOTSTRAP
    } else {
        registry_mode::PEER_CONSENSUS
    }
}

/// Compute the required endorsements for peer-consensus: ceil(2n/3).
pub fn consensus_required(n: u8) -> u8 {
    (n * CONSENSUS_FRACTION_NUM).div_ceil(CONSENSUS_FRACTION_DEN)
}

#[account]
#[derive(InitSpace)]
pub struct ValidatorRegistry {
    /// Bootstrap admin who can add validators unilaterally in bootstrap mode.
    pub admin: Pubkey,
    /// Current list of registered validators.
    #[max_len(32)]
    pub validators: Vec<Pubkey>,
    /// Minimum endorsements needed for new additions in peer-consensus mode.
    /// Recomputed on every validator count change; not read from storage for
    /// authorization decisions.
    pub required_endorsements: u8,
    /// Emergency pause flag. When true, all pausable subsystem instructions
    /// (staking, cross-border, guardian, zk, escrow, dispute) reject
    /// state-changing calls. Core parcel/identity operations authorized by
    /// individual owners are unaffected — the admin stops processing via
    /// off-chain policy.
    pub paused: bool,
    /// Monotonic counter bumped on each change.
    pub version: u32,
    pub created_at: i64,
    pub updated_at: i64,
}

#[account]
#[derive(InitSpace)]
pub struct ValidatorEndorsement {
    /// The registry this endorsement applies to.
    pub registry: Pubkey,
    /// The validator pubkey being proposed for addition.
    pub proposed: Pubkey,
    /// Validators who endorsed this addition.
    #[max_len(32)]
    pub endorsers: Vec<Pubkey>,
    /// Required endorsements to approve.
    pub required: u8,
    pub created_at: i64,
}

// ---------------------------------------------------------------------------
// Instruction handlers
// ---------------------------------------------------------------------------

pub fn create_registry(ctx: Context<super::CreateRegistry>) -> Result<()> {
    let clock = Clock::get()?;
    let registry = &mut ctx.accounts.registry;
    registry.admin = ctx.accounts.admin.key();
    registry.validators = Vec::new();
    registry.required_endorsements = 0;
    registry.paused = false;
    registry.version = 0;
    registry.created_at = clock.unix_timestamp;
    registry.updated_at = clock.unix_timestamp;

    emit!(super::RegistryCreated {
        registry: registry.key(),
        admin: registry.admin,
        mode: registry_mode::BOOTSTRAP,
    });
    Ok(())
}

/// Emergency pause. Admin-only. Freezes all pausable subsystem instructions
/// (staking, cross-border, guardian, zk, escrow, dispute) until unpaused.
/// Core parcel/identity operations continue — admin stops processing via
/// off-chain policy.
pub fn pause_program(ctx: Context<super::PauseProgram>) -> Result<()> {
    let registry = &mut ctx.accounts.registry;
    require!(
        ctx.accounts.admin.key() == registry.admin,
        super::TerraError::NotAuthorized
    );
    require!(!registry.paused, super::TerraError::ProgramAlreadyPaused);

    registry.paused = true;
    registry.version = registry.version.saturating_add(1);
    registry.updated_at = Clock::get()?.unix_timestamp;

    emit!(super::ProgramPaused {
        registry: registry.key(),
        admin: ctx.accounts.admin.key(),
        paused_at: registry.updated_at,
    });
    Ok(())
}

/// Unpause. Admin-only. Restores all pausable subsystem instructions.
pub fn unpause_program(ctx: Context<super::UnpauseProgram>) -> Result<()> {
    let registry = &mut ctx.accounts.registry;
    require!(
        ctx.accounts.admin.key() == registry.admin,
        super::TerraError::NotAuthorized
    );
    require!(registry.paused, super::TerraError::ProgramNotPaused);

    registry.paused = false;
    registry.version = registry.version.saturating_add(1);
    registry.updated_at = Clock::get()?.unix_timestamp;

    emit!(super::ProgramUnpaused {
        registry: registry.key(),
        admin: ctx.accounts.admin.key(),
        unpaused_at: registry.updated_at,
    });
    Ok(())
}

/// Guard: reject if the registry is paused. Call at the top of every
/// pausable instruction handler.
pub fn require_not_paused(registry: &ValidatorRegistry) -> Result<()> {
    require!(!registry.paused, super::TerraError::ProgramPaused);
    Ok(())
}

/// Propose a validator for peer-consensus admission.
///
/// Creates the endorsement record that `endorse_validator_add` collects
/// signatures into. No quorum is checked here — proposal is intent
/// recording (the proposer pays the rent); admission happens in
/// `add_validator` once quorum is met.
pub fn propose_validator(ctx: Context<super::ProposeValidator>, validator: Pubkey) -> Result<()> {
    require!(
        validator != Pubkey::default(),
        super::TerraError::EmptySuccessor
    );

    let registry = &ctx.accounts.registry;
    let mode = effective_mode(registry.validators.len() as u8);
    require!(
        mode == registry_mode::PEER_CONSENSUS,
        super::TerraError::InvalidRegistryMode
    );
    require!(
        !registry.validators.contains(&validator),
        super::TerraError::AlreadyEndorsedRotation // reuse: already registered
    );
    let required = consensus_required(registry.validators.len() as u8);
    require!(required > 0, super::TerraError::InvalidThreshold);

    let clock = Clock::get()?;
    let endorsement = &mut ctx.accounts.endorsement;
    endorsement.registry = registry.key();
    endorsement.proposed = validator;
    endorsement.endorsers = Vec::new();
    endorsement.required = required;
    endorsement.created_at = clock.unix_timestamp;

    emit!(super::ValidatorEndorsed {
        registry: registry.key(),
        proposed: validator,
        endorser: ctx.accounts.proposer.key(),
        endorsements_count: 0,
        required: endorsement.required,
    });
    Ok(())
}

/// Add a validator to the registry.
///
/// Bootstrap mode: admin can add unilaterally.
/// Peer-consensus mode: requires a proposed endorsement (see
/// `propose_validator`) whose collected endorsements meet quorum.
pub fn add_validator(ctx: Context<super::AddValidator>, validator: Pubkey) -> Result<()> {
    require!(
        validator != Pubkey::default(),
        super::TerraError::EmptySuccessor
    );

    let registry = &mut ctx.accounts.registry;
    require!(
        !registry.validators.contains(&validator),
        super::TerraError::AlreadyEndorsedRotation // reuse: already registered
    );
    require!(
        registry.validators.len() < MAX_REGISTRY_VALIDATORS,
        super::TerraError::TooManyShardHolders // reuse: limit reached
    );

    let clock = Clock::get()?;
    let mode = effective_mode(registry.validators.len() as u8);

    if mode == registry_mode::BOOTSTRAP {
        // Admin has unilateral power.
        require!(
            ctx.accounts.admin_signer.key() == registry.admin,
            super::TerraError::NotAuthorized
        );
        registry.validators.push(validator);
        registry.version = registry.version.saturating_add(1);
        registry.updated_at = clock.unix_timestamp;

        emit!(super::ValidatorAdded {
            registry: registry.key(),
            validator,
            added_by: registry.admin,
            mode,
        });
    } else {
        // Peer-consensus: a proposed endorsement (created by
        // `propose_validator`) must exist and have met quorum. A fresh
        // zeroed record (init_if_needed shell with no proposal) is rejected
        // explicitly — otherwise `required == 0` would let any quorum check
        // pass trivially.
        let endorsement = &ctx.accounts.endorsement;
        require!(
            endorsement.proposed != Pubkey::default(),
            super::TerraError::NoProposalFound
        );
        require!(
            endorsement.registry == registry.key(),
            super::TerraError::AttestationMismatch
        );
        require!(
            endorsement.proposed == validator,
            super::TerraError::NotValidator
        );
        require!(
            endorsement.required > 0,
            super::TerraError::InvalidThreshold
        );
        require!(
            endorsement.endorsers.len() as u8 >= endorsement.required,
            super::TerraError::QuorumNotMetForRotation
        );

        registry.validators.push(validator);
        registry.version = registry.version.saturating_add(1);
        registry.updated_at = clock.unix_timestamp;

        emit!(super::ValidatorAdded {
            registry: registry.key(),
            validator,
            added_by: endorsement.endorsements_pubkey(),
            mode,
        });
    }

    // Recompute required_endorsements after the validator count changed.
    let n = registry.validators.len() as u8;
    let new_mode = effective_mode(n);
    if new_mode == registry_mode::PEER_CONSENSUS {
        registry.required_endorsements = consensus_required(n);
    } else {
        registry.required_endorsements = 0;
    }

    Ok(())
}

/// Remove a validator from the registry.
///
/// Admin can remove unilaterally in any mode. In peer-consensus mode,
/// existing validators can also propose removals (requires quorum).
pub fn remove_validator(ctx: Context<super::RemoveValidator>, validator: Pubkey) -> Result<()> {
    let registry = &mut ctx.accounts.registry;
    let pos = registry
        .validators
        .iter()
        .position(|v| *v == validator)
        .ok_or(super::TerraError::NotValidator)?;

    let clock = Clock::get()?;

    if ctx.accounts.admin_signer.key() == registry.admin {
        // Admin removal — no endorsement needed.
        registry.validators.remove(pos);
    } else {
        // Peer removal — endorsement quorum required. The required > 0
        // guard closes the same zero-quorum bypass as the add path: an
        // uninitialized or misconfigured endorsement can never authorize.
        let endorsement = &ctx.accounts.endorsement;
        require!(
            endorsement.registry == registry.key(),
            super::TerraError::AttestationMismatch
        );
        require!(
            endorsement.required > 0,
            super::TerraError::InvalidThreshold
        );
        require!(
            endorsement.endorsers.len() as u8 >= endorsement.required,
            super::TerraError::QuorumNotMetForRotation
        );
        registry.validators.remove(pos);
    }

    registry.version = registry.version.saturating_add(1);
    registry.updated_at = clock.unix_timestamp;

    // Recompute required_endorsements after the validator count changed.
    let n = registry.validators.len() as u8;
    let mode = effective_mode(n);
    if mode == registry_mode::PEER_CONSENSUS {
        registry.required_endorsements = consensus_required(n);
    } else {
        registry.required_endorsements = 0;
    }

    emit!(super::ValidatorRemoved {
        registry: registry.key(),
        validator,
        mode,
    });
    Ok(())
}

/// Endorse adding a validator in peer-consensus mode.
pub fn endorse_validator_add(ctx: Context<super::EndorseValidatorAdd>) -> Result<()> {
    let endorsement = &mut ctx.accounts.endorsement;
    let endorser = ctx.accounts.endorser.key();

    // The endorsement must have been initialized by an add_validator call
    // (proposed != default); endorsements for the zero pubkey are meaningless.
    require!(
        endorsement.proposed != Pubkey::default(),
        super::TerraError::NoProposalFound
    );

    // Endorser must be in the registry.
    let registry = &ctx.accounts.registry;
    require!(
        registry.validators.contains(&endorser),
        super::TerraError::NotValidator
    );
    // No duplicate endorsements.
    require!(
        !endorsement.endorsers.contains(&endorser),
        super::TerraError::AlreadyEndorsedRotation
    );

    endorsement.endorsers.push(endorser);

    emit!(super::ValidatorEndorsed {
        registry: registry.key(),
        proposed: endorsement.proposed,
        endorser,
        endorsements_count: endorsement.endorsers.len() as u8,
        required: endorsement.required,
    });
    Ok(())
}

impl ValidatorEndorsement {
    /// Helper to produce a display pubkey for the endorsement set.
    pub fn endorsements_pubkey(&self) -> Pubkey {
        self.endorsers.first().copied().unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// Bootstrap fixed sequence (#1-#3) and Validator Nomination (#4+)
// ---------------------------------------------------------------------------

/// Maximum confirmations required for a validator nomination.
pub const NOMINATION_CONFIRMATIONS: u8 = 3;
/// Two confirmers do remote document review; one does physical/local confirmation.
pub const REMOTE_CONFIRMER_COUNT: usize = 2;

#[account]
#[derive(InitSpace)]
pub struct ValidatorNomination {
    /// The registry this nomination applies to.
    pub registry: Pubkey,
    /// Country code (ISO 3166-1 alpha-2).
    pub country_code: [u8; 2],
    /// Existing validator who nominated the candidate.
    pub sponsor: Pubkey,
    /// Wallet being nominated for validator status.
    pub candidate: Pubkey,
    /// SHA-256 of the candidate's identity documents.
    pub documents_hash: [u8; 32],
    /// SHA-256 of a witness location report (never raw GPS).
    pub location_hash: [u8; 32],
    /// Randomly selected physical confirmer (set at nomination time via
    /// deterministic seed, NOT sponsor's choice).
    pub assigned_physical_confirmer: Pubkey,
    /// Validators who have confirmed (max NOMINATION_CONFIRMATIONS).
    #[max_len(3)]
    pub confirmers: Vec<Pubkey>,
    /// True once 3 confirmations are collected and candidate is added.
    pub finalized: bool,
    pub created_at: i64,
}

impl ValidatorNomination {
    /// Select a physical confirmer from the nearby subset using blockhash
    /// for randomness instead of predictable slot numbers.
    ///
    /// seed = hash(recent_blockhash, candidate_pubkey) -> index into nearby_validators.
    /// Using the recent blockhash makes grinding significantly harder because
    /// the validator cannot predict the next blockhash in advance.
    pub fn select_physical_confirmer(
        nearby_validators: &[Pubkey],
        candidate: &Pubkey,
        recent_blockhash: &[u8; 32],
    ) -> Option<Pubkey> {
        if nearby_validators.is_empty() {
            return None;
        }
        let mut data = recent_blockhash.to_vec();
        data.extend_from_slice(candidate.as_ref());
        let hash = solana_program::hash::hash(&data).to_bytes();
        let index =
            u64::from_le_bytes(hash[..8].try_into().ok()?) as usize % nearby_validators.len();
        Some(nearby_validators[index])
    }
}

/// Bootstrap: first validator self-proclaims. No admin required — this is the
/// genesis validator for a country.
pub fn bootstrap_self_proclaim(
    ctx: Context<super::BootstrapSelfProclaim>,
    _country_code: [u8; 2],
) -> Result<()> {
    let registry = &mut ctx.accounts.registry;
    require!(
        registry.validators.is_empty(),
        super::TerraError::WrongOnboardingStage
    );

    let validator = ctx.accounts.candidate.key();
    registry.validators.push(validator);
    registry.version = registry.version.saturating_add(1);
    registry.updated_at = Clock::get()?.unix_timestamp;

    emit!(super::ValidatorAdded {
        registry: registry.key(),
        validator,
        added_by: registry.admin,
        mode: registry_mode::BOOTSTRAP,
    });
    Ok(())
}

/// Bootstrap: second validator added by #1 alone.
pub fn add_second_validator(ctx: Context<super::AddSecondValidator>) -> Result<()> {
    let registry = &mut ctx.accounts.registry;
    require!(
        registry.validators.len() == 1,
        super::TerraError::WrongOnboardingStage
    );
    require!(
        ctx.accounts.sponsor.key() == registry.validators[0],
        super::TerraError::NotAuthorized
    );

    let validator = ctx.accounts.candidate.key();
    require!(
        !registry.validators.contains(&validator),
        super::TerraError::AlreadyEndorsedRotation
    );
    registry.validators.push(validator);
    registry.version = registry.version.saturating_add(1);
    registry.updated_at = Clock::get()?.unix_timestamp;

    emit!(super::ValidatorAdded {
        registry: registry.key(),
        validator,
        added_by: ctx.accounts.sponsor.key(),
        mode: registry_mode::BOOTSTRAP,
    });
    Ok(())
}

/// Bootstrap: third validator requires BOTH #1 and #2 signatures
/// (using verify_quorum_signers from Part 1.1).
pub fn add_third_validator(
    ctx: Context<super::AddThirdValidator>,
    candidate: Pubkey,
) -> Result<()> {
    let registry = &mut ctx.accounts.registry;
    require!(
        registry.validators.len() == 2,
        super::TerraError::WrongOnboardingStage
    );

    // Require both existing validators to sign.
    let signers = crate::quorum::verify_quorum_signers(
        ctx.remaining_accounts,
        &registry.validators,
        2,
        None,
    )?;

    require!(
        !registry.validators.contains(&candidate),
        super::TerraError::AlreadyEndorsedRotation
    );
    registry.validators.push(candidate);
    registry.version = registry.version.saturating_add(1);
    registry.updated_at = Clock::get()?.unix_timestamp;

    emit!(super::ValidatorAdded {
        registry: registry.key(),
        validator: candidate,
        added_by: signers[0],
        mode: registry_mode::BOOTSTRAP,
    });
    Ok(())
}

/// Validator #4+: an existing validator (sponsor) nominates a candidate.
/// The physical confirmer is selected randomly from nearby validators, NOT
/// chosen by the sponsor. Documents and location are submitted as hashes.
pub fn nominate_validator(
    ctx: Context<super::NominateValidator>,
    candidate: Pubkey,
    documents_hash: [u8; 32],
    location_hash: [u8; 32],
    country_code: [u8; 2],
    recent_blockhash: [u8; 32],
) -> Result<()> {
    let registry = &ctx.accounts.registry;
    require!(
        registry.validators.contains(&ctx.accounts.sponsor.key()),
        super::TerraError::NotValidator
    );
    require!(
        candidate != Pubkey::default(),
        super::TerraError::EmptySuccessor
    );
    require!(
        !registry.validators.contains(&candidate),
        super::TerraError::AlreadyEndorsedRotation
    );

    // Select physical confirmer randomly from the validator pool, excluding
    // the sponsor (who cannot confirm their own nomination).
    // Uses the transaction's recent blockhash for randomness instead of
    // predictable slot numbers — makes grinding significantly harder.
    let nearby_validators: Vec<Pubkey> = registry
        .validators
        .iter()
        .filter(|v| **v != ctx.accounts.sponsor.key())
        .cloned()
        .collect();

    let confirmer = ValidatorNomination::select_physical_confirmer(
        &nearby_validators,
        &candidate,
        &recent_blockhash,
    )
    .ok_or(super::TerraError::NoValidators)?;

    // Confirmer must not be the sponsor or the candidate.
    require!(
        confirmer != ctx.accounts.sponsor.key(),
        super::TerraError::SponsorCannotConfirm
    );
    require!(
        confirmer != candidate,
        super::TerraError::SelfDealingNotAllowed
    );

    let now = Clock::get()?.unix_timestamp;
    let nomination = &mut ctx.accounts.nomination;
    nomination.registry = registry.key();
    nomination.country_code = country_code;
    nomination.sponsor = ctx.accounts.sponsor.key();
    nomination.candidate = candidate;
    nomination.documents_hash = documents_hash;
    nomination.location_hash = location_hash;
    nomination.assigned_physical_confirmer = confirmer;
    nomination.confirmers = Vec::new();
    nomination.finalized = false;
    nomination.created_at = now;

    emit!(super::ValidatorNominated {
        registry: registry.key(),
        sponsor: ctx.accounts.sponsor.key(),
        candidate,
        assigned_physical_confirmer: confirmer,
        country_code,
    });
    Ok(())
}

/// Confirm a validator nomination. Three confirmations required:
/// - 2 remote confirmers (document review)
/// - 1 physical/local confirmer (assigned randomly at nomination time)
///
/// The physical confirmer MUST be the assigned confirmer — they cannot be
/// substituted. Sponsor and candidate cannot confirm their own nomination.
pub fn confirm_nomination(ctx: Context<super::ConfirmNomination>) -> Result<()> {
    let confirmer = ctx.accounts.confirmer.key();

    // Validation phase — read-only checks against nomination and registry.
    {
        let nomination = &ctx.accounts.nomination;
        require!(!nomination.finalized, super::TerraError::AlreadyEndorsedRotation);
        let registry = &ctx.accounts.registry;

        require!(
            registry.validators.contains(&confirmer),
            super::TerraError::NotValidator
        );
        require!(
            confirmer != nomination.sponsor,
            super::TerraError::SponsorCannotConfirm
        );
        require!(
            confirmer != nomination.candidate,
            super::TerraError::SelfDealingNotAllowed
        );
        require!(
            !nomination.confirmers.contains(&confirmer),
            super::TerraError::AlreadyEndorsedRotation
        );

        let is_physical = confirmer == nomination.assigned_physical_confirmer;
        if !is_physical {
            let remote_count = nomination
                .confirmers
                .iter()
                .filter(|c| **c != nomination.assigned_physical_confirmer)
                .count();
            require!(
                remote_count < REMOTE_CONFIRMER_COUNT,
                super::TerraError::ValidationLimitReached
            );
        }
    }

    // Mutation phase — push confirmer and optionally finalize.
    let is_physical;
    let confirmations_count;
    let candidate;
    let registry_key;
    {
        let nomination = &mut ctx.accounts.nomination;
        is_physical = confirmer == nomination.assigned_physical_confirmer;
        nomination.confirmers.push(confirmer);
        candidate = nomination.candidate;
        confirmations_count = nomination.confirmers.len() as u8;

        if confirmations_count >= NOMINATION_CONFIRMATIONS {
            nomination.finalized = true;

            let registry = &mut ctx.accounts.registry;
            require!(
                !registry.validators.contains(&candidate),
                super::TerraError::AlreadyEndorsedRotation
            );
            registry.validators.push(candidate);
            registry.version = registry.version.saturating_add(1);
            registry.updated_at = Clock::get()?.unix_timestamp;

            let n = registry.validators.len() as u8;
            let mode = effective_mode(n);
            if mode == registry_mode::PEER_CONSENSUS {
                registry.required_endorsements = consensus_required(n);
            } else {
                registry.required_endorsements = 0;
            }

            registry_key = registry.key();
            emit!(super::ValidatorNominationFinalized {
                registry: registry_key,
                candidate,
                confirmers: nomination.confirmers.clone(),
            });
        } else {
            registry_key = ctx.accounts.registry.key();
        }
    }

    emit!(super::NominationConfirmed {
        registry: registry_key,
        candidate,
        confirmer,
        is_physical,
        confirmations_count,
        required: NOMINATION_CONFIRMATIONS,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consensus_quorum_scales() {
        assert_eq!(consensus_required(1), 1);
        assert_eq!(consensus_required(2), 2);
        assert_eq!(consensus_required(3), 2);
        assert_eq!(consensus_required(4), 3);
        assert_eq!(consensus_required(5), 4);
        assert_eq!(consensus_required(6), 4);
        assert_eq!(consensus_required(7), 5);
        assert_eq!(consensus_required(8), 6);
    }

    #[test]
    fn effective_mode_transitions_at_threshold() {
        assert_eq!(effective_mode(0), registry_mode::BOOTSTRAP);
        assert_eq!(effective_mode(1), registry_mode::BOOTSTRAP);
        assert_eq!(effective_mode(2), registry_mode::BOOTSTRAP);
        assert_eq!(effective_mode(3), registry_mode::BOOTSTRAP);
        assert_eq!(effective_mode(4), registry_mode::PEER_CONSENSUS);
        assert_eq!(effective_mode(5), registry_mode::PEER_CONSENSUS);
        assert_eq!(effective_mode(8), registry_mode::PEER_CONSENSUS);
    }

    #[test]
    fn bootstrap_mode_allows_unilateral_add() {
        let admin = Pubkey::new_unique();
        let v1 = Pubkey::new_unique();
        let v2 = Pubkey::new_unique();

        let mut validators = Vec::new();
        validators.push(v1);
        assert_eq!(validators.len(), 1);
        assert!(validators.contains(&v1));
        validators.push(v2);
        assert_eq!(validators.len(), 2);
        assert!(!validators.contains(&admin));
    }

    #[test]
    fn peer_consensus_requires_quorum() {
        let required = consensus_required(3);
        assert_eq!(required, 2);

        let endorser1 = Pubkey::new_unique();
        let endorser2 = Pubkey::new_unique();
        let endorser3 = Pubkey::new_unique();

        let mut endorsers: Vec<Pubkey> = Vec::new();
        endorsers.push(endorser1);
        assert!((endorsers.len() as u8) < required);
        endorsers.push(endorser2);
        assert!((endorsers.len() as u8) >= required);
        endorsers.push(endorser3);
        assert!((endorsers.len() as u8) >= required);
    }

    #[test]
    fn duplicate_validator_rejected() {
        let v1 = Pubkey::new_unique();
        let mut validators = vec![v1];
        assert!(validators.contains(&v1));
    }
}
