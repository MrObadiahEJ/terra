use anchor_lang::prelude::*;
use anchor_lang::prelude::AccountInfo;

use crate::{MAX_VALIDATORS, TerraError};

/// Count unique signers from `remaining` that are in `declared`, with optional
/// exclusion for self-dealing checks. Returns the list of unique valid signers.
///
/// Deduplicates by pubkey — the same key listed twice in `remaining_accounts`
/// counts as one. This prevents a single real signature listed twice from
/// inflating the quorum count.
pub fn verify_quorum_signers<'info>(
    remaining: &[AccountInfo<'info>],
    declared: &[Pubkey],
    threshold: u8,
    exclude: Option<Pubkey>,
) -> Result<Vec<Pubkey>> {
    let mut seen: Vec<Pubkey> = Vec::new();
    for acc in remaining {
        if acc.is_signer && declared.contains(&acc.key()) && !seen.contains(&acc.key()) {
            if let Some(ex) = exclude {
                require!(acc.key() != ex, TerraError::ValidatorOwnsAsset);
            }
            seen.push(acc.key());
        }
    }
    require!(
        seen.len() as u8 >= threshold,
        TerraError::InsufficientValidatorSigners
    );
    Ok(seen)
}

/// Convert a `Vec<Pubkey>` from `verify_quorum_signers` into a fixed-size
/// array suitable for storing in dispute/attestation records. Pads with
/// `Pubkey::default()` up to `MAX_VALIDATORS`.
pub fn into_fixed_array(signers: &[Pubkey]) -> [Pubkey; MAX_VALIDATORS] {
    let mut arr = [Pubkey::default(); MAX_VALIDATORS];
    let len = signers.len().min(MAX_VALIDATORS);
    arr[..len].copy_from_slice(&signers[..len]);
    arr
}
