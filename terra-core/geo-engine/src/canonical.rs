//! Canonical geometry digest (`terra-geom-v1`) — RFC-013 §9 open question #4.
//!
//! `geometry_hash` is the identifier Terra anchors on-chain for a geometry
//! version. For two implementations to agree — the Rust worker, a PostGIS
//! draft, an external photogrammetry service, a validator re-deriving the
//! digest from the stored document — the digest must depend only on the
//! *geometry itself*, never on JSON key order, whitespace, coordinate
//! formatting, ring start vertex, ring orientation or ordering inside a
//! multi-geometry.
//!
//! Rules of `terra-geom-v1`:
//!
//! - CRS is fixed to EPSG:4326 (`x` = longitude, `y` = latitude). A `crs`
//!   member other than 4326 / CRS 84 is rejected; a missing member is assumed
//!   4326 (RFC 7946 default).
//! - Coordinates are **2D only**, quantized to `1e-7` degrees (~1.1 cm at the
//!   equator), rounded half-away-from-zero. Third ordinates are rejected.
//! - Rings: consecutive duplicate vertices removed; the closing vertex is
//!   implicit (never counted twice); a ring with fewer than 3 distinct vertices
//!   or zero area is dropped as degenerate.
//! - Ring orientation is forced to RFC 7946 (exterior counter-clockwise, holes
//!   clockwise) and each ring is rotated to start at its lexicographically
//!   smallest `(lon, lat)` vertex.
//! - Polygon: shell first, then holes sorted; MultiPolygon / MultiLineString /
//!   MultiPoint children are sorted and de-duplicated.
//! - The digest is `SHA-256("terra-geom-v1" ‖ canonical_geojson)`; the canonical
//!   document is UTF-8 GeoJSON with fixed key order and each ordinate printed
//!   with exactly seven fractional digits.
//!
//! Published test vectors live in `testdata/geometry_digest_v1.json`; run
//! `cargo test -p terra-geo -- --ignored regenerate_vectors --nocapture` to
//! refresh them after a v-prefixed change. The `version` tag is part of the
//! digest, so a future v2 can coexist with v1 without ambiguity.

use anyhow::{anyhow, bail, Result};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;

/// Domain-separation + version tag hashed ahead of the canonical document.
pub const GEOMETRY_DIGEST_VERSION: &str = "terra-geom-v1";

/// Coordinates are stored as integers in units of `1e-7` degrees.
const COORD_SCALE: f64 = 10_000_000.0;
const COORD_SCALE_I: i64 = 10_000_000;

/// A quantized 2D coordinate, `(lon, lat)` in `1e-7` degrees.
type Pt = (i64, i64);

/// A canonical polygon: a CCW shell plus CW holes (sorted, de-duplicated).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Poly {
    shell: Vec<Pt>,
    holes: Vec<Vec<Pt>>,
}

/// Hex-encoded `terra-geom-v1` digest of a GeoJSON geometry.
pub fn geometry_digest_hex(geojson: &str) -> Result<String> {
    let bytes = geometry_digest(geojson)?;
    let mut s = String::with_capacity(64);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    Ok(s)
}

/// `terra-geom-v1` digest bytes (SHA-256 over the version tag + canonical doc).
pub fn geometry_digest(geojson: &str) -> Result<[u8; 32]> {
    let canonical = canonicalize(geojson)?;
    let mut hasher = Sha256::new();
    hasher.update(GEOMETRY_DIGEST_VERSION.as_bytes());
    hasher.update(canonical.as_bytes());
    Ok(hasher.finalize().into())
}

/// Canonical `terra-geom-v1` GeoJSON document for a geometry (or a `Feature`).
pub fn canonicalize(geojson: &str) -> Result<String> {
    let root: Value = serde_json::from_str(geojson).map_err(|e| anyhow!("invalid GeoJSON: {e}"))?;
    check_crs(&root)?;

    let geom = match root.get("type").and_then(Value::as_str) {
        Some("Feature") => {
            let g = root.get("geometry").unwrap_or(&Value::Null);
            if g.is_null() {
                bail!("Feature has a null geometry");
            }
            check_crs(g)?;
            g
        }
        _ => &root,
    };

    let ty = geom
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("geometry has no \"type\""))?;
    let coords = geom
        .get("coordinates")
        .ok_or_else(|| anyhow!("geometry has no \"coordinates\""))?;

    match ty {
        "Point" => Ok(emit_point(quantize(xy(coords)?)?)),
        "MultiPoint" => {
            let mut pts = collect_points(coords)?;
            pts.sort();
            pts.dedup();
            Ok(emit_multi_point("MultiPoint", &pts))
        }
        "LineString" => {
            let line =
                canon_line(coords)?.ok_or_else(|| anyhow!("LineString has no usable segment"))?;
            Ok(emit_line(&line))
        }
        "MultiLineString" => {
            let mut lines = Vec::new();
            for c in as_array(coords, "MultiLineString coordinates")? {
                if let Some(l) = canon_line(c)? {
                    lines.push(l);
                }
            }
            if lines.is_empty() {
                bail!("MultiLineString has no usable segment");
            }
            lines.sort();
            lines.dedup();
            Ok(emit_lines("MultiLineString", &lines))
        }
        "Polygon" => {
            let poly =
                canon_polygon(coords)?.ok_or_else(|| anyhow!("Polygon has no usable ring"))?;
            Ok(emit_polygon(&poly))
        }
        "MultiPolygon" => {
            let mut polys = Vec::new();
            for p in as_array(coords, "MultiPolygon coordinates")? {
                if let Some(poly) = canon_polygon(p)? {
                    polys.push(poly);
                }
            }
            if polys.is_empty() {
                bail!("MultiPolygon has no usable polygon");
            }
            polys.sort();
            polys.dedup();
            Ok(emit_multipolygon(&polys))
        }
        other => bail!(
            "unsupported geometry type \"{other}\" \
             (terra-geom-v1 covers 2D Point/LineString/Polygon and their multi- forms)"
        ),
    }
}

// ---------------------------------------------------------------------------
// Parsing & quantization
// ---------------------------------------------------------------------------

fn as_array<'a>(v: &'a Value, what: &str) -> Result<&'a Vec<Value>> {
    v.as_array()
        .ok_or_else(|| anyhow!("{what} is not an array"))
}

fn xy(v: &Value) -> Result<[f64; 2]> {
    let a = v
        .as_array()
        .ok_or_else(|| anyhow!("coordinate is not an array"))?;
    if a.len() != 2 {
        bail!(
            "terra-geom-v1 requires 2D [lon,lat] positions (found {} ordinates)",
            a.len()
        );
    }
    let lon = a[0]
        .as_f64()
        .ok_or_else(|| anyhow!("longitude is not a number"))?;
    let lat = a[1]
        .as_f64()
        .ok_or_else(|| anyhow!("latitude is not a number"))?;
    Ok([lon, lat])
}

fn quantize([lon, lat]: [f64; 2]) -> Result<Pt> {
    Ok((
        round_ord(lon, 180.0, "longitude")?,
        round_ord(lat, 90.0, "latitude")?,
    ))
}

fn round_ord(v: f64, limit: f64, what: &str) -> Result<i64> {
    if !v.is_finite() {
        bail!("{what} is not finite");
    }
    if v.abs() > limit {
        bail!("{what} {v} out of range ±{limit}");
    }
    let scaled = v * COORD_SCALE;
    let rounded = if scaled >= 0.0 {
        (scaled + 0.5).floor()
    } else {
        (scaled - 0.5).ceil()
    };
    Ok(rounded as i64)
}

fn check_crs(v: &Value) -> Result<()> {
    let Some(crs) = v.get("crs") else {
        return Ok(());
    };
    if crs.is_null() {
        return Ok(());
    }
    let name = crs
        .get("properties")
        .and_then(|p| p.get("name"))
        .and_then(Value::as_str);
    let ok = matches!(
        name,
        Some("urn:ogc:def:crs:OGC:1.3:CRS84")
            | Some("urn:ogc:def:crs:EPSG::4326")
            | Some("EPSG:4326")
            | Some("OGC:CRS84")
    );
    if !ok {
        bail!("unsupported CRS (terra-geom-v1 requires EPSG:4326 / CRS84): {name:?}");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Canonicalization
// ---------------------------------------------------------------------------

fn canon_ring(coords: &Value, want_ccw: bool) -> Result<Option<Vec<Pt>>> {
    let arr = as_array(coords, "ring coordinates")?;
    let mut pts: Vec<Pt> = Vec::with_capacity(arr.len());
    for c in arr {
        pts.push(quantize(xy(c)?)?);
    }
    pts.dedup();
    if pts.len() > 1 && pts.first() == pts.last() {
        pts.pop();
    }
    if pts.len() < 3 {
        return Ok(None);
    }
    let area = signed_area2(&pts);
    if area == 0 {
        return Ok(None);
    }
    if (area > 0) != want_ccw {
        pts.reverse();
    }
    rotate_to_min(&mut pts);
    Ok(Some(pts))
}

fn canon_line(coords: &Value) -> Result<Option<Vec<Pt>>> {
    let arr = as_array(coords, "LineString coordinates")?;
    let mut pts: Vec<Pt> = Vec::with_capacity(arr.len());
    for c in arr {
        pts.push(quantize(xy(c)?)?);
    }
    pts.dedup();
    if pts.len() < 2 {
        return Ok(None);
    }
    if pts.first() > pts.last() {
        pts.reverse();
    }
    Ok(Some(pts))
}

fn collect_points(coords: &Value) -> Result<Vec<Pt>> {
    let arr = as_array(coords, "MultiPoint coordinates")?;
    let mut pts = Vec::with_capacity(arr.len());
    for c in arr {
        pts.push(quantize(xy(c)?)?);
    }
    Ok(pts)
}

fn canon_polygon(coords: &Value) -> Result<Option<Poly>> {
    let rings = as_array(coords, "Polygon coordinates")?;
    if rings.is_empty() {
        return Ok(None);
    }
    let Some(shell) = canon_ring(&rings[0], true)? else {
        return Ok(None);
    };
    let mut holes = Vec::new();
    for r in &rings[1..] {
        if let Some(h) = canon_ring(r, false)? {
            holes.push(h);
        }
    }
    holes.sort();
    holes.dedup();
    Ok(Some(Poly { shell, holes }))
}

/// Twice the signed area (`i128` to stay exact); `> 0` means CCW.
fn signed_area2(pts: &[Pt]) -> i128 {
    let n = pts.len();
    if n < 3 {
        return 0;
    }
    let mut area: i128 = 0;
    for i in 0..n {
        let (x1, y1) = pts[i];
        let (x2, y2) = pts[(i + 1) % n];
        area += x1 as i128 * y2 as i128 - x2 as i128 * y1 as i128;
    }
    area
}

fn rotate_to_min(pts: &mut [Pt]) {
    if let Some((idx, _)) = pts.iter().enumerate().min_by_key(|(_, p)| **p) {
        pts.rotate_left(idx);
    }
}

// ---------------------------------------------------------------------------
// Serialization (fixed key order, 7 fractional digits)
// ---------------------------------------------------------------------------

fn push_ord(out: &mut String, v: i64) {
    let a = v.unsigned_abs();
    if v < 0 {
        out.push('-');
    }
    let _ = write!(
        out,
        "{}.{:07}",
        a / COORD_SCALE_I as u64,
        a % COORD_SCALE_I as u64
    );
}

fn push_pt(out: &mut String, (lon, lat): Pt) {
    out.push('[');
    push_ord(out, lon);
    out.push(',');
    push_ord(out, lat);
    out.push(']');
}

/// Serialize an *open* ring, re-appending the implicit closing vertex.
fn push_ring(out: &mut String, ring: &[Pt]) {
    out.push('[');
    for (i, p) in ring.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        push_pt(out, *p);
    }
    if let Some(first) = ring.first() {
        out.push(',');
        push_pt(out, *first);
    }
    out.push(']');
}

fn emit_point(pt: Pt) -> String {
    let mut o = String::from("{\"type\":\"Point\",\"coordinates\":");
    push_pt(&mut o, pt);
    o.push('}');
    o
}

fn emit_multi_point(ty: &str, pts: &[Pt]) -> String {
    let mut o = format!("{{\"type\":\"{ty}\",\"coordinates\":[");
    for (i, p) in pts.iter().enumerate() {
        if i > 0 {
            o.push(',');
        }
        push_pt(&mut o, *p);
    }
    o.push_str("]}");
    o
}

/// LineString: `coordinates` is a single array of positions.
fn emit_line(line: &[Pt]) -> String {
    let mut o = String::from("{\"type\":\"LineString\",\"coordinates\":[");
    for (j, p) in line.iter().enumerate() {
        if j > 0 {
            o.push(',');
        }
        push_pt(&mut o, *p);
    }
    o.push_str("]}");
    o
}

/// MultiLineString: `coordinates` is an array of lines, each an array of positions.
fn emit_lines(ty: &str, lines: &[Vec<Pt>]) -> String {
    let mut o = format!("{{\"type\":\"{ty}\",\"coordinates\":[");
    for (i, l) in lines.iter().enumerate() {
        if i > 0 {
            o.push(',');
        }
        o.push('[');
        for (j, p) in l.iter().enumerate() {
            if j > 0 {
                o.push(',');
            }
            push_pt(&mut o, *p);
        }
        o.push(']');
    }
    o.push_str("]}");
    o
}

/// Polygon: `coordinates` is the array of rings (shell first, then holes).
fn emit_polygon(poly: &Poly) -> String {
    let mut o = String::from("{\"type\":\"Polygon\",\"coordinates\":[");
    push_ring(&mut o, &poly.shell);
    for h in &poly.holes {
        o.push(',');
        push_ring(&mut o, h);
    }
    o.push_str("]}");
    o
}

/// MultiPolygon: `coordinates` is the array of polygons, each an array of rings.
fn emit_multipolygon(polys: &[Poly]) -> String {
    let mut o = String::from("{\"type\":\"MultiPolygon\",\"coordinates\":[");
    for (i, poly) in polys.iter().enumerate() {
        if i > 0 {
            o.push(',');
        }
        o.push('[');
        push_ring(&mut o, &poly.shell);
        for h in &poly.holes {
            o.push(',');
            push_ring(&mut o, h);
        }
        o.push(']');
    }
    o.push_str("]}");
    o
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Vectors {
        version: String,
        vectors: Vec<Vector>,
    }

    #[derive(Deserialize)]
    struct Vector {
        name: String,
        geojson: Value,
        digest: String,
    }

    fn load() -> Vectors {
        serde_json::from_str(include_str!("../testdata/geometry_digest_v1.json")).unwrap()
    }

    #[test]
    fn published_vectors_match() {
        let vs = load();
        assert_eq!(vs.version, GEOMETRY_DIGEST_VERSION);
        assert!(vs.vectors.len() >= 6);
        for v in &vs.vectors {
            let text = serde_json::to_string(&v.geojson).unwrap();
            assert_eq!(
                geometry_digest_hex(&text).unwrap(),
                v.digest,
                "vector {:?} digest mismatch",
                v.name
            );
        }
    }

    /// Equivalent encodings listed in a vector's `same_as` share its digest.
    #[test]
    fn equivalent_encodings_agree() {
        let vs = load();
        for v in &vs.vectors {
            let text = serde_json::to_string(&v.geojson).unwrap();
            let got = geometry_digest_hex(&text).unwrap();
            for other in &vs.vectors {
                if other.name == v.name {
                    continue;
                }
                // compare against any vector carrying the same expected digest
                if other.digest == v.digest {
                    let otext = serde_json::to_string(&other.geojson).unwrap();
                    assert_eq!(
                        geometry_digest_hex(&otext).unwrap(),
                        got,
                        "{} and {} should share a digest",
                        v.name,
                        other.name
                    );
                }
            }
        }
    }

    #[test]
    fn canonical_output_is_idempotent() {
        let vs = load();
        for v in &vs.vectors {
            let text = serde_json::to_string(&v.geojson).unwrap();
            let once = canonicalize(&text).unwrap();
            let twice = canonicalize(&once)
                .unwrap_or_else(|e| panic!("{} canonical not re-parseable: {once} ({e})", v.name));
            assert_eq!(twice, once, "{} not idempotent", v.name);
        }
    }

    #[test]
    fn ring_start_orientation_precision_and_key_order_are_irrelevant() {
        let base = r#"{"type":"Polygon","coordinates":[[[8.5,47.4],[8.6,47.4],[8.6,47.5],[8.5,47.5],[8.5,47.4]]]}"#;
        // reversed (CW), rotated to a different start, over-precise, key order swapped
        let variant = r#"{
            "coordinates":[[[8.60000000000,47.400000000],[8.5000000000,47.4000000001],
                            [8.5000000000,47.500000000],[8.6000000000,47.500000000],
                            [8.6000000000,47.400000000]]],
            "type":"Polygon"
        }"#;
        assert_eq!(
            geometry_digest_hex(base).unwrap(),
            geometry_digest_hex(variant).unwrap()
        );
    }

    #[test]
    fn geometry_change_changes_digest() {
        let a =
            r#"{"type":"Polygon","coordinates":[[[8.5,47.4],[8.6,47.4],[8.6,47.5],[8.5,47.4]]]}"#;
        let b =
            r#"{"type":"Polygon","coordinates":[[[8.5,47.4],[8.61,47.4],[8.6,47.5],[8.5,47.4]]]}"#;
        assert_ne!(
            geometry_digest_hex(a).unwrap(),
            geometry_digest_hex(b).unwrap()
        );
    }

    #[test]
    fn rejects_3d_and_bad_crs() {
        assert!(geometry_digest_hex(r#"{"type":"Point","coordinates":[8.5,47.4,500]}"#).is_err());
        assert!(geometry_digest_hex(
            r#"{"type":"Point","crs":{"properties":{"name":"EPSG:3857"}},"coordinates":[8.5,47.4]}"#
        )
        .is_err());
        assert!(geometry_digest_hex(
            r#"{"type":"Point","crs":{"properties":{"name":"EPSG:4326"}},"coordinates":[8.5,47.4]}"#
        )
        .is_ok());
    }

    /// Regeneration helper: prints the digest for every vector so the file can
    /// be refreshed after a versioned change.
    #[test]
    #[ignore]
    fn regenerate_vectors() {
        let vs = load();
        for v in &vs.vectors {
            let text = serde_json::to_string(&v.geojson).unwrap();
            println!("{} {}", v.name, geometry_digest_hex(&text).unwrap());
        }
    }
}
