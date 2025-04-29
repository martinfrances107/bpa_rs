mod grid;
pub mod io;
mod mesh;
#[cfg(test)]
mod test;

use std::path::PathBuf;

use glam::Vec3;
use grid::Grid;
use grid::SeedResult;
use grid::ball_pivot;
use grid::find_reverse_edge_on_front;
use grid::find_seed_triangle;
use grid::get_active_edge;
use grid::glue;
use grid::join;
use grid::not_used;
use grid::on_front;
use grid::output_triangle;
use io::save_points;
use io::save_triangles_ascii;
use mesh::EdgeStatus;
use mesh::MeshEdge;
use mesh::MeshFace;
use mesh::MeshPoint;

type Cell = Vec<MeshPoint>;

#[derive(Debug)]
pub struct Triangle([Vec3; 3]);

impl Triangle {
    fn normal(&self) -> Vec3 {
        let cross = (self.0[0] - self.0[1]).cross(self.0[0] - self.0[2]);
        cross.normalize()
    }
}

#[derive(Debug)]
pub struct Point {
    pos: Vec3,
    normal: Option<Vec3>,
}

impl Point {
    fn new(pos: Vec3) -> Self {
        Self { pos, normal: None }
    }
}

pub fn reconstruct(points: &[Point], radius: f32) -> Option<Vec<Triangle>> {
    let mut grid = Grid::new(points, radius);

    match find_seed_triangle(&mut grid, radius) {
        None => {
            eprintln!("No seed triangle found");
            None
        }
        Some(SeedResult { f, ball_center }) => {
            let mut triangles: Vec<Triangle> = Vec::new();
            let mut edges: Vec<MeshEdge> = Vec::new();
            output_triangle(&f, &mut triangles);

            // auto& e0 = edges.emplace_back(MeshEdge{seed[0], seed[1], seed[2], ballCenter});
            let seed = f.0;
            let mut e0 = MeshEdge::new(&seed[0], &seed[1], seed[2].clone(), ball_center);
            let mut e1 = MeshEdge::new(&seed[1], &seed[2], seed[0].clone(), ball_center);
            let mut e2 = MeshEdge::new(&seed[2], &seed[0], seed[1].clone(), ball_center);

            e0.prev = Some(Box::new(e2.clone()));
            e1.next = Some(Box::new(e2.clone()));
            e0.next = Some(Box::new(e1.clone()));
            e2.prev = Some(Box::new(e1.clone()));
            e1.prev = Some(Box::new(e0.clone()));
            e2.next = Some(Box::new(e0.clone()));

            // TODO: Set seed.
            todo!();

            let mut front = vec![e0, e1, e2];
            let debug = true;
            if debug {
                save_triangles_ascii(&PathBuf::from("seed.stl"), &triangles)
                    .expect("Failed(debug) to write seed to file");
            }

            let debug = true;
            loop {
                let e_ij = get_active_edge(&mut front);
                if e_ij.is_none() {
                    break;
                }

                if debug {
                    save_triangles_ascii(&PathBuf::from("front.stl"), &triangles)
                        .expect("Failed(debug) to write front to file");
                }

                let o_k = ball_pivot(&e_ij.clone().unwrap(), &mut grid, radius);
                if debug {
                    save_triangles_ascii(&PathBuf::from("current_mesh.stl"), &triangles)
                        .expect("Failed(debug) writing current mesh to file");
                }

                let mut boundary_test = false;
                if let Some(o_k) = &o_k {
                    if not_used(&o_k.p) || on_front(&o_k.p) {
                        boundary_test = true;

                        output_triangle(
                            &MeshFace([
                                e_ij.clone().unwrap().a,
                                o_k.p.clone(),
                                e_ij.clone().unwrap().b,
                            ]),
                            &mut triangles,
                        );

                        let (mut e_ik, mut e_kj) = join(
                            &mut e_ij.clone().unwrap(),
                            &mut o_k.p.clone(),
                            o_k.center,
                            &mut front,
                            &mut edges,
                        );

                        if let Some(mut e_ki) = find_reverse_edge_on_front(&e_ik) {
                            glue(&mut e_ik, &mut e_ki, &mut front);
                        }

                        if let Some(mut e_jk) = find_reverse_edge_on_front(&e_kj) {
                            glue(&mut e_kj, &mut e_jk, &mut front)
                        }
                    }
                }
                if !boundary_test {
                    if debug {
                        let cb_points = match o_k {
                            Some(pr) => {
                                vec![Point::new(pr.p.pos)]
                            }
                            None => {
                                vec![]
                            }
                        };
                        save_points(&PathBuf::from("current_boundary.ply"), &cb_points)
                            .expect("could not save current boundary");
                    }
                    e_ij.unwrap().status = EdgeStatus::Boundary;
                }
            }

            if debug {
                let mut boundary_edges = vec![];

                for e in front.iter() {
                    if e.status == EdgeStatus::Boundary {
                        boundary_edges.push(Triangle([e.a.pos, e.a.pos, e.b.pos]));
                    }
                }
                save_triangles_ascii(&PathBuf::from("boundary_edges.stl"), &boundary_edges)
                    .expect("Failed writing boundary_edges to file");
            }
            Some(triangles)
        }
    }
}
