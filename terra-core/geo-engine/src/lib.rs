//! # Terra Geo SDK (`terra-geo`)
//!
//! The Terra SDK core for spatial data structures, geometry digests,
//! component-package conformance, signed manifests, and quadtree discovery.
//!
//! ## SDK identity
//! - Name: `terra-geo`
//! - Version: `0.1.0`
//! - Protocol schemas implemented:
//!   - RFC-015 (Spatial Component Package)
//!   - terra-geom-v1 (2D geometry digest)
//!   - terra-geom-v2 (3D geometry digest)
//!   - `terra.manifest/v1` (signed content-addressed manifests)
//! - Module capabilities:
//!   - `canonical` / `canonical3d`: geometry canonicalization + digest
//!     (terra-geom-v1 / terra-geom-v2)
//!   - `component_package`: RFC-015 Spatial Component Package + conformance
//!     validator (`validate`, `validate_with_geometry`)
//!   - `manifest`: content-addressed signed manifests
//!     (`ContentAddress`, `ManifestPayload`, `SignedManifest`)
//!   - `spatial_index`: quadtree global-discovery partitions (ADR-020)
//!   - `access` / `graph` / `osm` / `poi` / `reach`: spatial access, road graph,
//!     OSM data, POIs, reachability
//! - Security assumptions: content-addressed artifact storage (SHA-256),
//!   Ed25519 domain-separated signatures, digest integrity verified before trust
//! - Upgrade/migration: schema versions are additive; digest algorithm versions
//!   are declared per-artifact (`digest_algorithm_version`) and never silently
//!   changed
//!
//! This crate is the official reference implementation of the Terra SDK
//! (directive §3 / §12). Modules communicate through documented, stable,
//! versioned interfaces. A change to one module does not force every other
//! module or application to upgrade.

pub mod access;
pub mod canonical;
pub mod canonical3d;
pub mod component_package;
pub mod graph;
pub mod manifest;
pub mod osm;
pub mod poi;
pub mod reach;
pub mod spatial_index;

pub use access::{nearest_road_access, road_access_along_boundary, AccessPoint};
pub use canonical::{canonicalize as canonicalize_geometry, geometry_digest, geometry_digest_hex};
pub use canonical3d::{
    canonicalize_v2 as canonicalize_geometry_v2, geometry_digest_hex_v2, geometry_digest_v2,
    GEOMETRY_DIGEST_VERSION_V2,
};
pub use component_package::{validate, validate_with_geometry, ComponentPackage, ConformanceError};
pub use graph::{build_graph, RoadGraph, RoadSegment};
pub use manifest::{
    ContentAddress, ManifestError, ManifestObject, ManifestPayload, SignedManifest,
    ValidationState, CONTENT_ADDRESS_PREFIX, MANIFEST_SCHEMA, MANIFEST_SIGNATURE_DOMAIN, MAX_LOD,
};
pub use osm::{read_osm_pbf, BuildingFootprint, OsmData, Poi, RoadWay};
pub use poi::{pois_near, pois_within};
pub use reach::{
    access_digest, analyze, encode_metrics, is_sealed_highway, NetworkGraph, ReachabilityReport,
    FLAG_ROAD_ACCESS, ROAD_ACCESS_THRESHOLD_M,
};
pub use sdk_meta::{
    sdk_capabilities, sdk_name, sdk_schema_versions, sdk_security_assumptions,
    sdk_upgrade_migration, sdk_version,
};
pub use spatial_index::{
    covering_cells, GeoBounds, QuadCell, SpatialIndexError, MAX_COVERAGE_CELLS, MAX_LEVEL,
};

// ---------------------------------------------------------------------------
// SDK metadata (directive §3 / §12): declares the SDK identity, capabilities,
// schema versions, dependencies, security assumptions, and upgrade behavior so
// the SDK is an explicit, versioned standard rather than an implicit crate.
// ---------------------------------------------------------------------------
pub mod sdk_meta {
    //! Metadata describing the `terra-geo` SDK standard.

    /// SDK crate name.
    pub fn sdk_name() -> &'static str {
        "terra-geo"
    }
    /// SDK crate version (semver).
    pub fn sdk_version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Capabilities exposed by this SDK module.
    pub fn sdk_capabilities() -> &'static [&'static str] {
        &[
            "geometry-digest: terra-geom-v1 (2D), terra-geom-v2 (3D)",
            "component-package-conformance: RFC-015 Spatial Component Package",
            "signed-manifest: terra.manifest/v1 (ContentAddress, ManifestPayload, SignedManifest)",
            "quadtree-discovery: ADR-020 global-discovery cells (q/level/x/y)",
            "spatial-access: nearest-road access, road graph, OSM data, POIs, reachability",
        ]
    }

    /// Protocol schemas implemented by this SDK, with versions.
    pub fn sdk_schema_versions() -> &'static [(&'static str, &'static str)] {
        &[
            ("RFC-015 Spatial Component Package", "1.0"),
            ("terra-geom-v1 geometry digest", "1"),
            ("terra-geom-v2 geometry digest", "2"),
            ("terra.manifest/v1 signed manifest", "1"),
        ]
    }

    /// Direct crate dependencies.
    pub fn sdk_dependencies() -> &'static [(&'static str, &'static str)] {
        &[
            ("geo", "0.29"),
            ("geo-types", "0.7"),
            ("serde", "1"),
            ("serde_json", "1"),
            ("anyhow", "1"),
            ("ed25519-dalek", "2"),
            ("osmpbf", "0.3"),
            ("sha2", "0.10"),
            ("thiserror", "1"),
        ]
    }

    /// Security assumptions made by this SDK.
    pub fn sdk_security_assumptions() -> &'static [&'static str] {
        &[
            "Artifact storage is content-addressed (SHA-256); integrity is verified by digest, never trusted by path.",
            "Manifests are Ed25519 domain-separated signed; a signature is verified before trust.",
            "Digest integrity (rule 1 of RFC-015) is verified against geometry bytes before a component is accepted.",
        ]
    }

    /// Upgrade / migration behavior.
    pub fn sdk_upgrade_migration() -> &'static str {
        "Schema versions are additive and never silently changed. Digest algorithm \
         versions are declared per-artifact (`digest_algorithm_version`) so old \
         versions remain verifiable. A change to one SDK module does not force any \
         other module or application to upgrade (directive §4)."
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn sdk_metadata_is_well_formed() {
            assert!(!sdk_name().is_empty());
            assert!(!sdk_version().is_empty());
            assert!(!sdk_capabilities().is_empty());
            assert!(!sdk_schema_versions().is_empty());
            assert!(!sdk_security_assumptions().is_empty());
            assert!(!sdk_upgrade_migration().is_empty());
            // Every schema version entry has a non-empty name and version.
            for (name, version) in sdk_schema_versions() {
                assert!(!name.is_empty());
                assert!(!version.is_empty());
            }
            // The component-package conformance capability is present (RFC-015).
            assert!(sdk_capabilities()
                .iter()
                .any(|c| c.contains("component-package-conformance")));
            // terra-geom-v1 and terra-geom-v2 are both declared.
            assert!(sdk_schema_versions()
                .iter()
                .any(|(n, _)| *n == "terra-geom-v1 geometry digest"));
            assert!(sdk_schema_versions()
                .iter()
                .any(|(n, _)| *n == "terra-geom-v2 geometry digest"));
        }
    }
}
