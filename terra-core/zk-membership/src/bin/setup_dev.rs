use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut args = std::env::args_os().skip(1);
    let proving_key = args.next().map(PathBuf::from).ok_or(
        "usage: cargo run -p terra-zk-membership --bin setup-dev -- <proving-key> <verifying-key>",
    )?;
    let verifying_key = args.next().map(PathBuf::from).ok_or(
        "usage: cargo run -p terra-zk-membership --bin setup-dev -- <proving-key> <verifying-key>",
    )?;
    if args.next().is_some() {
        return Err("expected exactly two output paths".into());
    }
    if proving_key.exists() || verifying_key.exists() {
        return Err("refusing to overwrite existing ZK setup artifacts".into());
    }
    for path in [&proving_key, &verifying_key] {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
    }

    terra_zk_membership::generate_development_setup(&proving_key, &verifying_key)?;
    eprintln!(
        "Generated local DEVELOPMENT-ONLY Groth16 keys. Never use these as production ceremony artifacts."
    );
    Ok(())
}
