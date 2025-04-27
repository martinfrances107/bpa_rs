mod grid;
mod io;
mod mesh;
#[cfg(test)]
mod test;

use std::path::PathBuf;

use clap::arg;
use glam::Vec3;
use mesh::MeshPoint;

use crate::grid::reconstruct;

use io::{load_xyz, save_triangles};

type Cell = Vec<MeshPoint>;

struct Triangle([Vec3; 3]);

impl Triangle {
    fn normal(&self) -> Vec3 {
        let cross = (self.0[0] - self.0[1]).cross(self.0[0] - self.0[2]);
        cross.normalize()
    }
}

#[derive(Debug)]
struct Point {
    pos: Vec3,
    normal: Option<Vec3>,
}

impl Point {
    fn new(pos: Vec3) -> Self {
        Self { pos, normal: None }
    }
}

use clap::Parser;

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
