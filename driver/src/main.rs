use std::path::PathBuf;

use clap::arg;
use clap::Parser;
use bpa_rs::reconstruct;
use bpa_rs::io::load_xyz;
use bpa_rs::io::save_triangles;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long = "input", short = 'i', help = "point cloud file")]
    input: PathBuf,
    #[clap(long = "radius", short = 'r')]
    radius: f32,
    #[clap(long="output", help="output mesh file mesh", short='o', default_value=None)]
    output: Option<PathBuf>,
}

fn main() {
    let args = Cli::parse();
    println!("args: {:?}", args);
    println!("input: {:?}", args.input);
    let output = args.output.clone().unwrap_or_else(|| {
        let mut path = args.input.clone();
        path.set_extension("stl");
        path
    });

    let points = load_xyz(&args.input);

    match reconstruct(&points, args.radius) {
        Some(triangles) => {
            save_triangles(&output, &triangles);
        }
        None => {
            eprintln!("Exception occurred reconstructing the surface");
        }
    }
}