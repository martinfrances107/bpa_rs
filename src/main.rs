
mod grid;
mod mesh;

#[cfg(test)]
mod test;

use glm::Vec3;
use mesh::MeshPoint;

type Cell<'a> = Vec<MeshPoint<'a>>;

struct Triangle([Vec3; 3]);

impl Triangle{
  fn normal(&self) -> Vec3 {

  //  auto normal() const { return glm::normalize(glm::cross((*this)[0] - (*this)[1], (*this)[0] - (*this)[2])); }
  let cross = glm::cross(self.0[0] - self.0[1], self.0[0] - self.0[2]);
  glm::normalize(cross)
  }

}

#[derive(Debug)]
struct Point{
  pos: Vec3,
  normal: Vec3
}

fn main() {
    println!("Hello, world!");
}
