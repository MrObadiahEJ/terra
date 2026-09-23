use super::constants::*;
use anchor_lang::prelude::Pubkey;

/// Returns true for the two guardianship succession kinds (3, 4).
pub fn is_guardianship_kind(kind: u8) -> bool {
    kind == succession_kind::GUARDIANSHIP || kind == succession_kind::COURT_APPOINTED_GUARDIAN
}

/// Normalize a requested grace period for a guardianship kind.
/// 0 => default 180d; otherwise must be >= 90d, clamped to the global max.
pub fn normalize_guardianship_grace(grace_secs: i64) -> i64 {
    if grace_secs == 0 {
        DEFAULT_GUARDIANSHIP_GRACE_SECS
    } else {
        grace_secs.clamp(MIN_GUARDIANSHIP_GRACE_SECS, MAX_SUCCESSION_GRACE_SECS)
    }
}

/// Validate the endorsement threshold for a guardianship request.
pub fn validate_guardianship_threshold(required_validations: u8, declared: usize) -> bool {
    (required_validations as usize) >= MIN_GUARDIANSHIP_VALIDATIONS as usize
        && (required_validations as usize) <= declared
}

/// Returns `Ok(unique_count)` if every non-default pubkey in `validators`
/// appears exactly once; otherwise `Err` with the index of the first
/// duplicate found.
///
/// Default (zero) pubkeys are padding and may repeat freely.
pub fn count_unique_validators(validators: &[Pubkey]) -> Result<usize, usize> {
    let mut unique: Vec<Pubkey> = Vec::with_capacity(validators.len());
    for (i, &v) in validators.iter().enumerate() {
        if v == Pubkey::default() {
            continue;
        }
        if unique.contains(&v) {
            return Err(i);
        }
        unique.push(v);
    }
    Ok(unique.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_validators_counts_distinct() {
        let a = Pubkey::new_unique();
        let b = Pubkey::new_unique();
        assert_eq!(count_unique_validators(&[a, b]).unwrap(), 2);
    }

    #[test]
    fn default_padding_ignored() {
        let a = Pubkey::new_unique();
        let defaults = [Pubkey::default(); 6];
        let mut set = vec![a];
        set.extend_from_slice(&defaults);
        assert_eq!(count_unique_validators(&set).unwrap(), 1);
    }

    #[test]
    fn duplicate_rejected_with_index() {
        let a = Pubkey::new_unique();
        let b = Pubkey::new_unique();
        // a at 0, b at 1, a again at 2 → duplicate at index 2
        assert_eq!(count_unique_validators(&[a, b, a]), Err(2));
    }

    #[test]
    fn all_defaults_is_zero_unique() {
        assert_eq!(count_unique_validators(&[Pubkey::default(); 8]).unwrap(), 0);
    }

    #[test]
    fn consecutive_duplicates_rejected() {
        let a = Pubkey::new_unique();
        assert_eq!(count_unique_validators(&[a, a]), Err(1));
    }
}
