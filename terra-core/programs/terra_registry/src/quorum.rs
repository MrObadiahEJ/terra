use anchor_lang::prelude::AccountInfo;
use anchor_lang::prelude::*;

use crate::{TerraError, MAX_VALIDATORS};

/// Count unique non-default pubkeys in a declared validator set.
///
/// Returns `Ok(unique_count)` if every non-default pubkey appears exactly
/// once; `Err(())` on the first duplicate. Default (zero) pubkeys are
/// padding and may repeat freely.
///
/// Used to prevent `[v1, v1, v1]` from reporting count=3 while only one
/// unique validator can ever sign — which would make thresholds unreachable
/// and corrupt quorum accounting.
#[allow(clippy::result_unit_err)]
pub fn count_unique_validators(validators: &[Pubkey]) -> std::result::Result<usize, ()> {
    let mut unique: Vec<Pubkey> = Vec::with_capacity(validators.len());
    for &v in validators.iter() {
        if v == Pubkey::default() {
            continue;
        }
        if unique.contains(&v) {
            return Err(());
        }
        unique.push(v);
    }
    Ok(unique.len())
}

/// Count unique validators, mapping duplicates to `TerraError::DuplicateValidator`.
pub fn require_unique_validators(validators: &[Pubkey]) -> Result<u8> {
    count_unique_validators(validators)
        .map(|n| n as u8)
        .map_err(|_| error!(TerraError::DuplicateValidator))
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_validators_counts_distinct() {
        let a = Pubkey::new_unique();
        let b = Pubkey::new_unique();
        assert_eq!(count_unique_validators(&[a, b]), Ok(2));
    }

    #[test]
    fn default_padding_ignored() {
        let a = Pubkey::new_unique();
        let mut set = vec![a];
        set.extend_from_slice(&[Pubkey::default(); 6]);
        assert_eq!(count_unique_validators(&set), Ok(1));
    }

    #[test]
    fn duplicate_rejected() {
        let a = Pubkey::new_unique();
        let b = Pubkey::new_unique();
        assert_eq!(count_unique_validators(&[a, b, a]), Err(()));
    }

    #[test]
    fn all_defaults_is_zero() {
        assert_eq!(count_unique_validators(&[Pubkey::default(); 8]), Ok(0));
    }

    #[test]
    fn require_unique_maps_duplicate_to_error() {
        let a = Pubkey::new_unique();
        assert!(require_unique_validators(&[a, a]).is_err());
        assert!(require_unique_validators(&[a]).is_ok());
    }
}
