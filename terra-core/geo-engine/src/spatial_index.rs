//! Hierarchical 2D spatial index for global discovery (Chapter 12 §12.4).
//!
//! Decision ADR-020: Terra uses a **quadtree over EPSG:4326** for the
//! control-plane partition IDs used by World Representation Manifests. The
//! cell ID is stable and dependency-free:
//!
//! ```text
//! q/{level}/{x}/{y}
//! ```
//!
//! Level 0 is the whole world; level *n* has `2^n × 2^n` cells. `x` increases
//! eastward, `y` increases northward. Cell bounds are half-open in the query
//! sense: point lookup uses `floor`, while bbox covering is deliberately
//! conservative (it includes cells that touch the query edge) so an index
//! never misses an object sitting exactly on a boundary.
//!
//! Quadtree cells are the **discovery partition**, not a claim that geometry,
//! storage chunks, visualization tiles, and assemblies are the same thing
//! (Chapter 12 §12.4). S2/H3 can later be added as an adapter for streaming
//! visualization tiles; octree/BVH remains the right structure for 3D volumes
//! and component-level culling.
//!
//! Bounds crossing the antimeridian are not represented as a single
//! `GeoBounds`; callers split them into two boxes. This keeps containment and
//! covering rules unambiguous.

use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// Maximum quadtree level. `2^30` cells per axis keeps `x`/`y` in `u32` and
/// gives a cell width of ~0.000000335° (~3.7 cm at the equator).
pub const MAX_LEVEL: u8 = 30;

/// Covering queries above this many cells are rejected. A caller that needs
/// a global query at a deep level should ask for a coarser level first.
pub const MAX_COVERAGE_CELLS: u64 = 65_536;

const LON_SPAN: f64 = 360.0;
const LAT_SPAN: f64 = 180.0;

/// Errors from spatial-index construction and queries.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum SpatialIndexError {
    #[error("quadtree level {0} exceeds maximum {MAX_LEVEL}")]
    LevelOutOfRange(u8),
    #[error("cell ({x},{y}) is outside level {level}")]
    CellOutOfRange { level: u8, x: u32, y: u32 },
    #[error("latitude {0} is outside [-90, 90]")]
    LatitudeOutOfRange(f64),
    #[error("longitude {0} is outside [-180, 180]")]
    LongitudeOutOfRange(f64),
    #[error(
        "bounds min must not exceed max on either axis (got {min_lon},{min_lat} .. {max_lon},{max_lat})"
    )]
    InvalidBounds {
        min_lon: f64,
        min_lat: f64,
        max_lon: f64,
        max_lat: f64,
    },
    #[error("non-finite coordinate in bounds")]
    NonFiniteBounds,
    #[error("covering at level {level} would return {count} cells (limit {MAX_COVERAGE_CELLS})")]
    TooManyCells { level: u8, count: u64 },
    #[error("invalid cell id `{0}` (expected q/level/x/y)")]
    InvalidCellId(String),
}

/// An axis-aligned geographic bounding box in EPSG:4326.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeoBounds {
    pub min_lon: f64,
    pub min_lat: f64,
    pub max_lon: f64,
    pub max_lat: f64,
}

impl GeoBounds {
    pub fn new(
        min_lon: f64,
        min_lat: f64,
        max_lon: f64,
        max_lat: f64,
    ) -> Result<Self, SpatialIndexError> {
        let bounds = Self {
            min_lon,
            min_lat,
            max_lon,
            max_lat,
        };
        bounds.validate()?;
        Ok(bounds)
    }

    pub fn point(lon: f64, lat: f64) -> Result<Self, SpatialIndexError> {
        Self::new(lon, lat, lon, lat)
    }

    pub fn validate(&self) -> Result<(), SpatialIndexError> {
        if !self.min_lon.is_finite()
            || !self.min_lat.is_finite()
            || !self.max_lon.is_finite()
            || !self.max_lat.is_finite()
        {
            return Err(SpatialIndexError::NonFiniteBounds);
        }
        if !(-180.0..=180.0).contains(&self.min_lon) {
            return Err(SpatialIndexError::LongitudeOutOfRange(self.min_lon));
        }
        if !(-180.0..=180.0).contains(&self.max_lon) {
            return Err(SpatialIndexError::LongitudeOutOfRange(self.max_lon));
        }
        if !(-90.0..=90.0).contains(&self.min_lat) {
            return Err(SpatialIndexError::LatitudeOutOfRange(self.min_lat));
        }
        if !(-90.0..=90.0).contains(&self.max_lat) {
            return Err(SpatialIndexError::LatitudeOutOfRange(self.max_lat));
        }
        if self.min_lon > self.max_lon || self.min_lat > self.max_lat {
            return Err(SpatialIndexError::InvalidBounds {
                min_lon: self.min_lon,
                min_lat: self.min_lat,
                max_lon: self.max_lon,
                max_lat: self.max_lat,
            });
        }
        Ok(())
    }

    pub fn width(&self) -> f64 {
        self.max_lon - self.min_lon
    }

    pub fn height(&self) -> f64 {
        self.max_lat - self.min_lat
    }

    pub fn center(&self) -> (f64, f64) {
        (
            (self.min_lon + self.max_lon) / 2.0,
            (self.min_lat + self.max_lat) / 2.0,
        )
    }

    pub fn contains(&self, lon: f64, lat: f64) -> bool {
        lon >= self.min_lon && lon <= self.max_lon && lat >= self.min_lat && lat <= self.max_lat
    }

    /// Conservative intersection: cells sharing only an edge are included.
    pub fn intersects(&self, other: &Self) -> bool {
        self.min_lon <= other.max_lon
            && other.min_lon <= self.max_lon
            && self.min_lat <= other.max_lat
            && other.min_lat <= self.max_lat
    }

    pub fn encloses(&self, other: &Self) -> bool {
        self.min_lon <= other.min_lon
            && self.min_lat <= other.min_lat
            && self.max_lon >= other.max_lon
            && self.max_lat >= other.max_lat
    }
}

/// A quadtree cell over the WGS84 lon/lat rectangle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct QuadCell {
    level: u8,
    x: u32,
    y: u32,
}

impl QuadCell {
    pub fn root() -> Self {
        Self {
            level: 0,
            x: 0,
            y: 0,
        }
    }

    pub fn at(level: u8, x: u32, y: u32) -> Result<Self, SpatialIndexError> {
        if level > MAX_LEVEL {
            return Err(SpatialIndexError::LevelOutOfRange(level));
        }
        let side = 1u32 << level;
        if x >= side || y >= side {
            return Err(SpatialIndexError::CellOutOfRange { level, x, y });
        }
        Ok(Self { level, x, y })
    }

    pub fn from_point(lon: f64, lat: f64, level: u8) -> Result<Self, SpatialIndexError> {
        if !lon.is_finite() {
            return Err(SpatialIndexError::LongitudeOutOfRange(lon));
        }
        if !lat.is_finite() {
            return Err(SpatialIndexError::LatitudeOutOfRange(lat));
        }
        if !(-180.0..=180.0).contains(&lon) {
            return Err(SpatialIndexError::LongitudeOutOfRange(lon));
        }
        if !(-90.0..=90.0).contains(&lat) {
            return Err(SpatialIndexError::LatitudeOutOfRange(lat));
        }
        if level > MAX_LEVEL {
            return Err(SpatialIndexError::LevelOutOfRange(level));
        }

        let side = 1u32 << level;
        let width = LON_SPAN / side as f64;
        let height = LAT_SPAN / side as f64;
        // `min(side - 1)` maps the eastern/northern edge (lon 180 / lat 90)
        // into the last cell instead of producing an out-of-range index.
        let x = (((lon + 180.0) / width).floor() as i64).clamp(0, side as i64 - 1) as u32;
        let y = (((lat + 90.0) / height).floor() as i64).clamp(0, side as i64 - 1) as u32;
        Self::at(level, x, y)
    }

    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn x(&self) -> u32 {
        self.x
    }

    pub fn y(&self) -> u32 {
        self.y
    }

    pub fn is_root(&self) -> bool {
        self.level == 0
    }

    pub fn parent(&self) -> Option<Self> {
        if self.is_root() {
            return None;
        }
        Some(Self {
            level: self.level - 1,
            x: self.x / 2,
            y: self.y / 2,
        })
    }

    /// Child cells in `(west-south, east-south, west-north, east-north)` order.
    /// Returns `None` at [`MAX_LEVEL`].
    pub fn children(&self) -> Option<[Self; 4]> {
        if self.level >= MAX_LEVEL {
            return None;
        }
        let level = self.level + 1;
        let x = self.x * 2;
        let y = self.y * 2;
        Some([
            Self { level, x, y },
            Self { level, x: x + 1, y },
            Self { level, x, y: y + 1 },
            Self {
                level,
                x: x + 1,
                y: y + 1,
            },
        ])
    }

    pub fn bounds(&self) -> GeoBounds {
        let side = 1u32 << self.level;
        let width = LON_SPAN / side as f64;
        let height = LAT_SPAN / side as f64;
        GeoBounds {
            min_lon: -180.0 + self.x as f64 * width,
            min_lat: -90.0 + self.y as f64 * height,
            max_lon: -180.0 + (self.x as f64 + 1.0) * width,
            max_lat: -90.0 + (self.y as f64 + 1.0) * height,
        }
    }

    pub fn contains(&self, lon: f64, lat: f64) -> bool {
        self.bounds().contains(lon, lat)
    }

    pub fn encloses(&self, bounds: &GeoBounds) -> bool {
        self.bounds().encloses(bounds)
    }

    pub fn intersects(&self, bounds: &GeoBounds) -> bool {
        self.bounds().intersects(bounds)
    }
}

impl fmt::Display for QuadCell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "q/{}/{}/{}", self.level, self.x, self.y)
    }
}

impl FromStr for QuadCell {
    type Err = SpatialIndexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('/');
        let prefix = parts.next();
        let level = parts.next();
        let x = parts.next();
        let y = parts.next();
        if prefix != Some("q") || parts.next().is_some() {
            return Err(SpatialIndexError::InvalidCellId(s.to_string()));
        }
        let parse = |v: Option<&str>, name: &str| {
            v.and_then(|v| v.parse::<u32>().ok())
                .ok_or_else(|| SpatialIndexError::InvalidCellId(format!("{s} (bad {name})")))
        };
        let level = level
            .and_then(|v| v.parse::<u8>().ok())
            .ok_or_else(|| SpatialIndexError::InvalidCellId(s.to_string()))?;
        let x = parse(x, "x")?;
        let y = parse(y, "y")?;
        Self::at(level, x, y)
    }
}

impl Serialize for QuadCell {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for QuadCell {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(D::Error::custom)
    }
}

/// Return every quadtree cell at `level` that intersects `bounds`.
///
/// The result is ordered by `y` ascending, then `x` ascending (south to north,
/// west to east). This is a discovery partition: the caller still queries the
/// manifest index for objects inside those cells.
pub fn covering_cells(bounds: &GeoBounds, level: u8) -> Result<Vec<QuadCell>, SpatialIndexError> {
    if level > MAX_LEVEL {
        return Err(SpatialIndexError::LevelOutOfRange(level));
    }
    bounds.validate()?;

    let side = 1u32 << level;
    let width = LON_SPAN / side as f64;
    let height = LAT_SPAN / side as f64;

    let x0 = (((bounds.min_lon + 180.0) / width).floor() as i64).clamp(0, side as i64 - 1);
    let x1 = (((bounds.max_lon + 180.0) / width).floor() as i64).clamp(0, side as i64 - 1);
    let y0 = (((bounds.min_lat + 90.0) / height).floor() as i64).clamp(0, side as i64 - 1);
    let y1 = (((bounds.max_lat + 90.0) / height).floor() as i64).clamp(0, side as i64 - 1);

    let columns = (x1 - x0 + 1) as u64;
    let rows = (y1 - y0 + 1) as u64;
    let count = columns * rows;
    if count > MAX_COVERAGE_CELLS {
        return Err(SpatialIndexError::TooManyCells { level, count });
    }

    let mut cells = Vec::with_capacity(count as usize);
    for y in y0..=y1 {
        for x in x0..=x1 {
            cells.push(QuadCell {
                level,
                x: x as u32,
                y: y as u32,
            });
        }
    }
    Ok(cells)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_and_roundtrip() {
        let root = QuadCell::root();
        assert_eq!(root.to_string(), "q/0/0/0");
        assert_eq!(
            root.bounds(),
            GeoBounds::new(-180.0, -90.0, 180.0, 90.0).unwrap()
        );
        assert_eq!("q/0/0/0".parse::<QuadCell>().unwrap(), root);

        let cell = QuadCell::at(12, 2048, 1024).unwrap();
        assert_eq!(cell.to_string(), "q/12/2048/1024");
        assert_eq!(cell.to_string().parse::<QuadCell>().unwrap(), cell);
    }

    #[test]
    fn point_lookup_uses_floor_and_clamps_edges() {
        assert_eq!(
            QuadCell::from_point(-180.0, -90.0, 1).unwrap(),
            QuadCell::at(1, 0, 0).unwrap()
        );
        assert_eq!(
            QuadCell::from_point(0.0, 0.0, 1).unwrap(),
            QuadCell::at(1, 1, 1).unwrap()
        );
        assert_eq!(
            QuadCell::from_point(180.0, 90.0, 1).unwrap(),
            QuadCell::at(1, 1, 1).unwrap()
        );
        assert_eq!(
            QuadCell::from_point(-0.1, -0.1, 1).unwrap(),
            QuadCell::at(1, 0, 0).unwrap()
        );
    }

    #[test]
    fn parent_and_children_roundtrip() {
        let parent = QuadCell::at(5, 7, 9).unwrap();
        let children = parent.children().unwrap();
        assert_eq!(children.len(), 4);
        for child in children {
            assert_eq!(child.parent().unwrap(), parent);
            assert!(child.encloses(
                &GeoBounds::point(child.bounds().center().0, child.bounds().center().1).unwrap()
            ));
        }
        assert!(QuadCell::root().parent().is_none());
        assert!(QuadCell::at(MAX_LEVEL, 0, 0).unwrap().children().is_none());
    }

    #[test]
    fn covering_small_extent() {
        let bounds = GeoBounds::new(11.49, 3.84, 11.53, 3.88).unwrap();
        let cells = covering_cells(&bounds, 10).unwrap();
        assert!(!cells.is_empty());
        assert!(cells.iter().all(|c| c.intersects(&bounds)));
        let mut sorted = cells.clone();
        sorted.sort();
        assert_eq!(cells, sorted);
    }

    #[test]
    fn covering_rejects_oversized_query() {
        let world = GeoBounds::new(-180.0, -90.0, 180.0, 90.0).unwrap();
        let err = covering_cells(&world, 20).unwrap_err();
        assert!(matches!(err, SpatialIndexError::TooManyCells { .. }));
    }

    #[test]
    fn invalid_inputs_rejected() {
        assert!(GeoBounds::new(1.0, 0.0, -1.0, 1.0).is_err());
        assert!(GeoBounds::new(0.0, -91.0, 1.0, 1.0).is_err());
        assert!(QuadCell::from_point(181.0, 0.0, 3).is_err());
        assert!(QuadCell::at(1, 2, 0).is_err());
        assert!(QuadCell::at(MAX_LEVEL + 1, 0, 0).is_err());
        assert!("q/1/2/0".parse::<QuadCell>().is_err());
        assert!("nope".parse::<QuadCell>().is_err());
    }

    #[test]
    fn containment_and_enclosure() {
        let cell = QuadCell::from_point(11.5, 3.85, 8).unwrap();
        let inside = GeoBounds::new(11.49, 3.84, 11.51, 3.86).unwrap();
        assert!(cell.contains(11.5, 3.85));
        assert!(cell.encloses(&inside));
        assert!(!cell.encloses(&GeoBounds::new(-10.0, -10.0, 10.0, 10.0).unwrap()));
    }
}
