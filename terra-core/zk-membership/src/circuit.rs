use ark_bn254::Fr;
use ark_crypto_primitives::crh::sha256::Sha256;
use ark_crypto_primitives::{
    crh::{sha256::constraints::Sha256Gadget, CRHScheme},
    sponge::poseidon::PoseidonConfig,
};
use ark_ff::PrimeField;
use ark_r1cs_std::prelude::ToBitsGadget;
use ark_r1cs_std::{
    alloc::AllocVar,
    boolean::Boolean,
    eq::EqGadget,
    fields::{fp::FpVar, FieldVar},
    select::CondSelectGadget,
    uint8::UInt8,
};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_std::vec::Vec;
use light_poseidon::parameters::bn254_x5::get_poseidon_parameters;

use crate::merkle::{LEAF_DOMAIN, TREE_DEPTH};
use crate::statement::{
    DISCLOSURE_OFFSET, NULLIFIER_OFFSET, OWNERSHIP_PROOF_DOMAIN, OWNERSHIP_STATEMENT_LEN,
    ROOT_OFFSET, VERSION_OFFSET, ZONE_OFFSET,
};

#[derive(Clone)]
pub struct MembershipCircuit {
    pub statement: Option<Vec<u8>>,
    pub secret: Option<Fr>,
    pub siblings: Option<Vec<Fr>>,
    pub indices: Option<Vec<bool>>,
}

impl MembershipCircuit {
    pub fn blank() -> Self {
        Self {
            statement: None,
            secret: None,
            siblings: None,
            indices: None,
        }
    }

    pub fn new(
        statement: Option<Vec<u8>>,
        secret: Option<Fr>,
        path: Option<crate::MerklePath>,
    ) -> Self {
        Self {
            statement,
            secret,
            siblings: path.as_ref().map(|p| p.siblings.clone()),
            indices: path.map(|p| p.indices),
        }
    }
}

impl ConstraintSynthesizer<Fr> for MembershipCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let public_input = FpVar::new_input(cs.clone(), || {
            let bytes = self
                .statement
                .as_ref()
                .ok_or(SynthesisError::AssignmentMissing)?;
            let digest =
                Sha256::evaluate(&(), bytes.clone()).map_err(|_| SynthesisError::Unsatisfiable)?;
            let mut input = [0u8; 32];
            input[1..].copy_from_slice(&digest[..31]);
            Ok(Fr::from_be_bytes_mod_order(&input))
        })?;

        let statement_bytes = (0..OWNERSHIP_STATEMENT_LEN)
            .map(|i| {
                UInt8::new_witness(cs.clone(), || {
                    self.statement
                        .as_ref()
                        .and_then(|statement| statement.get(i).copied())
                        .ok_or(SynthesisError::AssignmentMissing)
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        if let Some(statement) = &self.statement {
            if statement.len() != OWNERSHIP_STATEMENT_LEN {
                return Err(SynthesisError::Unsatisfiable);
            }
        }
        for (byte, expected) in statement_bytes
            .iter()
            .zip(OWNERSHIP_PROOF_DOMAIN.iter().copied())
        {
            byte.enforce_equal(&UInt8::constant(expected))?;
        }
        statement_bytes[DISCLOSURE_OFFSET].enforce_equal(&UInt8::constant(0))?;

        let mut sha256 = Sha256Gadget::<Fr>::default();
        sha256.update(&statement_bytes)?;
        let digest = sha256.finalize()?;
        let digest_bits = bytes_be_to_field_bits(&digest.0[..31])?;
        Boolean::le_bits_to_fp(&digest_bits)?.enforce_equal(&public_input)?;

        let root = field_from_be_bytes(&statement_bytes[ROOT_OFFSET..ROOT_OFFSET + 32])?;
        enforce_canonical_field_encoding(&root, &statement_bytes[ROOT_OFFSET..ROOT_OFFSET + 32])?;
        let zone = field_from_be_bytes(&statement_bytes[ZONE_OFFSET..ZONE_OFFSET + 32])?;
        let version = field_from_le_bytes(&statement_bytes[VERSION_OFFSET..VERSION_OFFSET + 4])?;
        let nullifier =
            field_from_be_bytes(&statement_bytes[NULLIFIER_OFFSET..NULLIFIER_OFFSET + 32])?;
        enforce_canonical_field_encoding(
            &nullifier,
            &statement_bytes[NULLIFIER_OFFSET..NULLIFIER_OFFSET + 32],
        )?;

        let secret = FpVar::new_witness(cs.clone(), || {
            self.secret.ok_or(SynthesisError::AssignmentMissing)
        })?;
        secret.is_zero()?.enforce_equal(&Boolean::FALSE)?;
        let parameters = poseidon_config()?;

        let mut current = poseidon_pair_gadget(
            &parameters,
            &secret,
            &FpVar::constant(Fr::from(LEAF_DOMAIN)),
        )?;
        if self
            .siblings
            .as_ref()
            .is_some_and(|values| values.len() != TREE_DEPTH)
            || self
                .indices
                .as_ref()
                .is_some_and(|values| values.len() != TREE_DEPTH)
        {
            return Err(SynthesisError::Unsatisfiable);
        }
        for level in 0..TREE_DEPTH {
            let sibling = FpVar::new_witness(cs.clone(), || {
                self.siblings
                    .as_ref()
                    .and_then(|values| values.get(level).copied())
                    .ok_or(SynthesisError::AssignmentMissing)
            })?;
            let index = Boolean::new_witness(cs.clone(), || {
                self.indices
                    .as_ref()
                    .and_then(|values| values.get(level).copied())
                    .ok_or(SynthesisError::AssignmentMissing)
            })?;
            let left = FpVar::conditionally_select(&index, &sibling, &current)?;
            let right = FpVar::conditionally_select(&index, &current, &sibling)?;
            current = poseidon_pair_gadget(&parameters, &left, &right)?;
        }
        current.enforce_equal(&root)?;

        let zone_version_scope = poseidon_pair_gadget(
            &parameters,
            &poseidon_pair_gadget(&parameters, &zone, &root)?,
            &version,
        )?;
        let expected_nullifier = poseidon_pair_gadget(&parameters, &secret, &zone_version_scope)?;
        expected_nullifier.enforce_equal(&nullifier)?;
        Ok(())
    }
}

fn poseidon_config() -> Result<PoseidonConfig<Fr>, SynthesisError> {
    let params = get_poseidon_parameters::<Fr>(3).map_err(|_| SynthesisError::Unsatisfiable)?;
    let ark = params
        .ark
        .chunks(params.width)
        .map(|round| round.to_vec())
        .collect();
    Ok(PoseidonConfig::new(
        params.full_rounds,
        params.partial_rounds,
        params.alpha,
        params.mds,
        ark,
        2,
        1,
    ))
}

fn poseidon_pair_gadget(
    params: &PoseidonConfig<Fr>,
    left: &FpVar<Fr>,
    right: &FpVar<Fr>,
) -> Result<FpVar<Fr>, SynthesisError> {
    let mut state = vec![FpVar::zero(), left.clone(), right.clone()];
    let half_full_rounds = params.full_rounds / 2;
    for round in 0..params.full_rounds + params.partial_rounds {
        for (item, constant) in state.iter_mut().zip(&params.ark[round]) {
            *item += *constant;
        }
        if round < half_full_rounds || round >= half_full_rounds + params.partial_rounds {
            for item in &mut state {
                let square = item.square()?;
                *item = square.square()? * item.clone();
            }
        } else {
            let square = state[0].square()?;
            state[0] = square.square()? * state[0].clone();
        }
        let mut mixed = Vec::with_capacity(state.len());
        for row in &params.mds {
            let value = state
                .iter()
                .zip(row)
                .fold(FpVar::zero(), |sum, (item, coefficient)| {
                    sum + item * *coefficient
                });
            mixed.push(value);
        }
        state = mixed;
    }
    Ok(state[0].clone())
}

fn field_from_be_bytes(bytes: &[UInt8<Fr>]) -> Result<FpVar<Fr>, SynthesisError> {
    let mut bits = Vec::with_capacity(bytes.len() * 8);
    for byte in bytes.iter().rev() {
        bits.extend(byte.to_bits_le()?);
    }
    Boolean::le_bits_to_fp(&bits)
}

fn field_from_le_bytes(bytes: &[UInt8<Fr>]) -> Result<FpVar<Fr>, SynthesisError> {
    let mut bits = Vec::with_capacity(bytes.len() * 8);
    for byte in bytes {
        bits.extend(byte.to_bits_le()?);
    }
    Boolean::le_bits_to_fp(&bits)
}

fn bytes_be_to_field_bits(bytes: &[UInt8<Fr>]) -> Result<Vec<Boolean<Fr>>, SynthesisError> {
    let mut bits = Vec::with_capacity(bytes.len() * 8);
    for byte in bytes.iter().rev() {
        bits.extend(byte.to_bits_le()?);
    }
    Ok(bits)
}

fn enforce_canonical_field_encoding(
    field: &FpVar<Fr>,
    bytes_be: &[UInt8<Fr>],
) -> Result<(), SynthesisError> {
    let mut expected_bits = field.to_bits_le()?;
    expected_bits.resize(256, Boolean::FALSE);
    if expected_bits.len() > 256 || bytes_be.len() != 32 {
        return Err(SynthesisError::Unsatisfiable);
    }
    for (byte_index, byte) in bytes_be.iter().enumerate() {
        let actual = byte.to_bits_le()?;
        let start = (31 - byte_index) * 8;
        for (actual_bit, expected_bit) in actual.iter().zip(&expected_bits[start..start + 8]) {
            actual_bit.enforce_equal(expected_bit)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{canonical_ownership_statement, derive_nullifier, field_to_be, Fr, MerklePath};
    use ark_r1cs_std::R1CSVar;
    use ark_relations::r1cs::ConstraintSystem;

    fn fixture() -> (Vec<u8>, Fr, MerklePath) {
        let secret = Fr::from(123456u64);
        let zone_bytes = [7u8; 32];
        let zone = Fr::from_be_bytes_mod_order(&zone_bytes);
        let path = MerklePath {
            siblings: (0..TREE_DEPTH)
                .map(|level| Fr::from((level + 1) as u64))
                .collect(),
            indices: (0..TREE_DEPTH).map(|level| level % 2 == 1).collect(),
        };
        let root = crate::merkle::compute_root(secret, &path).unwrap();
        let root_bytes = field_to_be(&root);
        let nullifier = derive_nullifier(secret, zone, root, 3).unwrap();
        let statement = canonical_ownership_statement(
            &zone_bytes,
            &root_bytes,
            3,
            &field_to_be(&nullifier),
            &[8u8; 32],
            "membership",
            0,
        )
        .unwrap();
        (statement, secret, path)
    }

    #[test]
    fn enforces_private_membership_and_statement_binding() {
        let (statement, secret, path) = fixture();
        let cs = ConstraintSystem::<Fr>::new_ref();
        MembershipCircuit::new(Some(statement.clone()), Some(secret), Some(path.clone()))
            .generate_constraints(cs.clone())
            .unwrap();
        assert!(
            cs.is_satisfied().unwrap(),
            "unsatisfied constraint: {:?}",
            cs.which_is_unsatisfied().unwrap()
        );

        let mut changed_statement = statement;
        changed_statement[crate::statement::ROOT_OFFSET + 31] ^= 1;
        let wrong_root_cs = ConstraintSystem::<Fr>::new_ref();
        MembershipCircuit::new(Some(changed_statement), Some(secret), Some(path.clone()))
            .generate_constraints(wrong_root_cs.clone())
            .unwrap();
        assert!(!wrong_root_cs.is_satisfied().unwrap());

        let mut wrong_path = path;
        wrong_path.siblings[0] += Fr::from(1u64);
        let wrong_path_cs = ConstraintSystem::<Fr>::new_ref();
        MembershipCircuit::new(Some(fixture().0), Some(secret), Some(wrong_path))
            .generate_constraints(wrong_path_cs.clone())
            .unwrap();
        assert!(!wrong_path_cs.is_satisfied().unwrap());
    }

    #[test]
    fn sha256_gadget_supports_fixed_statement_length() {
        let (statement, _, _) = fixture();
        let cs = ConstraintSystem::<Fr>::new_ref();
        let bytes = UInt8::new_witness_vec(cs.clone(), &statement).unwrap();
        let mut hash = Sha256Gadget::<Fr>::default();
        hash.update(&bytes).unwrap();
        let digest = hash.finalize().unwrap();
        assert!(
            cs.is_satisfied().unwrap(),
            "{:?}",
            cs.which_is_unsatisfied()
        );
        let field_bits = bytes_be_to_field_bits(&digest.0[..31]).unwrap();
        let field_hash = Boolean::le_bits_to_fp(&field_bits).unwrap();
        assert_eq!(
            digest.value().unwrap().to_vec(),
            Sha256::evaluate(&(), statement).unwrap()
        );
        let mut expected_hash = [0u8; 32];
        expected_hash[1..].copy_from_slice(&digest.value().unwrap()[..31]);
        assert_eq!(
            field_hash.value().unwrap(),
            Fr::from_be_bytes_mod_order(&expected_hash)
        );
        assert!(
            cs.is_satisfied().unwrap(),
            "{:?}",
            cs.which_is_unsatisfied()
        );
    }
}
