use geo::Coord;
use serde_json::Value;

use crate::error::AppError;

/// Convert a GeoJSON Polygon (EPSG:4326) into a `geo::Polygon`, validating the
/// ring structure and coordinate bounds. Shared by the fusion reachability and
/// the on-chain reconciliation handlers.
pub fn geojson_polygon(value: &Value) -> Result<geo::Polygon<f64>, AppError> {
    if value["type"].as_str() != Some("Polygon") {
        return Err(AppError::bad_request("geometry must be a GeoJSON Polygon"));
    }
    let rings = value["coordinates"]
        .as_array()
        .ok_or_else(|| AppError::bad_request("Polygon.coordinates is required"))?;
    let mut parsed = Vec::with_capacity(rings.len());

    for ring in rings {
        let ring = ring
            .as_array()
            .ok_or_else(|| AppError::bad_request("ring must be an array of positions"))?;
        let mut coords: Vec<Coord<f64>> = Vec::with_capacity(ring.len());
        for pos in ring {
            let pos = pos
                .as_array()
                .ok_or_else(|| AppError::bad_request("position must be [lon, lat]"))?;
            let lon = pos
                .first()
                .and_then(|v| v.as_f64())
                .ok_or_else(|| AppError::bad_request("position[0] (lon) required"))?;
            let lat = pos
                .get(1)
                .and_then(|v| v.as_f64())
                .ok_or_else(|| AppError::bad_request("position[1] (lat) required"))?;
            if !(-180.0..=180.0).contains(&lon) || !(-90.0..=90.0).contains(&lat) {
                return Err(AppError::bad_request("coordinate out of EPSG:4326 range"));
            }
            coords.push(Coord { x: lon, y: lat });
        }
        if coords.len() < 4 {
            return Err(AppError::bad_request("ring must have at least 4 positions"));
        }
        if coords[0] != *coords.last().expect("checked len >= 4") {
            coords.push(coords[0]);
        }
        parsed.push(coords.into());
    }

    let exterior = parsed.remove(0);
    let polygon = geo::Polygon::new(exterior, parsed.into_iter().collect::<Vec<_>>());
    Ok(polygon)
}
