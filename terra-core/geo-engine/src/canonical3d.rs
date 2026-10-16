//! Canonical 3D geometry digest (`terra-geom-v2`) — RFC-013 full-3D roadmap,
//! step 1 ("3D data standard").
//!
//! `terra-geom-v1` (see [`crate::canonical`]) is frozen: it hashes 2D-only
//! geometries in EPSG:4326 and every parcel hash in existence depends on its
//! exact rules. This module adds a **compatible v2** for three-dimensional
//! data without touching v1:
//!
//! - The digest domain tag is `"terra-geom-v2"`, so a v1 and a v2 digest of
//!   the same coordinates can never collide, and consumers can tell which
//!   canonicalization produced a hash by its version string alone.
//! - A v2 document *without* third ordinates or metadata canonicalizes to the
//!   byte-for-byte v1 document; only the domain tag differs. A 2D parcel
//!   record therefore has a well-defined v2 identity, and upgrading a record
//!   to 3D changes exactly one thing you can point at: the version tag plus
//!   the added ordinates/metadata.
//!
//! Rules of `terra-geom-v2`:
//!
//! - **Positional members** (all required): `crs` (horizontal), `elevation_ref`,
//!   `vertical_units`, `precision`. They are emitted first, in this fixed key
//!   order, before `type` and `coordinates`:
//!   - `crs`: `"EPSG:4326"` (accepts the RFC 7946 / OGC URN aliases). Other
//!     horizontal CRSes are rejected — longitude/latitude remain `x`/`y`.
//!   - `elevation_ref`: the vertical datum the third ordinate is measured
//!     against: `"EGM2008"` (geoid MSL), `"WGS84-ELLIPSOID"` (GNSS ellipsoid
//!     height), `"local"` (site-relative, e.g. ground floor 0) or `"unknown"`.
//!     Mixing ellipsoid and MSL heights silently is the classic source of
//!     metre-scale Z errors, so the reference is mandatory, not defaulted.
//!   - `vertical_units`: `"m"` (metres) or `"ft"`. Horizontal units are fixed
//!     degrees by the CRS.
//!   - `precision`: positive integer, decimal places of the *source* data.
//!     Ordinates are always quantized to v1's 1e-7-degree grid for X/Y and to
//!     a 1e-5-unit grid for Z (sub-centimetre resolution in metres, while
//!     bounding the fixed-width printed form: seven fractional digits for X/Y,
//!     five for Z). The declared precision itself is part of the digest, so
//!     re-stating the same geometry with a different claimed precision is a
//!     different version.
//! - Positions may carry 2 or 3 ordinates. Missing altitudes are legal
//!   (2.5D pipelines fill them later); a position with more than 3 is rejected.
//! - Quantization is half-away-from-zero, like v1. Z must be finite and
//!   |z| ≤ 100 000 in `vertical_units` (rejects garbage, covers Everest and
//!   the Dead Sea in m, all aircraft ceilings in ft).
//! - Rings, lines, polygons and multi-geometries follow v1 exactly: dedup,
//!   implicit closing vertex, degenerate drops, RFC 7946 orientation, rotation
//!   to the lexicographically smallest `(lon, lat)` vertex, sorted holes and
//!   children. **The third ordinate never affects ordering or orientation** —
//!   two rings that differ only in heights keep the same planar structure but
//!   produce different canonical text (and thus digests) because the printed
//!   positions include Z.
//! - Supported geometry types: Point / LineString / Polygon and their multi-
//!   forms, plus `GeometryCollection` (a versioned building = footprint shell
//!   + roof surfaces + terrain strips in one document). Children are sorted
//!     by their serialized canonical JSON, recursively.
//! - The digest is `SHA-256("terra-geom-v2" ‖ canonical_document)`; the
//!   canonical document is UTF-8 JSON with fixed key order, seven fractional
//!   digits for X/Y and five for Z (trailing zeros kept — the format is fixed
//!   width, not shortest-round-trip).
//!
//! Published test vectors live in `testdata/geometry_digest_v2.json`; run
//! `cargo test -p terra-geo -- --ignored regenerate_vectors_v2 --nocapture`
//! to refresh them after a v-prefixed change.

use anyhow::{anyhow, bail, Result};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;

/// Domain-separation + version tag hashed ahead of the canonical document.
pub const GEOMETRY_DIGEST_VERSION_V2: &str = "terra-geom-v2";

/// Coordinate grids: X/Y in 1e-7 degrees (identical to v1), Z in 1e-5 units.
const ORD_SCALE: f64 = 10_000_000.0;
const ORD_SCALE_I: i64 = 10_000_000;
const Z_SCALE: f64 = 100_000.0;
const Z_SCALE_I: i64 = 100_000;

/// Half-range limits for the printed ordinates.
const LIMIT_XY: f64 = 180.0; // enforced per-axis (180 lon / 90 lat) below
const LIMIT_Z: f64 = 100_000.0;

/// A quantized 3D position: `(lon, lat)` in 1e-7 degrees plus an optional
/// height in `vertical_units` quantized to 1e-5.
type Pt = (i64, i64, Option<i64>);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Units {
    Metres,
    Feet,
}

/// Parsed, validated positional metadata of a v2 document.
struct Meta {
    crs: &'static str,     // always normalized to "EPSG:4326"
    elevation_ref: String, // EGM2008 | WGS84-ELLIPSOID | local | unknown
    vertical_units: Units,
    precision: u32,
}

impl Meta {
    fn z_limit(&self) -> f64 {
        LIMIT_Z
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Hex-encoded `terra-geom-v2` digest of a GeoJSON(-3D) geometry or Feature.
pub fn geometry_digest_hex_v2(geojson: &str) -> Result<String> {
    let bytes = geometry_digest_v2(geojson)?;
    let mut s = String::with_capacity(64);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    Ok(s)
}

/// `terra-geom-v2` digest bytes (SHA-256 over the version tag + canonical doc).
pub fn geometry_digest_v2(geojson: &str) -> Result<[u8; 32]> {
    let canonical = canonicalize_v2(geojson)?;
    let mut hasher = Sha256::new();
    hasher.update(GEOMETRY_DIGEST_VERSION_V2.as_bytes());
    hasher.update(canonical.as_bytes());
    Ok(hasher.finalize().into())
}

/// Canonical `terra-geom-v2` document for a geometry (or a `Feature`).
pub fn canonicalize_v2(geojson: &str) -> Result<String> {
    let root: Value = serde_json::from_str(geojson).map_err(|e| anyhow!("invalid GeoJSON: {e}"))?;

    let geom = match root.get("type").and_then(Value::as_str) {
        Some("Feature") => {
            let g = root.get("geometry").unwrap_or(&Value::Null);
            if g.is_null() {
                bail!("Feature has a null geometry");
            }
            g
        }
        _ => &root,
    };

    let meta = parse_meta(geom)?;

    let ty = geom
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("geometry has no \"type\""))?;

    if ty == "GeometryCollection" {
        return canonicalize_collection(geom, &meta);
    }

    let coords = geom
        .get("coordinates")
        .ok_or_else(|| anyhow!("geometry has no \"coordinates\""))?;

    let body = match ty {
        "Point" => emit_point(&meta, pos(coords, &meta)?),
        "MultiPoint" => {
            let mut pts = collect_positions(coords, &meta)?;
            sort_dedup_pts(&mut pts);
            emit_multi_point(&meta, "MultiPoint", &pts)
        }
        "LineString" => {
            let line = canon_line(coords, &meta)?
                .ok_or_else(|| anyhow!("LineString has no usable segment"))?;
            emit_line(&meta, &line)
        }
        "MultiLineString" => {
            let mut lines = Vec::new();
            for c in as_array(coords, "MultiLineString coordinates")? {
                if let Some(l) = canon_line(c, &meta)? {
                    lines.push(l);
                }
            }
            if lines.is_empty() {
                bail!("MultiLineString has no usable segment");
            }
            lines.sort();
            lines.dedup();
            emit_lines(&meta, "MultiLineString", &lines)
        }
        "Polygon" => {
            let poly = canon_polygon(coords, &meta)?
                .ok_or_else(|| anyhow!("Polygon has no usable ring"))?;
            emit_polygon(&meta, &poly)
        }
        "MultiPolygon" => {
            let mut polys = Vec::new();
            for p in as_array(coords, "MultiPolygon coordinates")? {
                if let Some(poly) = canon_polygon(p, &meta)? {
                    polys.push(poly);
                }
            }
            if polys.is_empty() {
                bail!("MultiPolygon has no usable polygon");
            }
            polys.sort();
            polys.dedup();
            emit_multipolygon(&meta, &polys)
        }
        other => bail!(
            "unsupported geometry type \"{other}\" \
             (terra-geom-v2 covers Point/LineString/Polygon, their multi- forms, \
             and GeometryCollection)"
        ),
    };
    Ok(body)
}

fn canonicalize_collection(geom: &Value, meta: &Meta) -> Result<String> {
    let geoms = geom
        .get("geometries")
        .ok_or_else(|| anyhow!("GeometryCollection has no \"geometries\""))?;
    let arr = as_array(geoms, "GeometryCollection geometries")?;
    if arr.is_empty() {
        bail!("GeometryCollection is empty");
    }
    let mut docs: Vec<String> = Vec::with_capacity(arr.len());
    for g in arr {
        docs.push(canonicalize_value_v2(g, meta)?);
    }
    docs.sort();
    docs.dedup();
    let mut o = String::from("{");
    push_crs(&mut o, meta.crs);
    push_meta_tail(&mut o, meta);
    o.push_str(",\"type\":\"GeometryCollection\",\"geometries\":[");
    for (i, d) in docs.iter().enumerate() {
        if i > 0 {
            o.push(',');
        }
        o.push_str(d);
    }
    o.push_str("]}");
    Ok(o)
}

/// Canonicalize any geometry value under an already-parsed `meta` (used for
/// GeometryCollection children, which do not repeat the positional members).
fn canonicalize_value_v2(geom: &Value, meta: &Meta) -> Result<String> {
    check_no_crs_override(geom)?;
    let ty = geom
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("collection child has no \"type\""))?;
    if ty == "GeometryCollection" {
        bail!("nested GeometryCollection is not supported");
    }
    let coords = geom
        .get("coordinates")
        .ok_or_else(|| anyhow!("collection child has no \"coordinates\""))?;
    Ok(match ty {
        "Point" => emit_point(meta, pos(coords, meta)?),
        "MultiPoint" => {
            let mut pts = collect_positions(coords, meta)?;
            sort_dedup_pts(&mut pts);
            emit_multi_point(meta, "MultiPoint", &pts)
        }
        "LineString" => {
            let line = canon_line(coords, meta)?
                .ok_or_else(|| anyhow!("LineString has no usable segment"))?;
            emit_line(meta, &line)
        }
        "MultiLineString" => {
            let mut lines = Vec::new();
            for c in as_array(coords, "MultiLineString coordinates")? {
                if let Some(l) = canon_line(c, meta)? {
                    lines.push(l);
                }
            }
            if lines.is_empty() {
                bail!("MultiLineString has no usable segment");
            }
            lines.sort();
            lines.dedup();
            emit_lines(meta, "MultiLineString", &lines)
        }
        "Polygon" => {
            let poly = canon_polygon(coords, meta)?
                .ok_or_else(|| anyhow!("Polygon has no usable ring"))?;
            emit_polygon(meta, &poly)
        }
        "MultiPolygon" => {
            let mut polys = Vec::new();
            for p in as_array(coords, "MultiPolygon coordinates")? {
                if let Some(poly) = canon_polygon(p, meta)? {
                    polys.push(poly);
                }
            }
            if polys.is_empty() {
                bail!("MultiPolygon has no usable polygon");
            }
            polys.sort();
            polys.dedup();
            emit_multipolygon(meta, &polys)
        }
        other => bail!("unsupported collection child type \"{other}\""),
    })
}

/// A GeometryCollection child may not re-declare or contradict the parent CRS.
fn check_no_crs_override(geom: &Value) -> Result<()> {
    if let Some(crs) = geom.get("crs") {
        if !crs.is_null() {
            let name = crs_name_of(crs);
            if !is_supported_crs(name) {
                bail!("collection child declares an unsupported crs: {name:?}");
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Metadata parsing
// ---------------------------------------------------------------------------

fn parse_meta(geom: &Value) -> Result<Meta> {
    let crs_val = geom
        .get("crs")
        .ok_or_else(|| anyhow!("terra-geom-v2 requires a positional \"crs\" member"))?;
    let name = if let Some(s) = crs_val.as_str() {
        Some(s.to_string())
    } else {
        crs_val
            .get("properties")
            .and_then(|p| p.get("name"))
            .and_then(Value::as_str)
            .map(str::to_string)
    };
    let name_s = name
        .as_deref()
        .ok_or_else(|| anyhow!("crs member carries no name"))?;
    if !is_supported_crs(Some(name_s)) {
        bail!("unsupported horizontal CRS \"{name_s}\" (terra-geom-v2 requires EPSG:4326 / CRS84)");
    }

    let elevation_ref = geom
        .get("elevation_ref")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            anyhow!("terra-geom-v2 requires an \"elevation_ref\" member (EGM2008 | WGS84-ELLIPSOID | local | unknown)")
        })?;
    if !matches!(
        elevation_ref,
        "EGM2008" | "WGS84-ELLIPSOID" | "local" | "unknown"
    ) {
        bail!("unknown elevation_ref \"{elevation_ref}\"");
    }

    let vertical_units = match geom.get("vertical_units").and_then(Value::as_str) {
        Some("m") => Units::Metres,
        Some("ft") => Units::Feet,
        Some(other) => bail!("unknown vertical_units \"{other}\" (expected \"m\" or \"ft\")"),
        None => bail!("terra-geom-v2 requires a \"vertical_units\" member (\"m\" or \"ft\")"),
    };

    let precision = geom
        .get("precision")
        .and_then(Value::as_u64)
        .ok_or_else(|| anyhow!("terra-geom-v2 requires a positive integer \"precision\" member"))?;
    if precision == 0 || precision > 15 {
        bail!("precision {precision} out of range 1..=15");
    }

    Ok(Meta {
        crs: "EPSG:4326",
        elevation_ref: elevation_ref.to_string(),
        vertical_units,
        precision: precision as u32,
    })
}

fn is_supported_crs(name: Option<&str>) -> bool {
    matches!(
        name,
        Some("EPSG:4326")
            | Some("urn:ogc:def:crs:OGC:1.3:CRS84")
            | Some("urn:ogc:def:crs:EPSG::4326")
            | Some("OGC:CRS84")
    )
}

fn crs_name_of(crs: &Value) -> Option<&str> {
    crs.as_str().or_else(|| {
        crs.get("properties")
            .and_then(|p| p.get("name"))
            .and_then(Value::as_str)
    })
}

// ---------------------------------------------------------------------------
// Position parsing & quantization
// ---------------------------------------------------------------------------

fn as_array<'a>(v: &'a Value, what: &str) -> Result<&'a Vec<Value>> {
    v.as_array()
        .ok_or_else(|| anyhow!("{what} is not an array"))
}

/// Parse one position into a quantized [`Pt`]. Accepts 2 or 3 ordinates.
fn pos(v: &Value, meta: &Meta) -> Result<Pt> {
    let a = v
        .as_array()
        .ok_or_else(|| anyhow!("coordinate is not an array"))?;
    if a.len() < 2 || a.len() > 3 {
        bail!(
            "terra-geom-v2 requires [lon,lat] or [lon,lat,z] positions (found {} ordinates)",
            a.len()
        );
    }
    let lon = round_ord(a[0].as_f64(), LIMIT_XY, ORD_SCALE, "longitude")?;
    let lat = round_ord(a[1].as_f64(), 90.0, ORD_SCALE, "latitude")?;
    let z = if a.len() == 3 {
        Some(round_ord(a[2].as_f64(), meta.z_limit(), Z_SCALE, "height")?)
    } else {
        None
    };
    Ok((lon, lat, z))
}

fn round_ord(raw: Option<f64>, limit: f64, scale: f64, what: &str) -> Result<i64> {
    let v = raw.ok_or_else(|| anyhow!("{what} is not a number"))?;
    if !v.is_finite() {
        bail!("{what} is not finite");
    }
    if v.abs() > limit {
        bail!("{what} {v} out of range ±{limit}");
    }
    let scaled = v * scale;
    let rounded = if scaled >= 0.0 {
        (scaled + 0.5).floor()
    } else {
        (scaled - 0.5).ceil()
    };
    Ok(rounded as i64)
}

/// Planar key used for ordering/orientation decisions (Z deliberately ignored).
fn xy_of(p: &Pt) -> (i64, i64) {
    (p.0, p.1)
}

fn sort_dedup_pts(pts: &mut Vec<Pt>) {
    pts.sort();
    pts.dedup();
}

// ---------------------------------------------------------------------------
// Canonicalization (structure identical to v1, positions carry Z)
// ---------------------------------------------------------------------------

fn canon_ring(coords: &Value, want_ccw: bool, meta: &Meta) -> Result<Option<Vec<Pt>>> {
    let arr = as_array(coords, "ring coordinates")?;
    let mut pts: Vec<Pt> = Vec::with_capacity(arr.len());
    for c in arr {
        pts.push(pos(c, meta)?);
    }
    pts.dedup();
    if pts.len() > 1 && xy_of(pts.first().unwrap()) == xy_of(pts.last().unwrap()) {
        // Closing vertex is implicit even when its Z repeats the start.
        pts.pop();
    }
    if pts.len() < 3 {
        return Ok(None);
    }
    let planar: Vec<(i64, i64)> = pts.iter().map(xy_of).collect();
    let area = signed_area2(&planar);
    if area == 0 {
        return Ok(None);
    }
    if (area > 0) != want_ccw {
        pts.reverse();
    }
    rotate_to_min_xy(&mut pts);
    Ok(Some(pts))
}

fn canon_line(coords: &Value, meta: &Meta) -> Result<Option<Vec<Pt>>> {
    let arr = as_array(coords, "LineString coordinates")?;
    let mut pts: Vec<Pt> = Vec::with_capacity(arr.len());
    for c in arr {
        pts.push(pos(c, meta)?);
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

fn collect_positions(coords: &Value, meta: &Meta) -> Result<Vec<Pt>> {
    let arr = as_array(coords, "MultiPoint coordinates")?;
    let mut pts = Vec::with_capacity(arr.len());
    for c in arr {
        pts.push(pos(c, meta)?);
    }
    Ok(pts)
}

/// A canonical polygon: CCW shell + CW holes (sorted, de-duplicated).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Poly {
    shell: Vec<Pt>,
    holes: Vec<Vec<Pt>>,
}

fn canon_polygon(coords: &Value, meta: &Meta) -> Result<Option<Poly>> {
    let rings = as_array(coords, "Polygon coordinates")?;
    if rings.is_empty() {
        return Ok(None);
    }
    let Some(shell) = canon_ring(&rings[0], true, meta)? else {
        return Ok(None);
    };
    let mut holes = Vec::new();
    for r in &rings[1..] {
        if let Some(h) = canon_ring(r, false, meta)? {
            holes.push(h);
        }
    }
    holes.sort();
    holes.dedup();
    Ok(Some(Poly { shell, holes }))
}

/// Twice the signed area (`i128` to stay exact); `> 0` means CCW.
fn signed_area2(pts: &[(i64, i64)]) -> i128 {
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

fn rotate_to_min_xy(pts: &mut [Pt]) {
    if let Some((idx, _)) = pts.iter().enumerate().min_by_key(|(_, p)| xy_of(p)) {
        pts.rotate_left(idx);
    }
}

// ---------------------------------------------------------------------------
// Serialization (fixed key order; 7 decimals XY, 5 decimals Z)
// ---------------------------------------------------------------------------

fn push_crs(out: &mut String, crs: &str) {
    let _ = write!(out, "\"crs\":\"{crs}\"");
}

/// Emit `crs` opening + the remaining positional members, in fixed order.
/// (Callers start documents with `{"crs":` …; we simplify by writing the
/// leading brace separately.)
fn push_meta_tail(out: &mut String, meta: &Meta) {
    let _ = write!(out, ",\"elevation_ref\":\"{}\"", meta.elevation_ref);
    let unit = match meta.vertical_units {
        Units::Metres => "m",
        Units::Feet => "ft",
    };
    let _ = write!(
        out,
        ",\"vertical_units\":\"{unit}\",\"precision\":{}",
        meta.precision
    );
}

fn open_doc(meta: &Meta, ty: &str) -> String {
    let mut o = String::from("{");
    push_crs(&mut o, meta.crs);
    push_meta_tail(&mut o, meta);
    let _ = write!(o, ",\"type\":\"{ty}\",\"coordinates\":");
    o
}

fn push_ord(out: &mut String, v: i64, scale: i64, decimals: usize) {
    let a = v.unsigned_abs();
    if v < 0 {
        out.push('-');
    }
    let ip = a / scale as u64;
    let frac = a % scale as u64;
    if decimals == 0 {
        let _ = write!(out, "{ip}");
    } else {
        let _ = write!(out, "{ip}.{frac:0width$}", width = decimals);
    }
}

fn push_pt(out: &mut String, p: &Pt, meta: &Meta) {
    out.push('[');
    push_ord(out, p.0, ORD_SCALE_I, 7);
    out.push(',');
    push_ord(out, p.1, ORD_SCALE_I, 7);
    if let Some(z) = p.2 {
        out.push(',');
        // Fixed-width grid form: always five fractional digits (trailing zeros
        // kept). `precision` is still part of the digest via its own member —
        // this keeps canonical output idempotent for any declared precision.
        let _ = meta;
        push_ord(out, z, Z_SCALE_I, 5);
    }
    out.push(']');
}

fn push_pts(out: &mut String, pts: &[Pt], meta: &Meta) {
    for (i, p) in pts.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        push_pt(out, p, meta);
    }
}

/// Serialize an *open* ring, re-appending the implicit closing vertex.
fn push_ring(out: &mut String, ring: &[Pt], meta: &Meta) {
    out.push('[');
    push_pts(out, ring, meta);
    if let Some(first) = ring.first() {
        out.push(',');
        push_pt(out, first, meta);
    }
    out.push(']');
}

fn emit_point(meta: &Meta, pt: Pt) -> String {
    let mut s = open_doc(meta, "Point");
    push_pt(&mut s, &pt, meta);
    s.push('}');
    s
}

fn emit_multi_point(meta: &Meta, ty: &str, pts: &[Pt]) -> String {
    let mut o = open_doc(meta, ty);
    o.push('[');
    push_pts(&mut o, pts, meta);
    o.push_str("]}");
    o
}

fn emit_line(meta: &Meta, line: &[Pt]) -> String {
    let mut o = open_doc(meta, "LineString");
    o.push('[');
    push_pts(&mut o, line, meta);
    o.push_str("]}");
    o
}

fn emit_lines(meta: &Meta, ty: &str, lines: &[Vec<Pt>]) -> String {
    let mut o = open_doc(meta, ty);
    o.push('[');
    for (i, l) in lines.iter().enumerate() {
        if i > 0 {
            o.push(',');
        }
        o.push('[');
        push_pts(&mut o, l, meta);
        o.push(']');
    }
    o.push_str("]}");
    o
}

fn emit_polygon(meta: &Meta, poly: &Poly) -> String {
    let mut o = open_doc(meta, "Polygon");
    o.push('[');
    push_ring(&mut o, &poly.shell, meta);
    for h in &poly.holes {
        o.push(',');
        push_ring(&mut o, h, meta);
    }
    o.push_str("]}");
    o
}

fn emit_multipolygon(meta: &Meta, polys: &[Poly]) -> String {
    let mut o = open_doc(meta, "MultiPolygon");
    o.push('[');
    for (i, poly) in polys.iter().enumerate() {
        if i > 0 {
            o.push(',');
        }
        o.push('[');
        push_ring(&mut o, &poly.shell, meta);
        for h in &poly.holes {
            o.push(',');
            push_ring(&mut o, h, meta);
        }
        o.push(']');
    }
    o.push_str("]}");
    o
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical::geometry_digest_hex;
    use serde::Deserialize;

    const META: &str = "\"crs\":\"EPSG:4326\",\"elevation_ref\":\"EGM2008\",\"vertical_units\":\"m\",\"precision\":3,";

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
        serde_json::from_str(include_str!("../testdata/geometry_digest_v2.json")).unwrap()
    }

    #[test]
    fn published_vectors_match() {
        let vs = load();
        assert_eq!(vs.version, GEOMETRY_DIGEST_VERSION_V2);
        assert!(vs.vectors.len() >= 8);
        for v in &vs.vectors {
            let text = serde_json::to_string(&v.geojson).unwrap();
            assert_eq!(
                geometry_digest_hex_v2(&text).unwrap(),
                v.digest,
                "vector {:?} digest mismatch",
                v.name
            );
        }
    }

    #[test]
    fn canonical_output_is_idempotent() {
        let vs = load();
        for v in &vs.vectors {
            let text = serde_json::to_string(&v.geojson).unwrap();
            let once = canonicalize_v2(&text).unwrap();
            let twice = canonicalize_v2(&once)
                .unwrap_or_else(|e| panic!("{} canonical not re-parseable: {once} ({e})", v.name));
            assert_eq!(twice, once, "{} not idempotent", v.name);
        }
    }

    #[test]
    fn ring_start_orientation_and_key_order_are_irrelevant() {
        let base = format!(
            "{{{META}\"type\":\"Polygon\",\"coordinates\":[[[8.5,47.4,400],[8.6,47.4,401],[8.6,47.5,402],[8.5,47.5,403],[8.5,47.4,400]]]}}"
        );
        // reversed (CW), rotated start, over-precise, key order swapped
        let variant = "{\"coordinates\":[[[8.60,47.40,401.0000],[8.50,47.40,400.0000],\n                              [8.50,47.50,403.0000],[8.60,47.50,402.0000],\n                              [8.60,47.40,401.0000]]],\n             \"type\":\"Polygon\",\"precision\":3,\"vertical_units\":\"m\",\n             \"elevation_ref\":\"EGM2008\",\"crs\":\"urn:ogc:def:crs:OGC:1.3:CRS84\"}".to_string();
        assert_eq!(
            geometry_digest_hex_v2(&base).unwrap(),
            geometry_digest_hex_v2(&variant).unwrap()
        );
    }

    #[test]
    fn height_changes_digest_but_not_planar_structure() {
        let a = format!("{{{META}\"type\":\"Point\",\"coordinates\":[8.5,47.4,400]}}");
        let b = format!("{{{META}\"type\":\"Point\",\"coordinates\":[8.5,47.4,401]}}");
        assert_ne!(
            geometry_digest_hex_v2(&a).unwrap(),
            geometry_digest_hex_v2(&b).unwrap()
        );
        // same XY, different Z -> canonical docs differ only in the ordinate
        let ca = canonicalize_v2(&a).unwrap();
        let cb = canonicalize_v2(&b).unwrap();
        assert!(ca.contains("[8.5000000,47.4000000,400.00000]"));
        assert!(cb.contains("[8.5000000,47.4000000,401.00000]"));
    }

    #[test]
    fn v2_domain_tag_separates_it_from_v1() {
        // Same 2D geometry: v1 document equals the bare v1-style output; the v2
        // digest must differ from the v1 digest purely by the domain tag.
        let poly_2d = r#"{"type":"Polygon","coordinates":[[[8.5,47.4],[8.6,47.4],[8.6,47.5],[8.5,47.5],[8.5,47.4]]]}"#;
        let with_meta = format!(
            "{{{META}\"type\":\"Polygon\",\"coordinates\":[[[8.5,47.4],[8.6,47.4],[8.6,47.5],[8.5,47.5],[8.5,47.4]]]}}"
        );
        let v1 = geometry_digest_hex(poly_2d).unwrap();
        let v2 = geometry_digest_hex_v2(&with_meta).unwrap();
        assert_ne!(v1, v2);
        // The v2 canonical doc carries the metadata header; body matches v1 style.
        let c2 = canonicalize_v2(&with_meta).unwrap();
        assert!(c2.starts_with("{\"crs\":\"EPSG:4326\",\"elevation_ref\":\"EGM2008\",\"vertical_units\":\"m\",\"precision\":3,\"type\":\"Polygon\""));
    }

    #[test]
    fn missing_metadata_is_rejected() {
        // no elevation_ref
        assert!(geometry_digest_hex_v2(
            r#"{"crs":"EPSG:4326","vertical_units":"m","precision":3,"type":"Point","coordinates":[8.5,47.4,400]}"#
        )
        .is_err());
        // no crs at all (v2 requires explicit CRS)
        assert!(geometry_digest_hex_v2(
            r#"{"elevation_ref":"EGM2008","vertical_units":"m","precision":3,"type":"Point","coordinates":[8.5,47.4,400]}"#
        )
        .is_err());
        // bad vertical datum mix attempt: unknown ref rejected
        assert!(geometry_digest_hex_v2(
            r#"{"crs":"EPSG:4326","elevation_ref":"NAVD88","vertical_units":"m","precision":3,"type":"Point","coordinates":[8.5,47.4,400]}"#
        )
        .is_err());
        // unsupported horizontal CRS rejected
        assert!(geometry_digest_hex_v2(
            r#"{"crs":"EPSG:3857","elevation_ref":"EGM2008","vertical_units":"m","precision":3,"type":"Point","coordinates":[8.5,47.4,400]}"#
        )
        .is_err());
        // precision changes the digest
        let p3 = r#"{"crs":"EPSG:4326","elevation_ref":"EGM2008","vertical_units":"m","precision":3,"type":"Point","coordinates":[8.5,47.4,400]}"#;
        let p2 = r#"{"crs":"EPSG:4326","elevation_ref":"EGM2008","vertical_units":"m","precision":2,"type":"Point","coordinates":[8.5,47.4,400]}"#;
        assert_ne!(
            geometry_digest_hex_v2(p3).unwrap(),
            geometry_digest_hex_v2(p2).unwrap()
        );
    }

    #[test]
    fn geometry_collection_sorts_children() {
        let a = format!(
            "{{{META}\"type\":\"GeometryCollection\",\"geometries\":[\
             {{\"type\":\"Point\",\"coordinates\":[8.6,47.5,500]}},\
             {{\"type\":\"LineString\",\"coordinates\":[[8.5,47.4,400],[8.6,47.5,500]]}}]}}"
        );
        let b = format!(
            "{{{META}\"type\":\"GeometryCollection\",\"geometries\":[\
             {{\"type\":\"LineString\",\"coordinates\":[[8.5,47.4,400],[8.6,47.5,500]]}},\
             {{\"type\":\"Point\",\"coordinates\":[8.6,47.5,500]}}]}}"
        );
        assert_eq!(
            geometry_digest_hex_v2(&a).unwrap(),
            geometry_digest_hex_v2(&b).unwrap()
        );
    }

    #[test]
    fn rejects_out_of_range_and_bad_ordinate_count() {
        assert!(geometry_digest_hex_v2(&format!(
            "{{{META}\"type\":\"Point\",\"coordinates\":[8.5,47.4,150000]}}"
        ))
        .is_err()); // z beyond ±100000
        assert!(geometry_digest_hex_v2(&format!(
            "{{{META}\"type\":\"Point\",\"coordinates\":[8.5,47.4,400,1]}}"
        ))
        .is_err()); // 4 ordinates
    }

    #[test]
    fn quantization_matches_v1_grid() {
        // 1e-7 degrees, half-away-from-zero: +0.000000049 rounds to 0, prints 0.0000000
        let d = format!("{{{META}\"type\":\"Point\",\"coordinates\":[0.000000049,0.0,0]}}");
        let c = canonicalize_v2(&d).unwrap();
        assert!(c.contains("[0.0000000,0.0000000,0.00000]"), "{c}");
        let e = format!("{{{META}\"type\":\"Point\",\"coordinates\":[0.00000005,0.0,0]}}");
        assert!(canonicalize_v2(&e)
            .unwrap()
            .contains("[0.0000001,0.0000000,0.00000]"));
    }

    /// Regeneration helper: prints the digest for every vector so the file can
    /// be refreshed after a versioned change.
    #[test]
    #[ignore]
    fn regenerate_vectors_v2() {
        let vs = load();
        for v in &vs.vectors {
            let text = serde_json::to_string(&v.geojson).unwrap();
            println!("{} {}", v.name, geometry_digest_hex_v2(&text).unwrap());
        }
    }
}
