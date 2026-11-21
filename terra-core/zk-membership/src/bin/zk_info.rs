//! Print the on-chain-visible identity of a Groth16 verification key.
//!
//! The chain pins `SHA-256(VK frame)` in `OwnershipRoot.verification_key_hash`
//! and the API mirrors the same value, so this binary is the single source of
//! truth when wiring a setup:
//!
//! ```text
//! cargo run -p terra-zk-membership --bin zk-info -- --vk .zk/membership.vk --json
//! ```
//!
//! `verification_key_hash` is what `update_verification_key_hash` takes;
//! `verification_key_frame` is the 576-byte encoding embedded in every
//! `proof_data` frame.

use std::path::PathBuf;

use sha2::{Digest, Sha256};
use terra_zk_membership::frame::serialize_verification_key;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut args = std::env::args_os().skip(1);
    let mut vk_path: Option<PathBuf> = None;
    let mut json = false;
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--vk") => {
                vk_path = Some(PathBuf::from(
                    args.next().ok_or("--vk requires a path argument")?,
                ))
            }
            Some("--json") => json = true,
            Some(other) => return Err(format!("unexpected argument: {other}").into()),
            None => return Err("arguments must be valid UTF-8".into()),
        }
    }
    let vk_path = vk_path.ok_or("usage: zk-info --vk <verifying-key> [--json]")?;
    let verifying_key = terra_zk_membership::load_verifying_key(&vk_path)?;
    let frame = serialize_verification_key(&verifying_key).map_err(|error| {
        format!("the verifying key is not a one-public-input Groth16 key: {error}")
    })?;
    let hash = Sha256::digest(frame);

    if json {
        println!(
            "{{\"verification_key_hash\":\"{}\",\"verification_key_frame\":\"{}\"}}",
            hex::encode(hash),
            hex::encode(frame)
        );
    } else {
        println!("verification_key_hash = {}", hex::encode(hash));
        println!("verification_key_frame = {}", hex::encode(frame));
    }
    Ok(())
}
