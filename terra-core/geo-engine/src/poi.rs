use geo::{Coord, Distance, Haversine, Point};

use crate::osm::{OsmData, Poi};

/// All POIs within `radius_m` of `center`.
pub fn pois_within<'a>(data: &'a OsmData, center: Coord<f64>, radius_m: f64) -> Vec<&'a Poi> {
    let center_pt = Point::new(center.x, center.y);
    data.pois
        .iter()
        .filter(|poi| Haversine::distance(Point::new(poi.coord.x, poi.coord.y), center_pt) <= radius_m)
        .collect()
}

/// The `limit` nearest POIs to `center`, optionally filtered by category.
pub fn pois_near<'a>(
    data: &'a OsmData,
    center: Coord<f64>,
    limit: usize,
    category: Option<&str>,
) -> Vec<&'a Poi> {
    let center_pt = Point::new(center.x, center.y);
    let mut matches: Vec<&Poi> = data
        .pois
        .iter()
        .filter(|poi| category.is_none_or(|c| poi.category == c))
        .collect();

    matches.sort_by(|a, b| {
        let da = Haversine::distance(Point::new(a.coord.x, a.coord.y), center_pt);
        let db = Haversine::distance(Point::new(b.coord.x, b.coord.y), center_pt);
        da.total_cmp(&db)
    });

    matches.truncate(limit);
    matches
}
