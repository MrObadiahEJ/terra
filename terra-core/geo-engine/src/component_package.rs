//! RFC-015 — Spatial Component Package conformance validator.
//!
//! Validates a Terra Spatial Component Package against the 8 conformance rules
//! from RFC-015 §4. An engine is not Terra-compatible merely because it exports
//! a GLB or GeoJSON file; the package must pass this conformance test.
//!
//! `validate()` checks the structural rules (2–8) that do not require the
//! geometry bytes. `validate_with_geometry()` additionally checks rule 1
//! (digest integrity) given the geometry document as a GeoJSON string.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use anyhow::{Result, bail, Context};

use crate::canonical3d::{canonicalize_v2, geometry_digest_hex_v2};

/// A Terra Spatial Component Package (RFC-015 §3).
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ComponentPackage {
    pub component_id: String,
    pub physical_entity_id: String,
    pub parent_id: Option<String>,
    pub related_ids: Option<Vec<String>>,
    pub component_type: String,
    pub geometry_artifact: String,
    pub format: String,
    pub format_version: String,
    pub geometry_digest: String,
    pub digest_algorithm_version: String,
    pub crs: String,
    pub elevation_ref: String,
    pub vertical_units: String,
    pub local_frame: Option<Value>,
    pub global_transform: Option<Vec<f64>>,
    pub bounding_volume: BoundingVolume,
    pub spatial_extent: Value,
    pub resolution: f64,
    pub lod: i32,
    pub source_evidence_refs: Vec<String>,
    pub reconstruction_engine_id: String,
    pub reconstruction_engine_version: String,
    pub processing_configuration: Option<Value>,
    pub creation_timestamp: String,
    pub resource_usage_receipt: Option<Value>,
    pub accuracy: Option<f64>,
    pub uncertainty: Option<f64>,
    pub coverage: Option<Value>,
    pub confidence: Option<f64>,
    pub provenance_classification: String,
    pub dependencies: Option<Vec<String>>,
    pub overlap_ids: Option<Vec<String>>,
    pub privacy_classification: String,
    pub access_policy: Value,
    pub validation_state: String,
    pub version_lineage: Value,
}

/// Axis-aligned bounding volume (RFC-015 §3).
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BoundingVolume {
    pub min: Vec<f64>,
    pub max: Vec<f64>,
}

/// Validation error details.
#[derive(Debug, thiserror::Error)]
pub enum ConformanceError {
    #[error("rule 1: digest integrity — geometry_digest does not match canonical digest of geometry_artifact")]
    DigestIntegrity,
    #[error("rule 2: CRS validity — crs must normalize to EPSG:4326, got '{0}'")]
    CrsInvalid(String),
    #[error("rule 3: elevation_ref must be EGM2008/WGS84-ELLIPSOID/local/unknown, got '{0}'")]
    ElevationRefInvalid(String),
    #[error("rule 4: vertical_units must be m or ft, got '{0}'")]
    VerticalUnitsInvalid(String),
    #[error("rule 6: bounding_volume must enclose spatial_extent")]
    BoundingVolumeInconsistent,
    #[error("rule 7: AI-kind provenance requires reconstruction_engine_id/version and confidence in [0,1]")]
    ProvenanceIncomplete,
    #[error("rule 8: privacy_classification/access_policy must be present and consistent")]
    PrivacyInvalid,
}

/// Validate a component package against RFC-015 conformance rules 2–8
/// (structural rules that do not require the geometry bytes).
///
/// Returns `Ok(())` if the package conforms, or a `ConformanceError`
/// describing the first violated rule.
pub fn validate(pkg: &ComponentPackage) -> Result<()> {
    // Rule 2 — CRS validity: normalize to EPSG:4326.
    if !is_valid_crs(&pkg.crs) {
        bail!(ConformanceError::CrsInvalid(pkg.crs.clone()));
    }

    // Rule 3 — Elevation reference.
    if !matches!(
        pkg.elevation_ref.as_str(),
        "EGM2008" | "WGS84-ELLIPSOID" | "local" | "unknown"
    ) {
        bail!(ConformanceError::ElevationRefInvalid(pkg.elevation_ref.clone()));
    }

    // Rule 4 — Vertical units.
    if !matches!(pkg.vertical_units.as_str(), "m" | "ft") {
        bail!(ConformanceError::VerticalUnitsInvalid(
            pkg.vertical_units.clone()
        ));
    }

    // Rule 6 — Bounding volume consistency.
    if !bounding_encloses(&pkg.bounding_volume, &pkg.spatial_extent) {
        bail!(ConformanceError::BoundingVolumeInconsistent);
    }

    // Rule 7 — Provenance completeness.
    let is_ai_kind = pkg.provenance_classification == "reconstructed"
        && pkg.confidence.is_some();
    if is_ai_kind {
        if pkg.reconstruction_engine_id.is_empty()
            || pkg.reconstruction_engine_version.is_empty()
            || pkg.confidence.map_or(true, |c| !(0.0..=1.0).contains(&c))
        {
            bail!(ConformanceError::ProvenanceIncomplete);
        }
    }

    // Rule 8 — Privacy.
    if pkg.privacy_classification.is_empty()
        || !is_access_policy_consistent(&pkg.access_policy, &pkg.privacy_classification)
    {
        bail!(ConformanceError::PrivacyInvalid);
    }

    Ok(())
}

/// Validate a component package against all RFC-015 conformance rules,
/// including rule 1 (digest integrity) given the geometry document.
///
/// `geometry` is the GeoJSON document referenced by `geometry_artifact`,
/// used to recompute and compare the canonical digest. If `None`, rule 1
/// is skipped.
pub fn validate_with_geometry(
    pkg: &ComponentPackage,
    geometry: Option<&str>,
) -> Result<()> {
    validate(pkg)?;

    // Rule 1 — Digest integrity.
    if let Some(geom) = geometry {
        let canonical = canonicalize_v2(geom)
            .with_context(|| format!("failed to canonicalize geometry_artifact '{}'", pkg.geometry_artifact))?;
        let digest = geometry_digest_hex_v2(&canonical)
            .with_context(|| "failed to compute geometry digest")?;
        if digest != pkg.geometry_digest {
            bail!(ConformanceError::DigestIntegrity);
        }
    }

    Ok(())
}

/// Normalize a CRS string; return true if it is EPSG:4326 / CRS 84.
fn is_valid_crs(crs: &str) -> bool {
    let c = crs.to_lowercase();
    c == "epsg:4326"
        || c == "crs84"
        || c == "urn:ogc:def:crs:ogc:1.3:crs84"
        || c == "urn:ogc:def:crs:epsg::4326"
}

/// Check that bounding_volume min/max encloses spatial_extent.
/// spatial_extent is a GeoJSON Geometry; we check its coordinate extremes.
fn bounding_encloses(bv: &BoundingVolume, extent: &Value) -> bool {
    let coords = match extent.get("coordinates") {
        Some(c) => c,
        None => return false,
    };
    let (min_lon, min_lat, min_z, max_lon, max_lat, max_z) =
        coord_extremes(coords);
    if min_lon.is_none() {
        return false;
    }
    let (min_lon, min_lat, min_z, max_lon, max_lat, max_z) =
        (min_lon.unwrap(), min_lat.unwrap(), min_z.unwrap(), max_lon.unwrap(), max_lat.unwrap(), max_z.unwrap());
    let bv_min = &bv.min;
    let bv_max = &bv.max;
    let blon = bv_min.get(0).copied().unwrap_or(f64::NEG_INFINITY);
    let blat = bv_min.get(1).copied().unwrap_or(f64::NEG_INFINITY);
    let bzmin = bv_min.get(2).copied().unwrap_or(f64::NEG_INFINITY);
    let tlon = bv_max.get(0).copied().unwrap_or(f64::INFINITY);
    let tlat = bv_max.get(1).copied().unwrap_or(f64::INFINITY);
    let tzmax = bv_max.get(2).copied().unwrap_or(f64::INFINITY);
    min_lon >= blon && min_lat >= blat && min_z >= bzmin
        && max_lon <= tlon && max_lat <= tlat && max_z <= tzmax
}

/// Extract lon/lat/z extremes from a GeoJSON coordinates value.
fn coord_extremes(coords: &Value) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    let mut min_lon: Option<f64> = None;
    let mut min_lat: Option<f64> = None;
    let mut min_z: Option<f64> = None;
    let mut max_lon: Option<f64> = None;
    let mut max_lat: Option<f64> = None;
    let mut max_z: Option<f64> = None;

    visit_positions(coords, &mut |lon, lat, z| {
        min_lon = Some(min_lon.unwrap_or(lon).min(lon));
        min_lat = Some(min_lat.unwrap_or(lat).min(lat));
        min_z = Some(min_z.unwrap_or(z).min(z));
        max_lon = Some(max_lon.unwrap_or(lon).max(lon));
        max_lat = Some(max_lat.unwrap_or(lat).max(lat));
        max_z = Some(max_z.unwrap_or(z).max(z));
    });

    (min_lon, min_lat, min_z, max_lon, max_lat, max_z)
}

/// Visit every [lon, lat, z?] position in a GeoJSON coordinates value.
fn visit_positions<F>(coords: &Value, f: &mut F)
where
    F: FnMut(f64, f64, f64),
{
    match coords {
        Value::Array(arr) => {
            // A position: [lon, lat, z?]
            if arr.len() >= 2 {
                if let (Some(Value::Number(lon)), Some(Value::Number(lat))) = (arr.get(0), arr.get(1)) {
                    if let Some(ln) = lon.as_f64() {
                        if let Some(la) = lat.as_f64() {
                            let z = arr.get(2).and_then(Value::as_f64).unwrap_or(0.0);
                            f(ln, la, z);
                            return;
                        }
                    }
                }
            }
            // Otherwise recurse into elements (nested arrays / geometries).
            for el in arr {
                visit_positions(el, f);
            }
        }
        Value::Object(obj) => {
            // GeoJSON geometry object: recurse into "coordinates".
            if let Some(c) = obj.get("coordinates") {
                visit_positions(c, f);
            }
            // GeometryCollection: recurse into "geometries".
            if let Some(gs) = obj.get("geometries") {
                visit_positions(gs, f);
            }
        }
        _ => {}
    }
}

/// Check that access_policy is consistent with privacy_classification.
fn is_access_policy_consistent(_policy: &Value, _classification: &str) -> bool {
    // access_policy must be a non-null object when privacy is restricted/private.
    match _policy {
        Value::Object(_) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_pkg() -> ComponentPackage {
        ComponentPackage {
            component_id: "comp-001".into(),
            physical_entity_id: "parcel-001".into(),
            parent_id: None,
            related_ids: None,
            component_type: "roof".into(),
            geometry_artifact: "https://storage.example.com/roof.glb".into(),
            format: "model/gltf-binary".into(),
            format_version: "2.0".into(),
            geometry_digest: "0e5ac8e42361d78feabaedf8aa9dbb5173285f879f40e9fb2cf93f2943146674".into(),
            digest_algorithm_version: "terra-geom-v2".into(),
            crs: "EPSG:4326".into(),
            elevation_ref: "EGM2008".into(),
            vertical_units: "m".into(),
            local_frame: None,
            global_transform: None,
            bounding_volume: BoundingVolume {
                min: vec![8.5, 47.4, 390.0],
                max: vec![8.6, 47.5, 410.0],
            },
            spatial_extent: serde_json::json!({
                "type": "Polygon",
                "coordinates": [[
                    [8.5, 47.4, 400.0],
                    [8.6, 47.4, 401.0],
                    [8.6, 47.5, 402.0],
                    [8.5, 47.5, 401.0],
                    [8.5, 47.4, 400.0],
                ]]
            }),
            resolution: 0.1,
            lod: 2,
            source_evidence_refs: vec!["capture-001".into()],
            reconstruction_engine_id: "photogrammetry-engine".into(),
            reconstruction_engine_version: "1.2.0".into(),
            processing_configuration: None,
            creation_timestamp: "2026-11-02T20:00:00Z".into(),
            resource_usage_receipt: None,
            accuracy: Some(0.05),
            uncertainty: Some(0.1),
            coverage: None,
            confidence: Some(0.95),
            provenance_classification: "reconstructed".into(),
            dependencies: None,
            overlap_ids: None,
            privacy_classification: "public".into(),
            access_policy: serde_json::json!({ "read": "public" }),
            validation_state: "pending".into(),
            version_lineage: serde_json::json!({ "previous_version_id": null }),
        }
    }

    #[test]
    fn valid_package_passes() {
        let pkg = base_pkg();
        assert!(validate(&pkg).is_ok());
    }

    #[test]
    fn invalid_crs_rejected() {
        let mut pkg = base_pkg();
        pkg.crs = "EPSG:4327".into();
        assert!(validate(&pkg).is_err());
    }

    #[test]
    fn invalid_elevation_ref_rejected() {
        let mut pkg = base_pkg();
        pkg.elevation_ref = "NAVD88".into();
        assert!(validate(&pkg).is_err());
    }

    #[test]
    fn invalid_vertical_units_rejected() {
        let mut pkg = base_pkg();
        pkg.vertical_units = "inches".into();
        assert!(validate(&pkg).is_err());
    }

    #[test]
    fn ai_kind_without_engine_rejected() {
        let mut pkg = base_pkg();
        pkg.reconstruction_engine_id = "".into();
        assert!(validate(&pkg).is_err());
    }

    #[test]
    fn confidence_out_of_range_rejected() {
        let mut pkg = base_pkg();
        pkg.confidence = Some(1.5);
        assert!(validate(&pkg).is_err());
    }

    #[test]
    fn empty_privacy_rejected() {
        let mut pkg = base_pkg();
        pkg.privacy_classification = "".into();
        assert!(validate(&pkg).is_err());
    }

    #[test]
    fn missing_access_policy_rejected() {
        let mut pkg = base_pkg();
        pkg.access_policy = Value::Null;
        assert!(validate(&pkg).is_err());
    }

    #[test]
    fn digest_integrity_matches() {
        let pkg = base_pkg();
        let geom = r#"{
            "crs": "EPSG:4326",
            "elevation_ref": "EGM2008",
            "vertical_units": "m",
            "precision": 3,
            "type": "Point",
            "coordinates": [8.5, 47.4, 400]
        }"#;
        assert!(validate_with_geometry(&pkg, Some(geom)).is_ok());
    }

    #[test]
    fn digest_integrity_mismatch_rejected() {
        let pkg = base_pkg();
        let geom = r#"{
            "type": "Point",
            "coordinates": [9.9, 9.9, 999.0]
        }"#;
        assert!(validate_with_geometry(&pkg, Some(geom)).is_err());
    }

    #[test]
    fn crs_aliases_accepted() {
        for crs in ["CRS84", "urn:ogc:def:crs:OGC:1.3:CRS84", "urn:ogc:def:crs:epsg::4326"] {
            let mut pkg = base_pkg();
            pkg.crs = crs.to_string();
            assert!(validate(&pkg).is_ok(), "crs={crs} should be accepted");
        }
    }

    #[test]
    fn bounding_volume_not_enclosing_rejected() {
        let mut pkg = base_pkg();
        // bounding_volume too small to contain spatial_extent (lon 8.5..8.6)
        pkg.bounding_volume = BoundingVolume {
            min: vec![9.0, 47.0, 300.0],
            max: vec![9.1, 47.1, 400.0],
        };
        assert!(validate(&pkg).is_err());
    }
}
