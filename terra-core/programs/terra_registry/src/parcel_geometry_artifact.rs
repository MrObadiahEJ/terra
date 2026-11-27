//! Q-035(a) — content-addressed parcel geometry artifacts.
//!
//! The existing `Parcel` account intentionally keeps its Borsh layout: it
//! carries only `geometry_hash` (sha256 of the client's exterior ring) and
//! never the geometry bytes. This module adds a *separate* PDA that registers
//! a content-addressed artifact (sha256 of the raw GeoJSON bytes + storage URI)
//! for that parcel. Off-chain readers can then fetch the bytes, verify
//! `sha256(bytes) == content_hash` AND re-derive the parcel's `geometry_hash`
//! from the exterior ring before materializing a chain-only parcel.
//!
//! PDA: `["parcel_geometry_artifact", parcel, content_hash]`
//! The seed binds an artifact to one parcel and one exact byte digest; the
//! account is append-only/idempotent and never changes the `Parcel` layout.

use anchor_lang::prelude::*;

use crate::{TerraError, is_authorized_holder};

/// Maximum length of a storage URI (S3 key, IPFS CID, gateway URL). Kept in
/// sync with the evidence artifact bound so API validation is consistent.
pub const MAX_STORAGE_URI_LEN: usize = 256;

#[account]
#[derive(InitSpace)]
pub struct ParcelGeometryArtifact {
    /// Parcel PDA this artifact belongs to (PDA seed).
    pub parcel: Pubkey,
    /// SHA-256 of the exact GeoJSON bytes (PDA seed).
    pub content_hash: [u8; 32],
    /// Content-addressed storage reference (S3 key / IPFS CID / gateway URL).
    #[max_len(MAX_STORAGE_URI_LEN)]
    pub storage_uri: String,
    /// Wallet that paid rent and is authorized for this parcel.
    pub registrant: Pubkey,
    pub created_at: i64,
}

/// Reject a zero digest: `sha256` of real bytes can be zero only by collision.
pub fn is_nonzero_hash(hash: &[u8; 32]) -> bool {
    !hash.iter().all(|byte| *byte == 0)
}

/// Storage URI must be non-empty and bounded (rent / abuse control).
pub fn is_valid_storage_uri(uri: &str) -> bool {
    !uri.is_empty() && uri.len() <= MAX_STORAGE_URI_LEN && !uri.chars().any(|c| c.is_control())
}

/// True when an `init_if_needed` account has already been written.
/// `created_at == 0` cannot be produced by `Clock::get()` on a live cluster.
pub fn is_initialized(artifact: &ParcelGeometryArtifact) -> bool {
    artifact.created_at != 0
}

/// Register (or idempotently confirm) the content-addressed geometry artifact
/// for an existing parcel. Only the parcel's current rights holder may write;
/// identity-PDA holders work through the same overlay as `transfer_parcel`.
pub fn register_parcel_geometry_artifact(
    ctx: Context<crate::RegisterParcelGeometryArtifact>,
    content_hash: [u8; 32],
    storage_uri: String,
) -> Result<()> {
    require!(is_nonzero_hash(&content_hash), TerraError::EmptyContentHash);
    require!(
        is_valid_storage_uri(&storage_uri),
        TerraError::EmptyStorageReference
    );

    let parcel_key = ctx.accounts.parcel.key();
    is_authorized_holder(
        &ctx.accounts.ownership,
        parcel_key,
        ctx.remaining_accounts,
        ctx.accounts.authority.key(),
    )?;

    let artifact = &mut ctx.accounts.artifact;
    let reused = is_initialized(artifact);
    if reused {
        // Same PDA seed already exists. Never allow a different URI to be
        // substituted for a digest: the digest must keep identifying one
        // immutable byte stream.
        require!(
            artifact.parcel == parcel_key && artifact.content_hash == content_hash,
            TerraError::GeometryArtifactMismatch
        );
        require!(
            artifact.storage_uri == storage_uri,
            TerraError::GeometryArtifactMismatch
        );
    } else {
        let now = Clock::get()?.unix_timestamp;
        artifact.parcel = parcel_key;
        artifact.content_hash = content_hash;
        artifact.storage_uri = storage_uri.clone();
        artifact.registrant = ctx.accounts.authority.key();
        artifact.created_at = now;
    }

    emit!(crate::ParcelGeometryArtifactRegistered {
        artifact: artifact.key(),
        parcel: parcel_key,
        content_hash,
        storage_uri,
        registrant: ctx.accounts.authority.key(),
        reused,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_hash() {
        assert!(!is_nonzero_hash(&[0u8; 32]));
        let mut hash = [0u8; 32];
        hash[31] = 1;
        assert!(is_nonzero_hash(&hash));
    }

    #[test]
    fn storage_uri_bounds_and_controls() {
        assert!(is_valid_storage_uri("content/sha256/aa/bb/abcd"));
        assert!(!is_valid_storage_uri(""));
        assert!(!is_valid_storage_uri("line\nbreak"));
        assert!(!is_valid_storage_uri(&"x".repeat(MAX_STORAGE_URI_LEN + 1)));
        assert!(is_valid_storage_uri(&"x".repeat(MAX_STORAGE_URI_LEN)));
    }

    #[test]
    fn initialization_marker_is_stable() {
        let mut artifact = ParcelGeometryArtifact {
            parcel: Pubkey::default(),
            content_hash: [0u8; 32],
            storage_uri: String::new(),
            registrant: Pubkey::default(),
            created_at: 0,
        };
        assert!(!is_initialized(&artifact));
        artifact.created_at = 1;
        assert!(is_initialized(&artifact));
    }
}
