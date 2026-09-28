//! Spatial assets & versioned geometry (RFC-013, Vision Stage 3).
//!
//! Implements the `SpatialAsset` / `GeometryVersion` entities specified in
//! RFC-012 §5 (Design Rules 9 & 10):
//!
//! * **2D and 3D are native** — an asset declares its dimensionality
//!   (2D / 2.5D / 3D) and elevation range; geometry versions may cover
//!   observation layers at or below that dimensionality.
//! * **Never on-chain geometry** — only `geometry_hash`, provenance
//!   (`source`), `storage_reference` (PostGIS/S3/IPFS pointer) and versions.
//!
//! An asset is the spatial extension of a single `Parcel` (PDA
//! `["spatial_asset", parcel]`, one per parcel). Geometry versions are an
//! append-only, rent-paid log (`["geometry_version", asset, version]`,
//! version = `asset.geometry_version_count` at append time, capped at
//! [`MAX_GEOMETRY_VERSIONS`]). Anyone may anchor a version (an *observation
//! claim*, exactly like an evidence manifest artifact); a registered
//! validator's signature flips `verified` — the same claim → fact split the
//! whole protocol is built on. No version can ever be mutated or deleted.

use anchor_lang::prelude::*;

use crate::evidence_manifest::{is_nonzero_hash, is_valid_storage_reference};
use crate::TerraError;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Highest spatial dimensionality an asset supports (RFC-012 Design Rule 9).
pub mod spatial_dimension {
    /// Flat 2D boundary (cadastral polygon).
    pub const D2: u8 = 0;
    /// 2.5D — 2D boundary plus elevation/terrain.
    pub const D2_5: u8 = 1;
    /// Full 3D — buildings, floors, air/underground rights.
    pub const D3: u8 = 2;
    pub const MAX: u8 = 2;
}

/// Provenance of a geometry version (who/what produced the observation).
pub mod geometry_source {
    pub const MANUAL: u8 = 0;
    pub const SURVEY: u8 = 1;
    pub const DRONE_PHOTO: u8 = 2;
    pub const SATELLITE: u8 = 3;
    pub const LIDAR: u8 = 4;
    pub const PHOTOGRAMMETRY: u8 = 5;
    pub const AI_SEGMENTATION: u8 = 6;
    pub const API_IMPORT: u8 = 7;
    pub const MAX: u8 = 7;
}

/// Maximum geometry versions per spatial asset (bounds rent & state growth;
/// mirrors `MAX_MANIFEST_ARTIFACTS`' bounded-append pattern).
pub const MAX_GEOMETRY_VERSIONS: u32 = 64;

// ---------------------------------------------------------------------------
// Accounts
// ---------------------------------------------------------------------------

/// The 3D/4D extension of a parcel (RFC-012 §5 `SpatialAsset`).
///
/// PDA: `["spatial_asset", parcel]` — exactly one per parcel.
#[account]
#[derive(InitSpace)]
pub struct SpatialAsset {
    /// The Parcel PDA this asset extends.
    pub parcel: Pubkey,
    /// Registrar that initialized the asset (future lifecycle authority).
    pub authority: Pubkey,
    /// Highest supported dimensionality: `spatial_dimension`.
    pub dimensionality: u8,
    /// Minimum elevation of the asset envelope, millimetres (may be < 0).
    pub elevation_min_mm: i32,
    /// Maximum elevation of the asset envelope, millimetres (>= min).
    pub elevation_max_mm: i32,
    /// Append-only cursor: next version index (0..=MAX_GEOMETRY_VERSIONS).
    pub geometry_version_count: u32,
    /// Geometry hash of the newest version (all-zero until the first append).
    pub latest_geometry: [u8; 32],
    pub created_at: i64,
    pub updated_at: i64,
}

/// One append-only entry in a spatial asset's geometry history
/// (RFC-012 §5 `GeometryVersion`).
///
/// PDA: `["geometry_version", asset, version_le_bytes]`.
#[account]
#[derive(InitSpace)]
pub struct GeometryVersion {
    /// The owning `SpatialAsset` PDA.
    pub asset: Pubkey,
    /// Redundant parcel key for cheap off-chain joins.
    pub parcel: Pubkey,
    /// 0-based append index.
    pub version: u32,
    /// SHA-256 canonical digest of the off-chain geometry document.
    pub geometry_hash: [u8; 32],
    /// `geometry_source` provenance code.
    pub source: u8,
    /// `spatial_dimension` of this observation layer.
    pub dimension: u8,
    /// Off-chain pointer (PostGIS row / IPFS CID / storage URL), <= 128 chars.
    #[max_len(128)]
    pub storage_reference: String,
    /// Whoever paid to anchor this claim (permissionless).
    pub submitted_by: Pubkey,
    pub submitted_at: i64,
    /// Claim → fact: true once a registered validator verified it.
    pub verified: bool,
    /// Verifying validator wallet (zero until verified).
    pub verified_by: Pubkey,
    pub verified_at: i64,
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

pub fn is_valid_dimension(d: u8) -> bool {
    d <= spatial_dimension::MAX
}

pub fn is_valid_geometry_source(s: u8) -> bool {
    s <= geometry_source::MAX
}

/// An elevation envelope must be non-inverted (min <= max); equal bounds are
/// legal (a flat plane).
pub fn elevation_range_ok(min_mm: i32, max_mm: i32) -> bool {
    min_mm <= max_mm
}

/// A geometry version may not exceed the asset's declared dimensionality.
pub fn dimension_within_asset(version_dim: u8, asset_dim: u8) -> bool {
    version_dim <= asset_dim
}

/// The next append index is legal iff below the cap.
pub fn append_would_exceed_cap(current_count: u32) -> bool {
    current_count >= MAX_GEOMETRY_VERSIONS
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Initialize the spatial asset for an existing parcel (permissionless:
/// the asset is an anchor, not authority — it grants no rights).
pub fn init_spatial_asset(
    ctx: Context<crate::InitSpatialAsset>,
    dimensionality: u8,
    elevation_min_mm: i32,
    elevation_max_mm: i32,
) -> Result<()> {
    require!(
        is_valid_dimension(dimensionality),
        TerraError::InvalidSpatialDimension
    );
    require!(
        elevation_range_ok(elevation_min_mm, elevation_max_mm),
        TerraError::InvalidElevationRange
    );

    let now = Clock::get()?.unix_timestamp;
    let parcel_key = ctx.accounts.parcel.key();

    let asset = &mut ctx.accounts.asset;
    asset.parcel = parcel_key;
    asset.authority = ctx.accounts.registrar.key();
    asset.dimensionality = dimensionality;
    asset.elevation_min_mm = elevation_min_mm;
    asset.elevation_max_mm = elevation_max_mm;
    asset.geometry_version_count = 0;
    asset.latest_geometry = [0u8; 32];
    asset.created_at = now;
    asset.updated_at = now;

    emit!(crate::SpatialAssetCreated {
        asset: asset.key(),
        parcel: parcel_key,
        dimensionality,
        created_at: now,
    });

    Ok(())
}

/// Anchor one geometry version for an asset. Permissionless rent-paid
/// append: the entry is an immutable *claim*; `verified` flips only through
/// [`verify_geometry_version`].
pub fn append_geometry_version(
    ctx: Context<crate::AppendGeometryVersion>,
    geometry_hash: [u8; 32],
    source: u8,
    dimension: u8,
    storage_reference: String,
) -> Result<()> {
    require!(
        is_valid_geometry_source(source),
        TerraError::InvalidGeometrySource
    );
    require!(
        is_valid_dimension(dimension),
        TerraError::InvalidSpatialDimension
    );
    require!(
        is_nonzero_hash(&geometry_hash),
        TerraError::EmptyGeometryHash
    );
    require!(
        is_valid_storage_reference(&storage_reference),
        TerraError::EmptyStorageReference
    );

    let asset = &mut ctx.accounts.asset;
    require!(
        dimension_within_asset(dimension, asset.dimensionality),
        TerraError::InvalidSpatialDimension
    );
    require!(
        !append_would_exceed_cap(asset.geometry_version_count),
        TerraError::GeometryVersionsFull
    );

    let now = Clock::get()?.unix_timestamp;
    let version = asset.geometry_version_count;

    let entry = &mut ctx.accounts.geometry_version;
    entry.asset = asset.key();
    entry.parcel = asset.parcel;
    entry.version = version;
    entry.geometry_hash = geometry_hash;
    entry.source = source;
    entry.dimension = dimension;
    entry.storage_reference = storage_reference;
    entry.submitted_by = ctx.accounts.payer.key();
    entry.submitted_at = now;
    entry.verified = false;
    entry.verified_by = Pubkey::default();
    entry.verified_at = 0;

    asset.geometry_version_count = version + 1;
    asset.latest_geometry = geometry_hash;
    asset.updated_at = now;

    emit!(crate::GeometryVersionAppended {
        geometry_version: entry.key(),
        asset: entry.asset,
        version,
        geometry_hash,
        source,
        dimension,
        submitted_by: entry.submitted_by,
        submitted_at: now,
    });

    Ok(())
}

/// A registered validator verifies an anchored geometry version
/// (claim → fact, mirroring `verify_device`).
pub fn verify_geometry_version(ctx: Context<crate::VerifyGeometryVersion>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;

    let entry = &mut ctx.accounts.geometry_version;
    require!(!entry.verified, TerraError::GeometryAlreadyVerified);

    entry.verified = true;
    entry.verified_by = ctx.accounts.validator.key();
    entry.verified_at = now;

    let asset = &mut ctx.accounts.asset;
    asset.updated_at = now;

    emit!(crate::GeometryVersionVerified {
        geometry_version: entry.key(),
        asset: entry.asset,
        verified_by: entry.verified_by,
        verified_at: now,
    });

    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spatial_dimension_values_are_contiguous() {
        assert_eq!(spatial_dimension::D2, 0);
        assert_eq!(spatial_dimension::D2_5, 1);
        assert_eq!(spatial_dimension::D3, 2);
        assert_eq!(spatial_dimension::MAX, 2);
        assert!(is_valid_dimension(0));
        assert!(is_valid_dimension(2));
        assert!(!is_valid_dimension(3));
    }

    #[test]
    fn geometry_source_values_are_contiguous() {
        assert_eq!(geometry_source::MANUAL, 0);
        assert_eq!(geometry_source::SURVEY, 1);
        assert_eq!(geometry_source::DRONE_PHOTO, 2);
        assert_eq!(geometry_source::SATELLITE, 3);
        assert_eq!(geometry_source::LIDAR, 4);
        assert_eq!(geometry_source::PHOTOGRAMMETRY, 5);
        assert_eq!(geometry_source::AI_SEGMENTATION, 6);
        assert_eq!(geometry_source::API_IMPORT, 7);
        assert_eq!(geometry_source::MAX, 7);
        assert!(is_valid_geometry_source(0));
        assert!(is_valid_geometry_source(7));
        assert!(!is_valid_geometry_source(8));
    }

    #[test]
    fn elevation_range_rejects_inversion() {
        assert!(elevation_range_ok(0, 0));
        assert!(elevation_range_ok(-120, 4_400_000));
        assert!(elevation_range_ok(-50, -10));
        assert!(!elevation_range_ok(100, 50));
        assert!(!elevation_range_ok(1, 0));
    }

    #[test]
    fn version_dimension_must_fit_asset() {
        assert!(dimension_within_asset(
            spatial_dimension::D2,
            spatial_dimension::D3
        ));
        assert!(dimension_within_asset(
            spatial_dimension::D3,
            spatial_dimension::D3
        ));
        assert!(!dimension_within_asset(
            spatial_dimension::D3,
            spatial_dimension::D2
        ));
        assert!(!dimension_within_asset(
            spatial_dimension::D2_5,
            spatial_dimension::D2
        ));
    }

    #[test]
    fn append_cap_is_64() {
        assert_eq!(MAX_GEOMETRY_VERSIONS, 64);
        assert!(!append_would_exceed_cap(0));
        assert!(!append_would_exceed_cap(63));
        assert!(append_would_exceed_cap(64));
        assert!(append_would_exceed_cap(65));
    }
}
