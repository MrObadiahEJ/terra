use ark_bn254::Fr;
use ark_ff::{BigInteger, PrimeField};
use light_poseidon::{Poseidon, PoseidonHasher};

pub const TREE_DEPTH: usize = 20;
pub const LEAF_DOMAIN: u64 = 1;

#[derive(Clone, Debug)]
pub struct MerklePath {
    pub siblings: Vec<Fr>,
    /// `true` means the current node is the right child at that level.
    pub indices: Vec<bool>,
}

pub fn field_from_be(bytes: &[u8; 32]) -> Result<Fr, &'static str> {
    let field = Fr::from_be_bytes_mod_order(bytes);
    if field_to_be(&field) != *bytes {
        return Err("field element is not canonically encoded");
    }
    Ok(field)
}

pub fn field_to_be(value: &Fr) -> [u8; 32] {
    let bytes = value.into_bigint().to_bytes_be();
    let mut out = [0u8; 32];
    out[32 - bytes.len()..].copy_from_slice(&bytes);
    out
}

pub fn poseidon_hash_pair(left: Fr, right: Fr) -> Result<Fr, light_poseidon::PoseidonError> {
    Poseidon::<Fr>::new_circom(2)?.hash(&[left, right])
}

pub fn compute_root(
    secret: Fr,
    path: &MerklePath,
) -> Result<Fr, Box<dyn std::error::Error + Send + Sync>> {
    if path.siblings.len() != TREE_DEPTH || path.indices.len() != TREE_DEPTH {
        return Err(format!("membership path must have exactly {TREE_DEPTH} levels").into());
    }
    let mut current = poseidon_hash_pair(secret, Fr::from(LEAF_DOMAIN))?;
    for (sibling, is_right) in path
        .siblings
        .iter()
        .copied()
        .zip(path.indices.iter().copied())
    {
        current = if is_right {
            poseidon_hash_pair(sibling, current)?
        } else {
            poseidon_hash_pair(current, sibling)?
        };
    }
    Ok(current)
}

/// Derive a nullifier scoped to the zone, published root, and root version.
/// A holder can use a given membership credential only once per root version.
pub fn derive_nullifier(
    secret: Fr,
    zone: Fr,
    root: Fr,
    version: u32,
) -> Result<Fr, light_poseidon::PoseidonError> {
    let scope = poseidon_hash_pair(poseidon_hash_pair(zone, root)?, Fr::from(version as u64))?;
    poseidon_hash_pair(secret, scope)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circom_poseidon_reference_vector() {
        let result = poseidon_hash_pair(Fr::from(1u64), Fr::from(2u64)).unwrap();
        assert_eq!(
            hex::encode(field_to_be(&result)),
            "115cc0f5e7d690413df64c6b9662e9cf2a3617f2743245519e19607a4417189a"
        );
    }

    #[test]
    fn path_depth_and_field_encoding_are_strict() {
        let empty = MerklePath {
            siblings: vec![Fr::from(0u64); TREE_DEPTH],
            indices: vec![false; TREE_DEPTH],
        };
        assert!(compute_root(Fr::from(9u64), &empty).is_ok());
        assert!(compute_root(
            Fr::from(9u64),
            &MerklePath {
                siblings: vec![],
                indices: vec![],
            }
        )
        .is_err());

        let value = field_to_be(&Fr::from(42u64));
        assert_eq!(field_from_be(&value), Ok(Fr::from(42u64)));
        assert!(field_from_be(&[0xff; 32]).is_err());
    }
}
