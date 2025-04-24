
use glm::ceil;
use glm::ivec3;
use glm::IVec3;

use glm::Vector3;
use glm::max;
use glm::dot;
use glm::cross;

use crate::Cell;
use crate::mesh::MeshPoint;
use crate::mesh::MeshFace;
use crate::mesh::MeshEdge;
use crate::mesh::EdgeStatus;

use crate::Point;
use crate::Triangle;

struct Grid<'a>{
  cell_size: f32,
  dims: IVec3,
  cells: Vec<Cell<'a>>,
  lower: Vector3<f32>,
  upper: Vector3<f32>,
}

use std::collections::VecDeque;
use std::ops::Div;

impl  Grid<'_> {
    pub fn new(points: &Vec<Point>, radius: f32 ) -> Self {
      let cell_size = 2_f32 * radius;
      let mut lower = points.first().expect("Vec with no points").pos;
      let mut upper = points.first().expect("Vec with no points(2)").pos;
      for p in points{
        for i in 0..3{
          lower[i] = lower[i].min(p.pos[i]);
          upper[i] = upper[i].max(p.pos[i]);
        }
      }

      let ceil_float = ceil((upper-lower).div(cell_size));
      let ceil: IVec3 = ivec3(ceil_float[0] as i32, ceil_float[1] as i32, ceil_float[2] as i32);
      let dims = max(ceil, ivec3(1,1,1));

      let cells = Vec::with_capacity((dims.x * dims.y * dims.z) as usize);

      let grid = Grid{
        cell_size,
        dims,
        cells,
        lower,
        upper
    };

      for p in points{
        let index = grid.cell_index(p.pos);
        // grid.cell(index).points.push(p);
      }

      grid

      }

      fn cell_index(&self, point: Vector3<f32> ) -> Vector3<i32> {
        let diff = (point - self.lower) / self.cell_size;
        let index = ivec3(diff.x as i32, diff.y as i32, diff.z as i32);
        glm::clamp(index, ivec3(0,0,0), self.dims - 1)
      }

      fn cell(&self, index: IVec3) -> &Cell{
        let index = index.x * self.dims.x * self.dims.y + index.y * self.dims.x + index.x;
       &self.cells[index as usize]
      }

      fn spherical_neighborhood(&self, point: &Point, ignore: &IVec3 ) -> Vec<&MeshPoint<'_>> {
        let center_index = self.cell_index(point.pos);
        // Just an estimate.
        let capacity =  self.cell(center_index).len() * 27;
        let mut result = Vec::with_capacity(capacity);
        for x_off in -1..=1 {
          for y_off in -1..=1 {
            for z_off in -1..=1 {
              let index = center_index + ivec3(x_off, y_off, z_off);
             if index.x < 0 || index.x >= self.dims.x ||
                index.y < 0 || index.y >= self.dims.y ||
                index.z < 0 || index.z >= self.dims.z {
                  continue;
                }
              if ignore == &index {
                continue;
              }
              let cell = self.cell(index);
              todo!();
              // for p in cell{
              //   if p.pos.distance(point.pos) < point.radius{
              //     result.push(p.clone());
              //   }
              // }
          }
        }
      }
      result

  }


}

// from
// https://gamedev.stackexchange.com/questions/60630/how-do-i-find-the-circumcenter-of-a-triangle-in-3d
pub(crate) fn compute_ball_center(f: MeshFace, radius: f32) -> Option<Vector3<f32>> {
    let ac = f.0[2].pos - f.0[0].pos;
    let ab = f.0[1].pos - f.0[0].pos;
    let ab_cross_ac = glm::cross(ab, ac);
		// const vec3 toCircumCircleCenter = (cross(abXac, ab) * dot(ac, ac) + cross(ac, abXac) * dot(ab, ab)) / (2 * dot(abXac, abXac));

    let to_circum_circle_center =
      (cross(ab_cross_ac, ab) * dot(ac, ac) + cross(ac, ab_cross_ac) * dot(ab, ab)) / (2.0 * dot(ab_cross_ac, ab_cross_ac));


    let circum_circle_center = f.0[0].pos + to_circum_circle_center;

    let height_squared = radius * radius - dot(to_circum_circle_center, to_circum_circle_center);
    if height_squared.is_sign_negative() {
        return None;
      }

   Some(circum_circle_center + f.normal() * height_squared.sqrt())

}


fn is_ball_empty(ball_center: Vector3<f32>, points: &Vec<MeshPoint>, radius: f32) -> bool {
  !points.iter().any(|p| {

    let length_squared = glm::length(p.pos - ball_center);
    // TODO epsilon
    length_squared < radius * radius - 1e-4
  })
}

struct SeedResult<'a> {
  f: MeshFace<'a>,
  ball_center: Vector3<f32>,
}

fn find_seed_triangle<'a>(grid: &Grid<'a>, radius: f32) -> Option<SeedResult<'a>> {
			for  cell in &grid.cells {
        let avg_normal = cell.iter().fold(Vector3::new(0.0, 0.0, 0.0), |acc, p| {
          match p.normal {
            Some(n) => acc + n,
            None => acc
          }
      });

				for  p1 in cell.iter() {
          // auto neighborhood = grid.sphericalNeighborhood(p1.pos, {p1.pos});

          }
        }
        todo!()
}

fn get_active_edge<'a> (front: &'a mut Vec<&MeshEdge<'a>>) -> Option<&'a MeshEdge<'a>> {
 loop {
  match front.last() {
    None => return None,
    Some(edge) => {
      if edge.status == EdgeStatus::Active {
        return Some(edge);

      }
       // cleanup non-active edges from front
      front.pop();
    }
  }
}
}

fn not_used(p: &MeshPoint) -> bool {
  !p.used
}

fn on_front(_p: &MeshPoint) -> bool {
  // p.edges.iter().any(|e| e.status == EdgeStatus::Active)
  todo!();
}

fn remove(e: &mut MeshEdge) {
  e.status = EdgeStatus::Inner;

}

fn output_triangle( f: MeshFace, triangles: &mut Vec<Triangle>){
 triangles.push(Triangle([f.0[0].pos, f.0[1].pos, f.0[2].pos]));
}

fn join<'a>(e_ij: &MeshEdge, o_k: MeshPoint, o_k_ball_center: Vector3<f32>, front: &mut Vec<MeshEdge>, edges: &VecDeque<MeshEdge>) -> (MeshEdge<'a>, MeshEdge<'a>) {
// auto& e_ik = edges.emplace_back(MeshEdge{e_ij->a, o_k, e_ij->b, o_k_ballCenter});
let mut e_ik = MeshEdge::new(e_ij.a, &o_k, e_ij.b, o_k_ball_center);
let mut e_kj = MeshEdge::new(&o_k, e_ij.b, e_ij.a, o_k_ball_center);

  //TODO this will get complicated

todo!()
  }

  fn glue(a: &mut MeshEdge, b: &mut MeshEdge, front: &mut Vec<MeshEdge>) {

   // Debug here.


			// case 1
			if a.next == Some(b) && a.prev == Some(b) && b.next == Some(a) && b.prev == Some(a) {
				remove(a);
				remove(b);
				return;
			}

      // case 2
			if a.next == Some(b) && b.prev == Some(a) {
        // TODO must work out a pattern
				// a->prev->next = b->next;
				// b->next->prev = a->prev;
				remove(a);
				remove(b);
				return;
			}

			if a.prev == Some(b) && b.next == Some(a) {
				// a->next->prev = b->prev;
				// b->prev->next = a->next;
				remove(a);
				remove(b);
				return;
			}
			// case 3/4
			// a->prev->next = b->next;
			// b->next->prev = a->prev;
			// a->next->prev = b->prev;
			// b->prev->next = a->next;
			remove(a);
			remove(b);


  }

fn  find_reverse_edge_on_front<'a>(edge: &MeshEdge<'a>) -> Option<MeshEdge<'a>> {
    match edge.a.edges {
      None => return None,
      Some(ref edges) => {
        for e in edges.iter() {
          if e.a == edge.b {
            // FIXME:
            todo!()
            // return Some(*e);
          }
        }
        None
      }
    }

		}

fn reconstruct(points: Vec<Point>, radius: f32) -> Option<Vec<Triangle>> {
    let grid = Grid::new(&vec![], 0.0);

    match find_seed_triangle(&grid, radius){
      None => {
        eprintln!("No seed triangle found");
        return None;
      },
      Some(SeedResult{ f: seed, ball_center}) => {
        let mut triangles: Vec<Triangle> = Vec::new();
        let edges: VecDeque<MeshEdge> = VecDeque::new();
        output_triangle(seed, &mut triangles);

        // auto& e0 = edges.emplace_back(MeshEdge{seed[0], seed[1], seed[2], ballCenter});
        // let e0 = MeshEdge::new(&seed.0[0], &seed.0[1], &seed.0[2], ball_center);
        // let e1 = MeshEdge::new(&seed.0[1], &seed.0[2], &seed.0[0], ball_center);
        // TODO must fix
        // let e2 = MeshEdge::new(&seed.0[2], &seed.0[0], &seed.0[1], ball_center);

        // set next and prev
        todo!();

        // set edges

        // let front: Vec<&MeshEdge> = vec![&e0, &e1, &e2];

        // debug save triangles.

        // loop{
        //   if let Some(e_ij) = get_active_edge(&mut front){

        //   } else {
        //     break;
        //   }

        // }

      }
    }




    todo!()
  }