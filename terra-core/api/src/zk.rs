use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use ark_bn254::Bn254;
use ark_groth16::ProvingKey;
use sha2::{Digest, Sha256};
use terra_zk_membership::{frame::serialize_verification_key, VerifyingKey, TG16_VK_LEN};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZkMode {
    Development,
    Production,
}

#[derive(Clone)]
pub struct ZkRuntime {
    mode: ZkMode,
    proving_key: Option<Arc<ProvingKey<Bn254>>>,
    verifying_key: Arc<VerifyingKey<Bn254>>,
    verification_key_frame: [u8; TG16_VK_LEN],
    verification_key_hash: [u8; 32],
}

impl ZkRuntime {
    pub fn from_env() -> Result<Option<Self>> {
        let mode = match std::env::var("TERRA_ZK_MODE").as_deref() {
            Ok("development") => ZkMode::Development,
            Ok("production") => ZkMode::Production,
            Ok("disabled") | Err(_) => return Ok(None),
            Ok(other) => return Err(anyhow!("unsupported TERRA_ZK_MODE value: {other}")),
        };

        let vk_path = required_path("TERRA_ZK_VERIFYING_KEY_PATH")?;
        let verifying_key = terra_zk_membership::load_verifying_key(&vk_path).map_err(|error| {
            anyhow!(
                "loading ZK verification key from {}: {error}",
                vk_path.display()
            )
        })?;
        let verification_key_frame =
            serialize_verification_key(&verifying_key).map_err(|error| anyhow!(error))?;
        let verification_key_hash = Sha256::digest(verification_key_frame).into();

        let proving_key = if mode == ZkMode::Development {
            let path = required_path("TERRA_ZK_PROVING_KEY_PATH")?;
            let proving_key = terra_zk_membership::load_proving_key(&path).map_err(|error| {
                anyhow!("loading ZK proving key from {}: {error}", path.display())
            })?;
            if serialize_verification_key(&proving_key.vk).map_err(|error| anyhow!(error))?
                != verification_key_frame
            {
                return Err(anyhow!(
                    "configured proving and verification keys do not belong to the same setup"
                ));
            }
            Some(Arc::new(proving_key))
        } else {
            None
        };

        Ok(Some(Self {
            mode,
            proving_key,
            verifying_key: Arc::new(verifying_key),
            verification_key_frame,
            verification_key_hash,
        }))
    }

    pub fn mode(&self) -> ZkMode {
        self.mode
    }

    pub fn is_dev_prover_enabled(&self) -> bool {
        self.mode == ZkMode::Development && self.proving_key.is_some()
    }

    pub fn proving_key_arc(&self) -> Option<Arc<ProvingKey<Bn254>>> {
        self.proving_key.clone()
    }

    pub fn verifying_key(&self) -> &VerifyingKey<Bn254> {
        &self.verifying_key
    }

    pub fn verifying_key_arc(&self) -> Arc<VerifyingKey<Bn254>> {
        self.verifying_key.clone()
    }

    pub fn verification_key_frame(&self) -> &[u8; TG16_VK_LEN] {
        &self.verification_key_frame
    }

    pub fn verification_key_hash_hex(&self) -> String {
        hex::encode(self.verification_key_hash)
    }
}

fn required_path(name: &str) -> Result<std::path::PathBuf> {
    let value = std::env::var(name).with_context(|| format!("{name} is required"))?;
    let path = Path::new(&value);
    if !path.is_file() {
        return Err(anyhow!("{name} does not point to a file"));
    }
    Ok(path.to_path_buf())
}
