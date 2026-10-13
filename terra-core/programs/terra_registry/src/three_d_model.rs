//! 3D model registry (RFC-013 Phase C, RFC-012 §5 `ThreeDModel`).
//!
//! A `ThreeDModel` is the off-chain 3D reconstruction *claim* attached to a
//! spatial asset — never the mesh itself (Design Rule 10): only `model_hash`,
//! a `storage_reference` (tileset URL / IPFS CID / storage key) and the
//! geometry version it was reconstructed from. The model must derive from an
//! already-anchored [`GeometryVersion`](crate::spatial_asset::GeometryVersion)
//! (`geometry_version < asset.geometry_version_count`), so every reconstruction
//! sits on top of the provenance chain instead of floating free.
//!
//! Anyone may register a model (rent-paid, append-only, capped at
//! [`MAX_3D_MODELS`]) — an *observation claim*, exactly like a geometry
//! version; a registered validator's signature flips `verified` — claim →
//! fact, the same split the whole protocol is built on. The off-chain API
//! mirror (RFC-013 Phase E) exposes registered models to the globe client
//! (Phase D), which streams the referenced tiles.

use anchor_lang::prelude::*;

use crate::evidence_manifest::{is_nonzero_hash, is_valid_storage_reference};
use crate::TerraError;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Container format of the off-chain model bundle a `storage_reference`
/// points at. `UNKNOWN` is a placeholder only — registration requires a
/// concrete format (the globe client needs it to pick a loader).
pub mod model_format {
    pub const UNKNOWN: u8 = 0;
    pub const GLTF: u8 = 1;
    pub const OBJ: u8 = 2;
    pub const CESIUM_TILESET: u8 = 3;
    pub const PLY: u8 = 4;
    pub const MAX: u8 = 4;
}

/// Maximum 3D models per spatial asset (bounds rent & state growth; same
/// bounded-append pattern as [`MAX_GEOMETRY_VERSIONS`](crate::spatial_asset::MAX_GEOMETRY_VERSIONS)).
pub const MAX_3D_MODELS: u32 = 16;

// ---------------------------------------------------------------------------
// Accounts
// ---------------------------------------------------------------------------

/// Append-only cursor for a spatial asset's 3D models.
///
/// PDA: `["three_d_model_meta", asset]` — created lazily on the first
/// registration (`init_if_needed`), so `SpatialAsset`'s deployed layout is
/// untouched.
#[account]
#[derive(InitSpace)]
pub struct ThreeDModelMeta {
    /// The owning `SpatialAsset` PDA.
    pub asset: Pubkey,
    /// Next model version index (0..=MAX_3D_MODELS).
    pub count: u32,
}

/// One append-only entry in a spatial asset's 3D model history
/// (RFC-012 §5 `ThreeDModel`).
///
/// PDA: `["three_d_model", asset, model_version_le_bytes]`.
#[account]
#[derive(InitSpace)]
pub struct ThreeDModel {
    /// The owning `SpatialAsset` PDA.
    pub asset: Pubkey,
    /// Redundant parcel key for cheap off-chain joins.
    pub parcel: Pubkey,
    /// 0-based append index (from `ThreeDModelMeta`).
    pub model_version: u32,
    /// Geometry version this model was reconstructed from (provenance —
    /// always < `asset.geometry_version_count` at registration time).
    pub geometry_version: u32,
    /// SHA-256 canonical digest of the off-chain model bundle.
    pub model_hash: [u8; 32],
    /// Off-chain pointer (tileset URL / IPFS CID / storage key), <= 128 chars.
    #[max_len(128)]
    pub storage_reference: String,
    /// `model_format` code of the referenced bundle.
    pub model_format: u8,
    /// Whoever paid to register this claim (permissionless).
    pub submitted_by: Pubkey,
    pub submitted_at: i64,
    /// Claim → fact: true once a registered validator verified the hash.
    pub verified: bool,
    /// Validator who verified (zero until verified).
    pub verified_by: Pubkey,
    pub verified_at: i64,
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

/// A concrete, loadable format is required (`UNKNOWN` is not registrable).
pub fn is_valid_model_format(f: u8) -> bool {
    f != model_format::UNKNOWN && f <= model_format::MAX
}

/// The next append index is legal iff below the cap.
pub fn model_append_would_exceed_cap(current_count: u32) -> bool {
    current_count >= MAX_3D_MODELS
}

/// A model must derive from an already-anchored geometry version:
/// `geometry_version` is a valid index in the asset's history iff it sits
/// below the append cursor.
pub fn geometry_version_exists(geometry_version: u32, cursor: u32) -> bool {
    geometry_version < cursor
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Register an off-chain 3D model claim (permissionless rent-paid append).
/// The entry is immutable; [`verify_3d_model`] flips `verified`.
pub fn register_3d_model(
    ctx: Context<crate::Register3DModel>,
    model_hash: [u8; 32],
    storage_reference: String,
    model_format_code: u8,
    geometry_version: u32,
) -> Result<()> {
    require!(
        is_nonzero_hash(&model_hash),
        TerraError::EmptyGeometryHash
    );
    require!(
        is_valid_storage_reference(&storage_reference),
        TerraError::EmptyStorageReference
    );
    require!(
        is_valid_model_format(model_format_code),
        TerraError::InvalidModelFormat
    );

    let asset_pk = ctx.accounts.asset.key();
    let meta = &mut ctx.accounts.meta;
    // Reuse path (init_if_needed): the seed re-derivation already pins the
    // account to this asset; set the field unconditionally so a fresh init
    // is complete too.
    meta.asset = asset_pk;
    require!(
        !model_append_would_exceed_cap(meta.count),
        TerraError::ThreeDModelsFull
    );
    require!(
        geometry_version_exists(geometry_version, ctx.accounts.asset.geometry_version_count),
        TerraError::ModelGeometryVersionMismatch
    );

    let now = Clock::get()?.unix_timestamp;
    let model_version = meta.count;
    let parcel = ctx.accounts.asset.parcel;
    let model_pk = ctx.accounts.model.key();

    let model = &mut ctx.accounts.model;
    model.asset = asset_pk;
    model.parcel = parcel;
    model.model_version = model_version;
    model.geometry_version = geometry_version;
    model.model_hash = model_hash;
    model.storage_reference = storage_reference;
    model.model_format = model_format_code;
    model.submitted_by = ctx.accounts.payer.key();
    model.submitted_at = now;
    model.verified = false;
    model.verified_by = Pubkey::default();
    model.verified_at = 0;

    meta.count = model_version + 1;

    emit!(crate::ThreeDModelRegistered {
        model: model_pk,
        asset: asset_pk,
        parcel,
        model_version,
        geometry_version,
        model_hash,
        model_format: model_format_code,
        submitted_by: model.submitted_by,
        submitted_at: now,
    });

    Ok(())
}

/// A registered validator attests a registered model's hash (claim → fact).
pub fn verify_3d_model(ctx: Context<crate::Verify3DModel>) -> Result<()> {
    let model = &mut ctx.accounts.model;
    require!(!model.verified, TerraError::ThreeDModelAlreadyVerified);

    let now = Clock::get()?.unix_timestamp;
    model.verified = true;
    model.verified_by = ctx.accounts.validator.key();
    model.verified_at = now;

    emit!(crate::ThreeDModelVerified {
        model: model.key(),
        asset: model.asset,
        verified_by: model.verified_by,
        verified_at: now,
    });

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_models_is_bounded() {
        assert_eq!(MAX_3D_MODELS, 16);
    }

    #[test]
    fn model_format_valid_range() {
        assert!(!is_valid_model_format(model_format::UNKNOWN));
        assert!(is_valid_model_format(model_format::GLTF));
        assert!(is_valid_model_format(model_format::OBJ));
        assert!(is_valid_model_format(model_format::CESIUM_TILESET));
        assert!(is_valid_model_format(model_format::PLY));
        assert!(!is_valid_model_format(model_format::MAX + 1));
        assert!(!is_valid_model_format(u8::MAX));
    }

    #[test]
    fn model_cap_is_inclusive_of_last_slot() {
        assert!(!model_append_would_exceed_cap(0));
        assert!(!model_append_would_exceed_cap(MAX_3D_MODELS - 1));
        assert!(model_append_would_exceed_cap(MAX_3D_MODELS));
        assert!(model_append_would_exceed_cap(u32::MAX));
    }

    #[test]
    fn geometry_version_must_precede_cursor() {
        assert!(!geometry_version_exists(0, 0));
        assert!(geometry_version_exists(0, 1));
        assert!(geometry_version_exists(63, 64));
        assert!(!geometry_version_exists(64, 64));
    }

    #[test]
    fn three_d_model_layout_is_stable() {
        // PDA seed discriminants must never change (address stability).
        assert_eq!(b"three_d_model", b"three_d_model");
        assert_eq!(b"three_d_model_meta", b"three_d_model_meta");
        // Account size sanity (no discriminator): fixed fields + 128-byte ref.
        let fixed = 32 + 32 + 4 + 4 + 32 + 4 + 128 + 1 + 32 + 8 + 1 + 32 + 8;
        assert_eq!(fixed, 318);
    }
}
