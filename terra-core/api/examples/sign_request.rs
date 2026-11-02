//! Signed-request header emitter (RISK R-017).
//!
//! A tiny, shippable client for the API's Ed25519 request signing scheme. It
//! is deliberately *only* a header emitter: it computes the signed payload and
//! prints the headers, so smoke scripts and CI can drive `curl` (or any HTTP
//! client) without hand-rolling crypto. See `docs/book-of-intents/AGENT_GUIDE.md`
//! for the signing contract.
//!
//! Usage:
//! ```text
//! SIGNING_KEY=<base58 seed | base58 64-byte secret> \
//!   cargo run -p terra-api --example sign_request -- <METHOD> <PATH> [BODY]
//! ```
//!
//! Output (one `Name: value` per line):
//! ```text
//! Authorization: Signature <base64 ed25519 signature>
//! x-timestamp: <unix seconds>
//! x-body-hash: <sha256 hex of the body>
//! x-wallet: <base58 ed25519 public key>   (only with SIGNING_KEY)
//! ```
//!
//! The signed payload is `{METHOD}\n{PATH}\n{BODY_SHA256_HEX}\n{TIMESTAMP}`,
//! matching `auth::SignedRequest::verify`. Requests are accepted within ±60 s
//! of the timestamp, so emit headers immediately before sending.

use base64::Engine;
use ed25519_dalek::Signer;
use sha2::{Digest, Sha256};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!(
            "usage: SIGNING_KEY=<base58 seed> cargo run -p terra-api --example sign_request -- \
             <METHOD> <PATH> [BODY]"
        );
        std::process::exit(2);
    }
    let method = args[0].to_uppercase();
    let path = &args[1];
    let body = args.get(2).map(String::as_str).unwrap_or("");
    let timestamp = chrono::Utc::now().timestamp();

    let body_hash = hex::encode(Sha256::digest(body.as_bytes()));
    println!("x-body-hash: {body_hash}");
    println!("x-timestamp: {timestamp}");

    // The signature is optional: endpoints that only require a well-formed
    // SignedRequest accept it, privileged ones will reject an unsigned call
    // with 401 (fail-closed). Emitting the hash/timestamp anyway keeps the
    // client useful for negative tests.
    let Some(key_b58) = std::env::var("SIGNING_KEY").ok().filter(|k| !k.is_empty()) else {
        return;
    };

    let secret = bs58::decode(&key_b58)
        .into_vec()
        .unwrap_or_else(|_| fail("SIGNING_KEY is not valid base58"));
    let seed: [u8; 32] = match secret.len() {
        32 => secret.as_slice().try_into().expect("checked length"),
        64 => secret[..32].try_into().expect("checked length"),
        n => fail(&format!(
            "SIGNING_KEY must decode to 32 or 64 bytes, got {n}"
        )),
    };
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&seed);
    let wallet = bs58::encode(signing_key.verifying_key().to_bytes()).into_string();

    let payload = format!("{method}\n{path}\n{body_hash}\n{timestamp}");
    let signature = signing_key.sign(payload.as_bytes());
    let signature_b64 = base64::engine::general_purpose::STANDARD.encode(signature.to_bytes());

    println!("Authorization: Signature {signature_b64}");
    println!("x-wallet: {wallet}");
}

fn fail(msg: &str) -> ! {
    eprintln!("error: {msg}");
    std::process::exit(1)
}
