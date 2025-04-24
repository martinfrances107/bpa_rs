mod grid;
mod mesh;

#[cfg(test)]
mod test;

use glam::Vec3;
use mesh::MeshPoint;

type Cell<'a> = Vec<MeshPoint<'a>>;

struct Triangle([Vec3; 3]);

impl Triangle {
    fn normal(&self) -> Vec3 {
        let cross = (self.0[0] - self.0[1]).cross( self.0[0] - self.0[2]);
        cross.normalize()
    }
}

#[derive(Debug)]
struct Point {
    pos: Vec3,
    normal: Vec3,
}

fn main() {
    println!("Hello, world!");
}
