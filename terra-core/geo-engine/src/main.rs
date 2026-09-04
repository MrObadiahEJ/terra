use std::path::Path;
use std::process::ExitCode;

use anyhow::{Context, Result};
use geo::Coord;
use terra_geo::{access, build_graph, osm, poi};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err:#}");
            eprintln!("usage:");
            eprintln!("  terra-geo import <file.pbf>");
            eprintln!("  terra-geo query <file.pbf> <lon> <lat> [radius_m]");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<()> {
    match args.get(1).map(String::as_str) {
        Some("import") => {
            let path = args.get(2).context("missing <file.pbf>")?;
            import(path)
        }
        Some("query") => {
            let path = args.get(2).context("missing <file.pbf>")?;
            let lon: f64 = args.get(3).context("missing <lon>")?.parse()?;
            let lat: f64 = args.get(4).context("missing <lat>")?.parse()?;
            let radius: f64 = args
                .get(5)
                .map(|r| r.parse())
                .transpose()?
                .unwrap_or(1000.0);
            query(path, lon, lat, radius)
        }
        _ => Err(anyhow::anyhow!("unknown command")),
    }
}

fn import(path: &str) -> Result<()> {
    let data = osm::read_osm_pbf(Path::new(path))?;
    let graph = build_graph(&data);
    println!("nodes:        {}", data.nodes.len());
    println!("roads:        {}", data.roads.len());
    println!("road segments:{}", graph.segment_count());
    println!("road length:  {:.1} km", graph.total_length_m() / 1000.0);
    println!("pois:         {}", data.pois.len());
    if let Some(bbox) = graph.bbox {
        println!(
            "bbox:         ({:.5},{:.5}) -> ({:.5},{:.5})",
            bbox.min().x,
            bbox.min().y,
            bbox.max().x,
            bbox.max().y
        );
    }
    Ok(())
}

fn query(path: &str, lon: f64, lat: f64, radius_m: f64) -> Result<()> {
    let data = osm::read_osm_pbf(Path::new(path))?;
    let graph = build_graph(&data);
    let origin = Coord { x: lon, y: lat };

    println!("nearest roads to ({lon},{lat}):");
    for hit in access::nearest_road_access(&graph, origin, 5) {
        println!(
            "  {:>8.1} m  {} [{}]  ({} {})",
            hit.distance_m,
            hit.road_name.as_deref().unwrap_or("unnamed"),
            hit.highway,
            hit.point.x,
            hit.point.y
        );
    }

    println!("nearest POIs within {radius_m} m:");
    for p in poi::pois_near(&data, origin, 10, None) {
        let name = p.name.as_deref().unwrap_or("(unnamed)");
        println!("  {} ({}) — {}", name, p.kind, p.category);
    }
    Ok(())
}
