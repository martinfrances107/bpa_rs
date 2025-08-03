#![deny(clippy::all)]
#![warn(clippy::cargo)]
#![warn(clippy::complexity)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::perf)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
//! Convert a point cloud (.ply) file into a STL mesh

use std::path::PathBuf;

use bpa_rs::io::save_triangles;
use bpa_rs::{Point, reconstruct};
use clap::Parser;
use log::info;

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Cli {
    #[arg(long = "input", short = 'i', help = "point cloud file")]
    input: PathBuf,
    #[clap(long = "radius", short = 'r')]
    radius: f32,
    #[clap(long="output", help="output mesh file mesh", short='o', default_value=None)]
    output: Option<PathBuf>,
}

fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("starting up");

    let args = Cli::parse();
    let output = args.output.clone().unwrap_or_else(|| {
        let mut path = args.input.clone();
        path.set_extension("stl");
        path
    });

    let points: Vec<Point> = bpa_rs::io::load_ply(&args.input)?;

    match reconstruct(&points, args.radius) {
        Some(triangles) => {
            info!("reconstruction complete... saving");
            if let Err(e) = save_triangles(&output, &triangles) {
                eprintln!("Exception occurred while writing to file. {e}");
            }
        }
        None => {
            eprintln!("Exception occurred reconstructing the surface");
        }
    }

    Ok(())
}
