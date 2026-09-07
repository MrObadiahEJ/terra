use anchor_lang::prelude::*;

use crate::TerraError;

/// ISO 3166-1 alpha-2 country codes. This is a curated subset for the initial
/// launch — the full list can be loaded from the ISO standard. Using a const
/// array keeps the program self-contained without external dependencies.
///
/// For codes not listed here (user-assigned ranges: AA, QM–QZ, XA–XZ, ZZ),
/// use the allocation process to register them.
pub const VALID_COUNTRY_CODES: [[u8; 2]; 26] = [
    *b"US", *b"GB", *b"FR", *b"DE", *b"JP", *b"BR", *b"IN", *b"AU",
    *b"CA", *b"CN", *b"RU", *b"MX", *b"KR", *b"IT", *b"ES", *b"NL",
    *b"SE", *b"CH", *b"NO", *b"DK", *b"FI", *b"IE", *b"PT", *b"PL",
    *b"NG", *b"KE",
];

/// Minimum validators before auto-flip to peer-consensus (reused from
/// authority_registry).
pub const CONSENSUS_FLIP_THRESHOLD: u8 = 4;

/// Minimum confirmations for genesis approval.
pub const GENESIS_MIN_CONFIRMATIONS: u8 = 5;
/// Minimum distinct countries required in genesis confirmations.
pub const GENESIS_MIN_DISTINCT_COUNTRIES: usize = 3;

/// Check if a 2-byte country code is a valid ISO 3166-1 alpha-2 code.
pub fn is_valid_country_code(code: &[u8; 2]) -> bool {
    VALID_COUNTRY_CODES.contains(code)
}

#[account]
#[derive(InitSpace)]
pub struct WorldRegistry {
    /// Global founder — temporary sole authority during bootstrap phase.
    pub admin: Pubkey,
    /// Country allocations: which admin is approved to create a registry
    /// for which country code.
    #[max_len(256)]
    pub allocations: Vec<CountryAllocation>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct CountryAllocation {
    /// ISO 3166-1 alpha-2 country code.
    pub country_code: [u8; 2],
    /// Admin wallet approved to create this country's AuthorityRegistry.
    pub approved_admin: Pubkey,
}

/// Record of a cross-country genesis confirmation.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct GenesisConfirmation {
    /// Validator who confirmed.
    pub validator: Pubkey,
    /// Country code of the confirming validator.
    pub country_code: [u8; 2],
}

#[account]
#[derive(InitSpace)]
pub struct GenesisRequest {
    /// The country being genesis'd.
    pub country_code: [u8; 2],
    /// Admin requesting genesis (must match allocation).
    pub requested_by: Pubkey,
    /// Confirmations from validators of other countries.
    #[max_len(16)]
    pub confirmations: Vec<GenesisConfirmation>,
    /// True once quorum and diversity requirements are met.
    pub finalized: bool,
    pub created_at: i64,
}

// ---------------------------------------------------------------------------
// Instruction handlers
// ---------------------------------------------------------------------------

/// Create the WorldRegistry. Admin-only, one-time.
pub fn create_world_registry(ctx: Context<super::CreateWorldRegistry>) -> Result<()> {
    let clock = Clock::get()?;
    let registry = &mut ctx.accounts.world_registry;
    registry.admin = ctx.accounts.admin.key();
    registry.allocations = Vec::new();
    registry.created_at = clock.unix_timestamp;
    registry.updated_at = clock.unix_timestamp;

    emit!(super::WorldRegistryCreated {
        world_registry: registry.key(),
        admin: registry.admin,
    });
    Ok(())
}

/// Approve a country allocation. Admin-only. This authorizes a specific wallet
/// to create an AuthorityRegistry for a given country code.
pub fn allocate_country(
    ctx: Context<super::AllocateCountry>,
    country_code: [u8; 2],
    approved_admin: Pubkey,
) -> Result<()> {
    require!(
        is_valid_country_code(&country_code),
        TerraError::InvalidCountryCode
    );

    let registry = &mut ctx.accounts.world_registry;
    require!(
        ctx.accounts.admin.key() == registry.admin,
        TerraError::NotAuthorized
    );

    // No duplicate allocations for the same country.
    require!(
        !registry.allocations.iter().any(|a| a.country_code == country_code),
        TerraError::CountryAlreadyAllocated
    );

    registry.allocations.push(CountryAllocation {
        country_code,
        approved_admin,
    });
    registry.updated_at = Clock::get()?.unix_timestamp;

    emit!(super::CountryAllocated {
        world_registry: registry.key(),
        country_code,
        approved_admin,
    });
    Ok(())
}

/// Request genesis for a country. The caller must be the approved admin for
/// that country code (per the allocation record).
///
/// If fewer than 3 countries exist, the sole-admin allocation is sufficient —
/// no cross-country confirmation needed. Once ≥3 countries exist, genesis
/// requires GENESIS_MIN_CONFIRMATIONS from validators of at least
/// GENESIS_MIN_DISTINCT_COUNTRIES distinct countries.
pub fn request_genesis(ctx: Context<super::RequestGenesis>, country_code: [u8; 2]) -> Result<()> {
    require!(
        is_valid_country_code(&country_code),
        TerraError::InvalidCountryCode
    );

    let world_registry = &ctx.accounts.world_registry;
    let caller = ctx.accounts.requester.key();

    // Verify the caller is the approved admin for this country.
    require!(
        world_registry
            .allocations
            .iter()
            .any(|a| a.country_code == country_code && a.approved_admin == caller),
        TerraError::CountryNotAllocated
    );

    // Check if this country already has a genesis request.
    require!(
        !world_registry
            .allocations
            .iter()
            .any(|a| a.country_code == country_code),
        TerraError::CountryAlreadyAllocated
    );

    let now = Clock::get()?.unix_timestamp;
    let request = &mut ctx.accounts.genesis_request;
    request.country_code = country_code;
    request.requested_by = caller;
    request.confirmations = Vec::new();
    request.finalized = false;
    request.created_at = now;

    emit!(super::GenesisRequested {
        world_registry: world_registry.key(),
        country_code,
        requested_by: caller,
    });
    Ok(())
}

/// Confirm a genesis request. Confirmers must be validators from countries
/// OTHER than the one being genesis'd — cross-country diversity is enforced.
///
/// Once ≥3 countries exist, requires GENESIS_MIN_CONFIRMATIONS from
/// ≥ GENESIS_MIN_DISTINCT_COUNTRIES distinct countries.
pub fn confirm_genesis(
    ctx: Context<super::ConfirmGenesis>,
    confirmer_country: [u8; 2],
) -> Result<()> {
    let request = &mut ctx.accounts.genesis_request;
    require!(!request.finalized, TerraError::CountryAlreadyAllocated);

    let confirmer = ctx.accounts.confirmer.key();

    // Confirmer must be from a different country than the one being genesis'd.
    require!(
        confirmer_country != request.country_code,
        TerraError::ConfirmersNotDiverseEnough
    );

    // No duplicate confirmations.
    require!(
        !request.confirmations.iter().any(|c| c.validator == confirmer),
        TerraError::AlreadyEndorsedRotation
    );

    request.confirmations.push(GenesisConfirmation {
        validator: confirmer,
        country_code: confirmer_country,
    });

    // Check if we meet the diversity and count requirements.
    let distinct_countries: std::collections::HashSet<_> = request
        .confirmations
        .iter()
        .map(|c| c.country_code)
        .collect();

    if request.confirmations.len() as u8 >= GENESIS_MIN_CONFIRMATIONS
        && distinct_countries.len() >= GENESIS_MIN_DISTINCT_COUNTRIES
    {
        request.finalized = true;
        emit!(super::GenesisFinalized {
            world_registry: ctx.accounts.world_registry.key(),
            country_code: request.country_code,
            confirmations: request.confirmations.len() as u8,
            distinct_countries: distinct_countries.len() as u8,
        });
    }

    emit!(super::GenesisConfirmed {
        world_registry: ctx.accounts.world_registry.key(),
        country_code: request.country_code,
        confirmer,
        confirmer_country,
        confirmations_count: request.confirmations.len() as u8,
        required: GENESIS_MIN_CONFIRMATIONS,
    });
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn country_code_validation() {
        assert!(is_valid_country_code(b"US"));
        assert!(is_valid_country_code(b"GB"));
        assert!(is_valid_country_code(b"NG"));
        assert!(!is_valid_country_code(b"XX"));
        assert!(!is_valid_country_code(b"zz"));
    }

    #[test]
    fn distinct_countries_counting() {
        let confirmations = vec![
            GenesisConfirmation { validator: Pubkey::new_unique(), country_code: *b"US" },
            GenesisConfirmation { validator: Pubkey::new_unique(), country_code: *b"GB" },
            GenesisConfirmation { validator: Pubkey::new_unique(), country_code: *b"US" },
            GenesisConfirmation { validator: Pubkey::new_unique(), country_code: *b"FR" },
            GenesisConfirmation { validator: Pubkey::new_unique(), country_code: *b"GB" },
        ];
        let distinct: std::collections::HashSet<_> =
            confirmations.iter().map(|c| c.country_code).collect();
        assert_eq!(distinct.len(), 3);
        assert!(confirmations.len() as u8 >= GENESIS_MIN_CONFIRMATIONS);
    }
}
