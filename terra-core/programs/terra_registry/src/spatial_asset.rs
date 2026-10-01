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
//!
//! Phase B (RFC-013 §7):
//!
//! * **Evidence linkage** — [`append_geometry_version_from_evidence`] anchors
//!   a version *from* an evidence manifest bound to a verification task
//!   (manifest → observation chain); standalone appends store
//!   `evidence_manifest = Pubkey::default()`.
//! * **Elevation provenance** — each version carries its own elevation
//!   envelope plus an [`elevation_source`] code. The provenance is mandatory
//!   for any layer that implies Z ([`elevation_matches_dimension`]) and the
//!   envelope must sit inside the asset's bounding envelope.
//! * **Validator quorum** — `verified` flips only when `attest_count >=
//!   required` unique registered validators have attested (threshold resolved
//!   from the global `QuorumConfig` at append time, default 2); every
//!   attestation appends to the version's `attestors` list (append-only).

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

/// Provenance of a version's elevation envelope (Phase B, RFC-013 §4).
/// `NONE` ⇔ the version carries no elevation (flat, dimension D2); any
/// other code ⇔ the layer implies Z (dimension ≥ D2_5) and states where
/// the Z values came from. Verification (quorum) attests this provenance.
pub mod elevation_source {
    /// No elevation carried (flat layer, dimension must be D2).
    pub const NONE: u8 = 0;
    /// Survey-grade instrument / certified process.
    pub const SURVEY: u8 = 1;
    /// Digital elevation model tiles (public DEM rasters).
    pub const DEM: u8 = 2;
    /// Airborne / terrestrial LiDAR point cloud.
    pub const LIDAR: u8 = 3;
    /// Photogrammetric reconstruction (SfM / MVS).
    pub const PHOTOGRAMMETRY: u8 = 4;
    /// Hand-entered from plans or records.
    pub const MANUAL: u8 = 5;
    pub const MAX: u8 = 5;
}

/// Maximum geometry versions per spatial asset (bounds rent & state growth;
/// mirrors `MAX_MANIFEST_ARTIFACTS`' bounded-append pattern).
pub const MAX_GEOMETRY_VERSIONS: u32 = 64;

/// Default quorum for geometry verification when no `QuorumConfig` is
/// supplied (same default as claim `required_attestations`).
pub const DEFAULT_GEOMETRY_QUORUM: u8 = 2;

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
    /// Claim → fact: true once `attest_count >= required` validators attested.
    pub verified: bool,
    /// Validator whose attestation completed the quorum (zero until verified).
    pub verified_by: Pubkey,
    pub verified_at: i64,
    // --- Phase B (RFC-013 §7) ---
    /// EvidenceManifest this version was anchored from
    /// (`Pubkey::default()` = standalone permissionless append).
    pub evidence_manifest: Pubkey,
    /// This version's elevation envelope, millimetres (both 0 when
    /// `elevation_source == elevation_source::NONE`).
    pub elevation_min_mm: i32,
    pub elevation_max_mm: i32,
    /// `elevation_source` provenance code for the Z values above.
    pub elevation_source: u8,
    /// Quorum threshold resolved at append (global `QuorumConfig`, else 2;
    /// 1..=MAX_VALIDATORS).
    pub required: u8,
    /// Unique validator attestors so far (zero-filled padding after
    /// `attest_count`; append-only — slots are never reused).
    pub attestors: [Pubkey; crate::MAX_VALIDATORS],
    /// Confirmed attestations so far (0..=required until verified).
    pub attest_count: u8,
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

pub fn is_valid_elevation_source(s: u8) -> bool {
    s <= elevation_source::MAX
}

/// Elevation provenance ⇔ dimension that implies Z (Phase B invariant):
/// `elevation_source::NONE` only on a flat D2 layer; any non-NONE
/// provenance requires the version to be D2_5 or D3.
pub fn elevation_matches_dimension(source: u8, dimension: u8) -> bool {
    (source == elevation_source::NONE) == (dimension < spatial_dimension::D2_5)
}

/// `elevation_source::NONE` must carry an all-zero envelope: a version
/// that declares "no elevation" may not smuggle Z bounds along, and a
/// version with a real envelope must state its provenance.
pub fn elevation_none_is_zero(source: u8, min_mm: i32, max_mm: i32) -> bool {
    source != elevation_source::NONE || (min_mm == 0 && max_mm == 0)
}

/// The version's elevation envelope must sit inside the asset's bounding
/// envelope (equal bounds legal — a flat slice of the asset).
pub fn elevation_within_asset(
    min_mm: i32,
    max_mm: i32,
    asset_min_mm: i32,
    asset_max_mm: i32,
) -> bool {
    min_mm >= asset_min_mm && max_mm <= asset_max_mm
}

/// Pure quorum-threshold resolution: a present `QuorumConfig` value wins,
/// otherwise the protocol default of 2 (same default as claim creation).
/// The result must be within 1..=MAX_VALIDATORS so the fixed attestor list
/// can always reach quorum.
pub fn resolve_required_value(configured: Option<u8>) -> Result<u8> {
    let required = configured.unwrap_or(DEFAULT_GEOMETRY_QUORUM);
    require!(
        required >= 1 && (required as usize) <= crate::MAX_VALIDATORS,
        TerraError::InvalidRequiredAttestations
    );
    Ok(required)
}

/// Resolve the quorum threshold from the optional global `QuorumConfig`
/// (passed in `remaining_accounts`, keyed `(0, [0,0])` — spatial assets
/// have no parcel_type/region, so only the global config applies).
pub fn resolve_geometry_quorum<'info>(remaining_accounts: &[AccountInfo<'info>]) -> Result<u8> {
    let cfg =
        crate::verification::session::try_load_quorum_config(remaining_accounts, 0, [0u8; 2])?;
    resolve_required_value(cfg.map(|c| c.required_attestations))
}

/// Has this validator already attested? Only slots `< attest_count` are
/// meaningful (append-only list, zero-padding ignored).
pub fn is_attestor(attestors: &[Pubkey; crate::MAX_VALIDATORS], validator: &Pubkey) -> bool {
    attestors.contains(validator)
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

/// Shared append core for both anchor variants: validates geometry +
/// elevation provenance, resolves the quorum threshold, writes the
/// immutable version entry and advances the asset cursor.
#[allow(clippy::too_many_arguments)]
fn append_version_common(
    asset: &mut SpatialAsset,
    asset_pk: Pubkey,
    entry: &mut GeometryVersion,
    entry_pk: Pubkey,
    payer: Pubkey,
    evidence_manifest: Pubkey,
    geometry_hash: [u8; 32],
    source: u8,
    dimension: u8,
    storage_reference: String,
    elevation_min_mm: i32,
    elevation_max_mm: i32,
    elevation_source_code: u8,
    remaining_accounts: &[AccountInfo<'_>],
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
    require!(
        dimension_within_asset(dimension, asset.dimensionality),
        TerraError::InvalidSpatialDimension
    );
    require!(
        !append_would_exceed_cap(asset.geometry_version_count),
        TerraError::GeometryVersionsFull
    );

    // Phase B: elevation provenance (RFC-013 §9.3 — who may attest: the
    // validator quorum; here we only anchor the declared provenance).
    require!(
        is_valid_elevation_source(elevation_source_code),
        TerraError::InvalidElevationSource
    );
    require!(
        elevation_range_ok(elevation_min_mm, elevation_max_mm),
        TerraError::InvalidElevationRange
    );
    require!(
        elevation_none_is_zero(elevation_source_code, elevation_min_mm, elevation_max_mm),
        TerraError::InvalidElevationSource
    );
    require!(
        elevation_matches_dimension(elevation_source_code, dimension),
        TerraError::InvalidElevationSource
    );
    require!(
        elevation_within_asset(
            elevation_min_mm,
            elevation_max_mm,
            asset.elevation_min_mm,
            asset.elevation_max_mm,
        ),
        TerraError::InvalidElevationRange
    );

    let required = resolve_geometry_quorum(remaining_accounts)?;

    let now = Clock::get()?.unix_timestamp;
    let version = asset.geometry_version_count;

    entry.asset = asset_pk;
    entry.parcel = asset.parcel;
    entry.version = version;
    entry.geometry_hash = geometry_hash;
    entry.source = source;
    entry.dimension = dimension;
    entry.storage_reference = storage_reference;
    entry.submitted_by = payer;
    entry.submitted_at = now;
    entry.verified = false;
    entry.verified_by = Pubkey::default();
    entry.verified_at = 0;
    entry.evidence_manifest = evidence_manifest;
    entry.elevation_min_mm = elevation_min_mm;
    entry.elevation_max_mm = elevation_max_mm;
    entry.elevation_source = elevation_source_code;
    entry.required = required;
    entry.attestors = [Pubkey::default(); crate::MAX_VALIDATORS];
    entry.attest_count = 0;

    asset.geometry_version_count = version + 1;
    asset.latest_geometry = geometry_hash;
    asset.updated_at = now;

    emit!(crate::GeometryVersionAppended {
        geometry_version: entry_pk,
        asset: asset_pk,
        version,
        geometry_hash,
        source,
        dimension,
        submitted_by: entry.submitted_by,
        submitted_at: now,
        evidence_manifest,
        elevation_min_mm,
        elevation_max_mm,
        elevation_source: elevation_source_code,
        required,
    });

    Ok(())
}

/// Anchor one geometry version for an asset. Permissionless rent-paid
/// append: the entry is an immutable *claim*; quorum verification
/// ([`verify_geometry_version`]) turns it into a fact. Standalone path —
/// `evidence_manifest` stays the default (zero) pubkey; pass the global
/// `QuorumConfig` in `remaining_accounts` to override the default quorum.
#[allow(clippy::too_many_arguments)]
pub fn append_geometry_version(
    ctx: Context<crate::AppendGeometryVersion>,
    geometry_hash: [u8; 32],
    source: u8,
    dimension: u8,
    storage_reference: String,
    elevation_min_mm: i32,
    elevation_max_mm: i32,
    elevation_source: u8,
) -> Result<()> {
    let asset_pk = ctx.accounts.asset.key();
    let entry_pk = ctx.accounts.geometry_version.key();
    append_version_common(
        &mut ctx.accounts.asset,
        asset_pk,
        &mut ctx.accounts.geometry_version,
        entry_pk,
        ctx.accounts.payer.key(),
        Pubkey::default(),
        geometry_hash,
        source,
        dimension,
        storage_reference,
        elevation_min_mm,
        elevation_max_mm,
        elevation_source,
        ctx.remaining_accounts,
    )
}

/// Anchor one geometry version *from* an evidence manifest (Phase B):
/// the version declares which task-bound manifest it grows out of, keeping
/// geometry inside the RFC-012 evidence stack (`manifest.observation`
/// transitively links the capture). Permissionless; the manifest must hold
/// at least one artifact and its task must not be terminal.
#[allow(clippy::too_many_arguments)]
pub fn append_geometry_version_from_evidence(
    ctx: Context<crate::AppendGeometryVersionFromEvidence>,
    geometry_hash: [u8; 32],
    source: u8,
    dimension: u8,
    storage_reference: String,
    elevation_min_mm: i32,
    elevation_max_mm: i32,
    elevation_source: u8,
) -> Result<()> {
    require!(
        ctx.accounts.manifest.task_id == ctx.accounts.task.task_id,
        TerraError::InvalidTaskRequirement
    );
    require!(
        crate::evidence_manifest::can_attach_evidence(ctx.accounts.task.status),
        TerraError::TaskAlreadyFinalized
    );
    require!(
        ctx.accounts.manifest.artifact_count >= 1,
        TerraError::EmptyEvidenceManifest
    );

    let asset_pk = ctx.accounts.asset.key();
    let entry_pk = ctx.accounts.geometry_version.key();
    let manifest_pk = ctx.accounts.manifest.key();
    append_version_common(
        &mut ctx.accounts.asset,
        asset_pk,
        &mut ctx.accounts.geometry_version,
        entry_pk,
        ctx.accounts.payer.key(),
        manifest_pk,
        geometry_hash,
        source,
        dimension,
        storage_reference,
        elevation_min_mm,
        elevation_max_mm,
        elevation_source,
        ctx.remaining_accounts,
    )
}

/// A registered validator attests an anchored geometry version; each unique
/// validator is appended to the version's attestor list. Once
/// `attest_count >= required` (quorum resolved at append), `verified`
/// flips — the claim becomes a fact, and the attesting validator set has
/// jointly attested the geometry *and* its elevation provenance
/// (Phase B quorum; single-validator verify was the Phase A interim).
pub fn verify_geometry_version(ctx: Context<crate::VerifyGeometryVersion>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let validator = ctx.accounts.validator.key();

    let entry = &mut ctx.accounts.geometry_version;
    require!(!entry.verified, TerraError::GeometryAlreadyVerified);
    require!(
        entry.attest_count < crate::MAX_VALIDATORS as u8,
        TerraError::GeometryQuorumFull
    );
    require!(
        !is_attestor(&entry.attestors, &validator),
        TerraError::GeometryAlreadyAttested
    );

    let slot = entry.attest_count as usize;
    entry.attestors[slot] = validator;
    entry.attest_count += 1;
    let count = entry.attest_count;
    let required = entry.required;

    emit!(crate::GeometryVersionAttested {
        geometry_version: entry.key(),
        asset: entry.asset,
        validator,
        attest_count: count,
        required,
    });

    if count >= required {
        entry.verified = true;
        entry.verified_by = validator;
        entry.verified_at = now;
        ctx.accounts.asset.updated_at = now;

        emit!(crate::GeometryVersionVerified {
            geometry_version: entry.key(),
            asset: entry.asset,
            verified_by: validator,
            verified_at: now,
            attest_count: count,
            required,
        });
    }

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

    // --- Phase B (RFC-013 §7) ------------------------------------------------

    #[test]
    fn elevation_source_values_are_contiguous() {
        assert_eq!(elevation_source::NONE, 0);
        assert_eq!(elevation_source::SURVEY, 1);
        assert_eq!(elevation_source::DEM, 2);
        assert_eq!(elevation_source::LIDAR, 3);
        assert_eq!(elevation_source::PHOTOGRAMMETRY, 4);
        assert_eq!(elevation_source::MANUAL, 5);
        assert_eq!(elevation_source::MAX, 5);
        for s in 0..=elevation_source::MAX {
            assert!(is_valid_elevation_source(s));
        }
        assert!(!is_valid_elevation_source(6));
    }

    #[test]
    fn elevation_none_requires_zero_envelope() {
        assert!(elevation_none_is_zero(elevation_source::NONE, 0, 0));
        assert!(!elevation_none_is_zero(elevation_source::NONE, 10, 20));
        assert!(!elevation_none_is_zero(elevation_source::NONE, -5, 0));
        // Non-NONE provenance may carry any (non-inverted) envelope.
        assert!(elevation_none_is_zero(
            elevation_source::SURVEY,
            -50,
            1_200_000
        ));
        assert!(elevation_none_is_zero(elevation_source::DEM, 0, 0));
    }

    #[test]
    fn elevation_provenance_biconditional_with_dimension() {
        // NONE ⇔ flat D2 layer.
        assert!(elevation_matches_dimension(
            elevation_source::NONE,
            spatial_dimension::D2
        ));
        assert!(!elevation_matches_dimension(
            elevation_source::NONE,
            spatial_dimension::D2_5
        ));
        assert!(!elevation_matches_dimension(
            elevation_source::NONE,
            spatial_dimension::D3
        ));
        // Non-NONE ⇔ layer implies Z (D2_5 / D3).
        assert!(!elevation_matches_dimension(
            elevation_source::SURVEY,
            spatial_dimension::D2
        ));
        assert!(!elevation_matches_dimension(
            elevation_source::DEM,
            spatial_dimension::D2
        ));
        assert!(elevation_matches_dimension(
            elevation_source::SURVEY,
            spatial_dimension::D2_5
        ));
        assert!(elevation_matches_dimension(
            elevation_source::LIDAR,
            spatial_dimension::D3
        ));
        assert!(elevation_matches_dimension(
            elevation_source::MANUAL,
            spatial_dimension::D2_5
        ));
    }

    #[test]
    fn elevation_envelope_must_fit_asset() {
        assert!(elevation_within_asset(0, 0, -50, 1_200_000));
        assert!(elevation_within_asset(-50, 1_200_000, -50, 1_200_000));
        assert!(elevation_within_asset(-10, 10, -50, 50));
        assert!(!elevation_within_asset(-60, 0, -50, 50));
        assert!(!elevation_within_asset(0, 51, -50, 50));
        // Inversion itself is elevation_range_ok's job (tested above).
    }

    #[test]
    fn quorum_resolution_bounds() {
        assert_eq!(
            resolve_required_value(None).unwrap(),
            DEFAULT_GEOMETRY_QUORUM
        );
        assert_eq!(resolve_required_value(Some(1)).unwrap(), 1);
        assert_eq!(resolve_required_value(Some(8)).unwrap(), 8);
        assert!(resolve_required_value(Some(0)).is_err());
        assert!(resolve_required_value(Some(9)).is_err());
        assert!(resolve_required_value(Some(255)).is_err());
        assert_eq!(DEFAULT_GEOMETRY_QUORUM, 2);
    }

    #[test]
    fn attestor_list_dedup_and_padding() {
        let mut attestors = [Pubkey::default(); crate::MAX_VALIDATORS];
        let v1 = Pubkey::new_unique();
        let v2 = Pubkey::new_unique();
        assert!(!is_attestor(&attestors, &v1));
        attestors[0] = v1;
        attestors[1] = v2;
        assert!(is_attestor(&attestors, &v1));
        assert!(is_attestor(&attestors, &v2));
        // Zero padding never matches a real (non-zero) validator key, and
        // any other fresh key is absent.
        let v3 = Pubkey::new_unique();
        assert!(v3 != Pubkey::default());
        assert!(!is_attestor(&attestors, &v3));
    }
}
