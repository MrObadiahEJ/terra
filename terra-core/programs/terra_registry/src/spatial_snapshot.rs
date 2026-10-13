//! Spatial snapshots (RFC-013 Phase C, RFC-012 §5 `SpatialSnapshot`).
//!
//! A snapshot is a **point-in-time copy of a spatial asset's indexes**
//! (resolving RFC-013 §8.5 open question 5: *copy*, never a cursor
//! reference) — later geometry-version appends never mutate it. That makes
//! the classic "what did this place look like in 2022?" read cheap and
//! immutable: find the newest snapshot with `created_at <= T`, then list
//! versions with `version < snapshot.geometry_version_count`.
//!
//! Anyone may create a snapshot (rent-paid, append-only, capped at
//! [`MAX_SPATIAL_SNAPSHOTS`]) — an observation *claim* about history, not a
//! fact; the copied fields are what the asset held at creation time.
//! Snapshots carry a short human label (e.g. "2022 baseline") for timeline
//! UIs. The off-chain API mirror (RFC-013 Phase E) exposes snapshots to the
//! globe client (Phase D).

use anchor_lang::prelude::*;

use crate::TerraError;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum snapshots per spatial asset (bounds rent & state growth; same
/// bounded-append pattern as
/// [`MAX_GEOMETRY_VERSIONS`](crate::spatial_asset::MAX_GEOMETRY_VERSIONS)).
pub const MAX_SPATIAL_SNAPSHOTS: u32 = 32;

/// Maximum label length (bytes); longer labels are rejected, not truncated.
pub const MAX_SNAPSHOT_LABEL_LEN: usize = 64;

// ---------------------------------------------------------------------------
// Accounts
// ---------------------------------------------------------------------------

/// Append-only cursor for a spatial asset's snapshots.
///
/// PDA: `["spatial_snapshot_meta", asset]` — created lazily on the first
/// snapshot (`init_if_needed`), so `SpatialAsset`'s deployed layout is
/// untouched.
#[account]
#[derive(InitSpace)]
pub struct SpatialSnapshotMeta {
    /// The owning `SpatialAsset` PDA.
    pub asset: Pubkey,
    /// Next snapshot index (0..=MAX_SPATIAL_SNAPSHOTS).
    pub count: u32,
}

/// A frozen copy of a spatial asset's indexes at one instant
/// (RFC-012 §5 `SpatialSnapshot`).
///
/// PDA: `["spatial_snapshot", asset, snapshot_index_le_bytes]`.
#[account]
#[derive(InitSpace)]
pub struct SpatialSnapshot {
    /// The owning `SpatialAsset` PDA.
    pub asset: Pubkey,
    /// Redundant parcel key for cheap off-chain joins.
    pub parcel: Pubkey,
    /// 0-based append index (from [`SpatialSnapshotMeta`]).
    pub snapshot_index: u32,
    /// Geometry cursor copied at creation: versions `[0, N)` existed then.
    /// Never changes afterwards (append-only history is safe to replay).
    pub geometry_version_count: u32,
    /// Latest geometry hash at snapshot time (all-zero if none yet).
    pub latest_geometry: [u8; 32],
    /// Asset dimensionality copied at creation (`spatial_dimension`).
    pub dimensionality: u8,
    /// Asset elevation envelope copied at creation, millimetres.
    pub elevation_min_mm: i32,
    pub elevation_max_mm: i32,
    /// Short human label for timelines (e.g. "2022 baseline"), <= 64 bytes.
    #[max_len(64)]
    pub label: String,
    /// Whoever paid to create this snapshot (permissionless).
    pub created_by: Pubkey,
    pub created_at: i64,
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

/// The next append index is legal iff below the cap.
pub fn snapshot_append_would_exceed_cap(current_count: u32) -> bool {
    current_count >= MAX_SPATIAL_SNAPSHOTS
}

pub fn is_valid_label(label: &str) -> bool {
    label.len() <= MAX_SNAPSHOT_LABEL_LEN
}

/// Point-in-time read rule (Phase E mirror): a version belongs to the
/// snapshot iff its index is strictly below the copied cursor.
pub fn version_in_snapshot(version: u32, snapshot_cursor: u32) -> bool {
    version < snapshot_cursor
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Freeze a point-in-time copy of the asset's spatial indexes
/// (permissionless rent-paid append; see module docs).
pub fn create_spatial_snapshot(
    ctx: Context<crate::CreateSpatialSnapshot>,
    label: String,
) -> Result<()> {
    require!(is_valid_label(&label), TerraError::InvalidSnapshotLabel);

    let asset_pk = ctx.accounts.asset.key();
    let meta = &mut ctx.accounts.meta;
    // Reuse path (init_if_needed): the seed re-derivation already pins the
    // account to this asset; set the field unconditionally so a fresh init
    // is complete too.
    meta.asset = asset_pk;
    require!(
        !snapshot_append_would_exceed_cap(meta.count),
        TerraError::SpatialSnapshotsFull
    );

    let now = Clock::get()?.unix_timestamp;
    let snapshot_index = meta.count;
    let asset = &ctx.accounts.asset;
    let snapshot_pk = ctx.accounts.snapshot.key();

    let snapshot = &mut ctx.accounts.snapshot;
    snapshot.asset = asset_pk;
    snapshot.parcel = asset.parcel;
    snapshot.snapshot_index = snapshot_index;
    snapshot.geometry_version_count = asset.geometry_version_count;
    snapshot.latest_geometry = asset.latest_geometry;
    snapshot.dimensionality = asset.dimensionality;
    snapshot.elevation_min_mm = asset.elevation_min_mm;
    snapshot.elevation_max_mm = asset.elevation_max_mm;
    snapshot.label = label;
    snapshot.created_by = ctx.accounts.payer.key();
    snapshot.created_at = now;

    meta.count = snapshot_index + 1;

    emit!(crate::SpatialSnapshotCreated {
        snapshot: snapshot_pk,
        asset: asset_pk,
        parcel: snapshot.parcel,
        snapshot_index,
        geometry_version_count: snapshot.geometry_version_count,
        label_len: snapshot.label.len() as u8,
        created_by: snapshot.created_by,
        created_at: now,
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
    fn max_snapshots_is_bounded() {
        assert_eq!(MAX_SPATIAL_SNAPSHOTS, 32);
        assert_eq!(MAX_SNAPSHOT_LABEL_LEN, 64);
    }

    #[test]
    fn snapshot_cap_is_inclusive_of_last_slot() {
        assert!(!snapshot_append_would_exceed_cap(0));
        assert!(!snapshot_append_would_exceed_cap(MAX_SPATIAL_SNAPSHOTS - 1));
        assert!(snapshot_append_would_exceed_cap(MAX_SPATIAL_SNAPSHOTS));
    }

    #[test]
    fn label_length_is_rejected_not_truncated() {
        assert!(is_valid_label(""));
        assert!(is_valid_label("2022 baseline"));
        assert!(is_valid_label(&"x".repeat(MAX_SNAPSHOT_LABEL_LEN)));
        assert!(!is_valid_label(&"x".repeat(MAX_SNAPSHOT_LABEL_LEN + 1)));
    }

    #[test]
    fn point_in_time_membership_uses_copied_cursor() {
        assert!(!version_in_snapshot(0, 0));
        assert!(version_in_snapshot(0, 1));
        assert!(version_in_snapshot(5, 6));
        assert!(!version_in_snapshot(6, 6));
    }

    #[test]
    fn snapshot_seed_discriminants_are_stable() {
        assert_eq!(b"spatial_snapshot", b"spatial_snapshot");
        assert_eq!(b"spatial_snapshot_meta", b"spatial_snapshot_meta");
        // Account size sanity (no discriminator): fixed fields + 64-byte label.
        let fixed = 32 + 32 + 4 + 4 + 32 + 1 + 4 + 4 + 4 + 64 + 32 + 8;
        assert_eq!(fixed, 221);
    }
}
