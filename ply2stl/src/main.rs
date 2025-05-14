#![deny(clippy::all)]
#![warn(clippy::cargo)]
#![warn(clippy::complexity)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::perf)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![allow(clippy::many_single_char_names)]
#![doc = include_str!("../../README.md")]

use std::path::PathBuf;

use bpa_rs::io::load_xyz;
use bpa_rs::io::save_triangles_ascii;
use bpa_rs::reconstruct;
use clap::Parser;
use clap::arg;

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
    let args = Cli::parse();
    println!("args: {args:?}");
    println!("input: {:?}", args.input);
    let output = args.output.clone().unwrap_or_else(|| {
        let mut path = args.input.clone();
        path.set_extension("stl");
        path
    });

    let points = load_xyz(&args.input)?;

    match reconstruct(&points, args.radius) {
        Some(triangles) => {
            if let Err(e) = save_triangles_ascii(&output, &triangles) {
                eprintln!("Exception occurred while writing to file. {e}");
            }
        }
        None => {
            eprintln!("Exception occurred reconstructing the surface");
        }
    }

    Ok(())
}
