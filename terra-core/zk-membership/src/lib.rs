//! Development and protocol implementation for private land-membership proofs.
//!
//! Production use requires independently generated and audited Groth16 setup
//! artifacts. The API's development prover is explicitly loopback-only.

pub mod circuit;
pub mod frame;
pub mod merkle;
pub mod statement;

pub use ark_bn254::Fr;
pub use ark_groth16::{ProvingKey, VerifyingKey};
pub use circuit::MembershipCircuit;
pub use frame::{decode_frame, encode_frame, Frame, TG16_FRAME_LEN, TG16_VK_LEN};
pub use merkle::TREE_DEPTH;
pub use merkle::{
    compute_root, derive_nullifier, field_from_be, field_to_be, poseidon_hash_pair, MerklePath,
};
pub use statement::{
    canonical_ownership_statement, OWNERSHIP_PROOF_DOMAIN, OWNERSHIP_STATEMENT_LEN,
};

use ark_bn254::Bn254;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_snark::{CircuitSpecificSetupSNARK, SNARK};
use rand::thread_rng;
use std::path::Path;

/// Run a local, development-only Groth16 setup. These keys must never be
/// represented as audited or production ceremony artifacts.
pub fn generate_development_setup(
    proving_key_path: &Path,
    verifying_key_path: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let circuit = MembershipCircuit::blank();
    let (proving_key, verifying_key) =
        ark_groth16::Groth16::<Bn254>::setup(circuit, &mut thread_rng())?;

    let mut pk_bytes = Vec::new();
    proving_key.serialize_compressed(&mut pk_bytes)?;
    std::fs::write(proving_key_path, pk_bytes)?;

    let mut vk_bytes = Vec::new();
    verifying_key.serialize_compressed(&mut vk_bytes)?;
    std::fs::write(verifying_key_path, vk_bytes)?;
    Ok(())
}

pub fn load_proving_key(
    path: &Path,
) -> Result<ProvingKey<Bn254>, Box<dyn std::error::Error + Send + Sync>> {
    Ok(ProvingKey::deserialize_compressed(
        std::fs::read(path)?.as_slice(),
    )?)
}

pub fn load_verifying_key(
    path: &Path,
) -> Result<VerifyingKey<Bn254>, Box<dyn std::error::Error + Send + Sync>> {
    Ok(VerifyingKey::deserialize_compressed(
        std::fs::read(path)?.as_slice(),
    )?)
}

pub fn prove(
    proving_key: &ProvingKey<Bn254>,
    witness: MembershipWitness,
) -> Result<ark_groth16::Proof<Bn254>, Box<dyn std::error::Error + Send + Sync>> {
    Ok(ark_groth16::Groth16::<Bn254>::prove(
        proving_key,
        witness.into_circuit(),
        &mut thread_rng(),
    )?)
}

pub fn verify(
    verifying_key: &VerifyingKey<Bn254>,
    statement: &[u8],
    proof: &ark_groth16::Proof<Bn254>,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    use ark_ff::PrimeField;
    use sha2::{Digest, Sha256};

    let digest = Sha256::digest(statement);
    let mut public_input_bytes = [0u8; 32];
    public_input_bytes[1..].copy_from_slice(&digest[..31]);
    let public_input = Fr::from_be_bytes_mod_order(&public_input_bytes);
    Ok(ark_groth16::Groth16::<Bn254>::verify(
        verifying_key,
        &[public_input],
        proof,
    )?)
}

#[derive(Clone)]
pub struct MembershipWitness {
    pub statement: Vec<u8>,
    pub secret: Fr,
    pub path: MerklePath,
}

impl MembershipWitness {
    pub fn into_circuit(self) -> MembershipCircuit {
        MembershipCircuit::new(Some(self.statement), Some(self.secret), Some(self.path))
    }
}
