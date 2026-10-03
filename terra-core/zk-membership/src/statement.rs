pub const OWNERSHIP_PROOF_DOMAIN: &[u8] = b"TERRA_ZK_OWNERSHIP_PROOF_V1";
pub const PURPOSE_CAPACITY: usize = 128;
pub const OWNERSHIP_STATEMENT_LEN: usize =
    OWNERSHIP_PROOF_DOMAIN.len() + 32 + 32 + 4 + 32 + 32 + 1 + PURPOSE_CAPACITY + 1;
pub const ZONE_OFFSET: usize = OWNERSHIP_PROOF_DOMAIN.len();
pub const ROOT_OFFSET: usize = ZONE_OFFSET + 32;
pub const VERSION_OFFSET: usize = ROOT_OFFSET + 32;
pub const NULLIFIER_OFFSET: usize = VERSION_OFFSET + 4;
pub const PROVER_OFFSET: usize = NULLIFIER_OFFSET + 32;
pub const PURPOSE_LENGTH_OFFSET: usize = PROVER_OFFSET + 32;
pub const PURPOSE_OFFSET: usize = PURPOSE_LENGTH_OFFSET + 1;
pub const DISCLOSURE_OFFSET: usize = PURPOSE_OFFSET + PURPOSE_CAPACITY;

pub fn canonical_ownership_statement(
    zone_set: &[u8; 32],
    merkle_root: &[u8; 32],
    root_version: u32,
    nullifier: &[u8; 32],
    prover: &[u8; 32],
    purpose: &str,
    disclosure_type: u8,
) -> Result<Vec<u8>, &'static str> {
    let purpose = purpose.as_bytes();
    if purpose.is_empty() || purpose.len() > PURPOSE_CAPACITY {
        return Err("proof purpose must contain 1..=128 UTF-8 bytes");
    }

    let mut statement = Vec::with_capacity(OWNERSHIP_STATEMENT_LEN);
    statement.extend_from_slice(OWNERSHIP_PROOF_DOMAIN);
    statement.extend_from_slice(zone_set);
    statement.extend_from_slice(merkle_root);
    statement.extend_from_slice(&root_version.to_le_bytes());
    statement.extend_from_slice(nullifier);
    statement.extend_from_slice(prover);
    statement.push(purpose.len() as u8);
    statement.extend_from_slice(purpose);
    statement.resize(statement.len() + (PURPOSE_CAPACITY - purpose.len()), 0);
    statement.push(disclosure_type);
    debug_assert_eq!(statement.len(), OWNERSHIP_STATEMENT_LEN);
    Ok(statement)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statement_uses_fixed_width_padded_fields() {
        let statement =
            canonical_ownership_statement(&[1; 32], &[2; 32], 4, &[3; 32], &[4; 32], "land", 0)
                .unwrap();
        assert_eq!(statement.len(), OWNERSHIP_STATEMENT_LEN);
        assert_eq!(&statement[ZONE_OFFSET..ROOT_OFFSET], &[1; 32]);
        assert_eq!(&statement[ROOT_OFFSET..VERSION_OFFSET], &[2; 32]);
        assert_eq!(
            &statement[VERSION_OFFSET..NULLIFIER_OFFSET],
            &4u32.to_le_bytes()
        );
        assert_eq!(statement[PURPOSE_LENGTH_OFFSET], 4);
        assert_eq!(&statement[PURPOSE_OFFSET..PURPOSE_OFFSET + 4], b"land");
        assert!(statement[PURPOSE_OFFSET + 4..DISCLOSURE_OFFSET]
            .iter()
            .all(|byte| *byte == 0));
        assert_eq!(statement[DISCLOSURE_OFFSET], 0);
    }
}
