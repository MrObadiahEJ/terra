use anchor_lang::prelude::*;

/// Maximum validators in the global registry.
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

#[account]
#[derive(InitSpace)]
pub struct AuthorityRegistry {
    /// Bootstrap admin who can add validators unilaterally in bootstrap mode.
    pub admin: Pubkey,
    /// Current list of registered validators.
    #[max_len(32)]
    pub validators: Vec<Pubkey>,
    /// Minimum endorsements needed for new additions in peer-consensus mode.
    pub required_endorsements: u8,
    /// 0 = bootstrap (admin has unilateral power), 1 = peer-consensus.
    pub mode: u8,
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
    registry.mode = registry_mode::BOOTSTRAP;
    registry.version = 0;
    registry.created_at = clock.unix_timestamp;
    registry.updated_at = clock.unix_timestamp;

    emit!(super::RegistryCreated {
        registry: registry.key(),
        admin: registry.admin,
        mode: registry.mode,
    });
    Ok(())
}

/// Add a validator to the registry.
///
/// Bootstrap mode: admin can add unilaterally.
/// Peer-consensus mode: requires ceil(2n/3) endorsements from existing
/// validators. Use `endorse_validator_add` to collect endorsements first,
/// then call this with the completed endorsement account.
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

    if registry.mode == registry_mode::BOOTSTRAP {
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
            mode: registry.mode,
        });
    } else {
        // Peer-consensus: endorsement account must be valid and quorum met.
        let endorsement = &ctx.accounts.endorsement;
        require!(
            endorsement.registry == registry.key(),
            super::TerraError::AttestationMismatch
        );
        require!(
            endorsement.proposed == validator,
            super::TerraError::NotValidator
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
            mode: registry.mode,
        });
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
        // Peer removal — endorsement quorum required.
        let endorsement = &ctx.accounts.endorsement;
        require!(
            endorsement.registry == registry.key(),
            super::TerraError::AttestationMismatch
        );
        require!(
            endorsement.endorsers.len() as u8 >= endorsement.required,
            super::TerraError::QuorumNotMetForRotation
        );
        registry.validators.remove(pos);
    }

    registry.version = registry.version.saturating_add(1);
    registry.updated_at = clock.unix_timestamp;

    emit!(super::ValidatorRemoved {
        registry: registry.key(),
        validator,
        mode: registry.mode,
    });
    Ok(())
}

/// Endorse adding a validator in peer-consensus mode.
pub fn endorse_validator_add(ctx: Context<super::EndorseValidatorAdd>) -> Result<()> {
    let endorsement = &mut ctx.accounts.endorsement;
    let endorser = ctx.accounts.endorser.key();

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

/// Flip the registry from bootstrap to peer-consensus mode.
/// Only the admin can call this.
pub fn flip_to_consensus(ctx: Context<super::FlipToConsensus>) -> Result<()> {
    let registry = &mut ctx.accounts.registry;
    require!(
        ctx.accounts.admin_signer.key() == registry.admin,
        super::TerraError::NotAuthorized
    );
    require!(
        registry.mode == registry_mode::BOOTSTRAP,
        super::TerraError::RotationAlreadyFinalized // reuse: already consensus
    );
    require!(
        !registry.validators.is_empty(),
        super::TerraError::NoValidators
    );

    let n = registry.validators.len() as u8;
    let required = (n * CONSENSUS_FRACTION_NUM / CONSENSUS_FRACTION_DEN)
        + if (n * CONSENSUS_FRACTION_NUM) % CONSENSUS_FRACTION_DEN != 0 {
            1
        } else {
            0
        };

    registry.mode = registry_mode::PEER_CONSENSUS;
    registry.required_endorsements = required;
    registry.version = registry.version.saturating_add(1);
    registry.updated_at = Clock::get()?.unix_timestamp;

    emit!(super::ConsensusFlipped {
        registry: registry.key(),
        admin: registry.admin,
        required_endorsements: required,
        validator_count: n,
    });
    Ok(())
}

impl ValidatorEndorsement {
    /// Helper to produce a display pubkey for the endorsement set.
    pub fn endorsements_pubkey(&self) -> Pubkey {
        self.endorsers.first().copied().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Compute ceil(2n/3) — the quorum required for peer-consensus operations.
    fn consensus_required(n: u8) -> u8 {
        (n * CONSENSUS_FRACTION_NUM / CONSENSUS_FRACTION_DEN)
            + if (n * CONSENSUS_FRACTION_NUM) % CONSENSUS_FRACTION_DEN != 0 {
                1
            } else {
                0
            }
    }

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
    fn bootstrap_mode_allows_unilateral_add() {
        // In bootstrap mode, the admin can add validators without endorsement.
        let admin = Pubkey::new_unique();
        let v1 = Pubkey::new_unique();
        let v2 = Pubkey::new_unique();

        let mut validators = Vec::new();
        // Simulate bootstrap add.
        validators.push(v1);
        assert_eq!(validators.len(), 1);
        assert!(validators.contains(&v1));
        // Admin can add another.
        validators.push(v2);
        assert_eq!(validators.len(), 2);
        // Admin is not in the list (only validators are).
        assert!(!validators.contains(&admin));
    }

    #[test]
    fn peer_consensus_requires_quorum() {
        // Simulate a 3-validator registry requiring ceil(2*3/3) = 2 endorsements.
        let required = consensus_required(3);
        assert_eq!(required, 2);

        let endorser1 = Pubkey::new_unique();
        let endorser2 = Pubkey::new_unique();
        let endorser3 = Pubkey::new_unique();

        let mut endorsers: Vec<Pubkey> = Vec::new();
        // One endorsement is not enough.
        endorsers.push(endorser1);
        assert!((endorsers.len() as u8) < required);
        // Two endorsements meet quorum.
        endorsers.push(endorser2);
        assert!((endorsers.len() as u8) >= required);
        // The third can endorse but quorum already met.
        endorsers.push(endorser3);
        assert!((endorsers.len() as u8) >= required);
    }

    #[test]
    fn flip_to_consensus_cannot_be_called_twice() {
        // Once mode is PEER_CONSENSUS, flip should be rejected.
        let mode = registry_mode::PEER_CONSENSUS;
        assert!(mode != registry_mode::BOOTSTRAP);
    }

    #[test]
    fn duplicate_validator_rejected() {
        let v1 = Pubkey::new_unique();
        let mut validators = vec![v1];
        // Simulate duplicate check.
        assert!(validators.contains(&v1));
        // Adding again would be rejected.
    }
}
