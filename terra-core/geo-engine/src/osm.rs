use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use geo::Coord;

/// A road extracted from OSM (a `highway=*` way).
#[derive(Debug, Clone)]
pub struct RoadWay {
    pub id: i64,
    pub name: Option<String>,
    pub highway: String,
    pub oneway: bool,
    /// Node ids in traversal order.
    pub nodes: Vec<i64>,
}

/// A point of interest extracted from OSM.
#[derive(Debug, Clone)]
pub struct Poi {
    pub id: i64,
    pub name: Option<String>,
    /// Primary OSM key used to classify the POI (e.g. "amenity").
    pub category: String,
    /// The value for `category` (e.g. "school", "fuel").
    pub kind: String,
    pub tags: HashMap<String, String>,
    pub coord: Coord<f64>,
}

/// Everything we care about from a PBF extract.
#[derive(Debug, Default)]
pub struct OsmData {
    /// node id -> coordinate
    pub nodes: HashMap<i64, Coord<f64>>,
    /// drivable/walkable roads (filtered `highway=*` ways)
    pub roads: Vec<RoadWay>,
    /// points of interest
    pub pois: Vec<Poi>,
}

/// Parse an OSM PBF file, keeping roads and POIs.
pub fn read_osm_pbf(path: &Path) -> Result<OsmData> {
    let reader = osmpbf::ElementReader::from_path(path)
        .with_context(|| format!("failed to open {}", path.display()))?;

    let mut data = OsmData::default();
    reader.for_each(|element| match element {
        osmpbf::Element::Node(node) => {
            let coord = Coord {
                x: node.lon(),
                y: node.lat(),
            };
            data.nodes.insert(node.id(), coord);
            if let Some(poi) = poi_from_tags(node.id(), coord, node.tags()) {
                data.pois.push(poi);
            }
        }
        osmpbf::Element::DenseNode(node) => {
            let coord = Coord {
                x: node.lon(),
                y: node.lat(),
            };
            data.nodes.insert(node.id(), coord);
            if let Some(poi) = poi_from_tags(node.id(), coord, node.tags()) {
                data.pois.push(poi);
            }
        }
        osmpbf::Element::Way(way) => {
            let tags: HashMap<String, String> = way
                .tags()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
            if let Some(highway) = tags.get("highway") {
                if is_road_highway(highway) {
                    let oneway = matches!(
                        tags.get("oneway").map(String::as_str),
                        Some("yes") | Some("true") | Some("1")
                    );
                    data.roads.push(RoadWay {
                        id: way.id(),
                        name: tags.get("name").cloned(),
                        highway: highway.clone(),
                        oneway,
                        nodes: way.refs().collect(),
                    });
                }
            }
        }
        _ => {}
    })?;

    Ok(data)
}

/// `highway=*` values that count as a road for access purposes.
fn is_road_highway(highway: &str) -> bool {
    matches!(
        highway,
        "motorway"
            | "trunk"
            | "primary"
            | "secondary"
            | "tertiary"
            | "unclassified"
            | "residential"
            | "service"
            | "motorway_link"
            | "trunk_link"
            | "primary_link"
            | "secondary_link"
            | "tertiary_link"
            | "living_street"
            | "road"
            | "track"
    )
}

/// OSM keys that make a node a point of interest.
const POI_KEYS: &[&str] = &[
    "amenity",
    "shop",
    "tourism",
    "leisure",
    "office",
    "craft",
    "healthcare",
    "emergency",
    "public_transport",
    "railway",
];

fn poi_from_tags<'a, I>(id: i64, coord: Coord<f64>, tags: I) -> Option<Poi>
where
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    let mut map = HashMap::new();
    for (k, v) in tags {
        map.insert(k.to_string(), v.to_string());
    }
    let (category, kind) = POI_KEYS
        .iter()
        .find_map(|key| map.get(*key).map(|value| (key.to_string(), value.clone())))?;
    Some(Poi {
        id,
        name: map.get("name").cloned(),
        category,
        kind,
        tags: map,
        coord,
    })
}
