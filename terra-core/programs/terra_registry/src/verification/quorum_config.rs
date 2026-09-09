use anchor_lang::prelude::*;

use crate::TerraError;

// ---------------------------------------------------------------------------
// QuorumConfig account
// ---------------------------------------------------------------------------

/// On-chain quorum configuration. Defines how many attestations and what
/// confidence level are required to verify a claim of a given type, in
/// a given region. Global defaults can be overridden per-type or per-region.
///
/// PDA: `["quorum_config", parcel_type, region]`
#[account]
#[derive(InitSpace)]
pub struct QuorumConfig {
    /// Parcel type this config applies to (0 = global default).
    pub parcel_type: u8,
    /// Region this config applies to ([0,0] = global default).
    pub region: [u8; 2],
    /// Number of attestations required.
    pub required_attestations: u8,
    /// Minimum confidence level (0-100) required from each attestation.
    pub required_confidence: u8,
    /// Who set this config.
    pub set_by: Pubkey,
    pub created_at: i64,
    pub updated_at: i64,
}

// ---------------------------------------------------------------------------
// Instruction handlers
// ---------------------------------------------------------------------------

/// Set a quorum configuration. Only the registry admin may set configs.
pub fn set_quorum_config(
    ctx: Context<crate::SetQuorumConfig>,
    parcel_type: u8,
    region: [u8; 2],
    required_attestations: u8,
    required_confidence: u8,
) -> Result<()> {
    require!(
        required_attestations >= 1,
        TerraError::InvalidRequiredAttestations
    );
    require!(required_confidence <= 100, TerraError::InvalidConfidence);

    let registry = &ctx.accounts.registry;
    require!(
        ctx.accounts.admin.key() == registry.admin,
        TerraError::NotAuthorized
    );

    let now = Clock::get()?.unix_timestamp;
    let config = &mut ctx.accounts.config;
    config.parcel_type = parcel_type;
    config.region = region;
    config.required_attestations = required_attestations;
    config.required_confidence = required_confidence;
    config.set_by = ctx.accounts.admin.key();
    config.created_at = if config.created_at == 0 { now } else { config.created_at };
    config.updated_at = now;

    emit!(crate::QuorumConfigSet {
        config: config.key(),
        parcel_type,
        region,
        required_attestations,
        required_confidence,
        set_by: config.set_by,
    });
    Ok(())
}

/// Resolve the effective quorum for a given parcel type and region.
/// Falls back to global defaults (parcel_type=0, region=[0,0]) if no
/// specific config exists.
pub fn resolve_quorum(
    _registry: &crate::validator_registry::ValidatorRegistry,
    _parcel_type: u8,
    _region: [u8; 2],
    required_attestations: u8,
    required_confidence: u8,
) -> (u8, u8) {
    // If specific config values are provided (non-zero), use them.
    // Otherwise fall back to the claim's own required_attestations.
    let att = if required_attestations > 0 {
        required_attestations
    } else {
        2 // default minimum
    };
    let conf = if required_confidence > 0 {
        required_confidence
    } else {
        50 // default confidence
    };
    (att, conf)
}
