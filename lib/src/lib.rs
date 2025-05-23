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

/// Stores the point cloud, helper functions and the main algorithm.
pub mod grid;
/// Load and Save points and meshes.
pub mod io;
/// Internal structures for Points, Edges and Faces.
pub mod mesh;
#[cfg(test)]
mod test;

use core::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::vec;

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

const DEBUG: bool = false;

// Why  Rc<RefCell<MeshPoint>>?
//
// When looping over neighborhood points the design needs mutable access
// to cell points.
//
// for j in 0..neighborhood.len() {
//     for k in 0..neighborhood.len() {
//      /* Mutable access. */
//     }
// }
//
// dipping in and out of adjacent cells to form "neighborhood", a mutable
// collections points,
type Cell = Vec<Rc<RefCell<MeshPoint>>>;

/// A series of Points
#[derive(Debug)]
pub struct Triangle([Vec3; 3]);

impl Triangle {
    fn normal(&self) -> Vec3 {
        let cross = (self.0[0] - self.0[1]).cross(self.0[0] - self.0[2]);
        cross.normalize()
    }
}

/// Base primitive for triangles and meshes.
#[derive(Debug)]
pub struct Point {
    /// Position of the point
    pub pos: Vec3,
    /// Normal of the point
    pub normal: Vec3,
}

/// Returns a mesh from a point cloud.
///
/// Main entry point for this library.
///
/// # Panics
///  (Debug ONLY) File system issues when `saving_points()`'s or `saving_triangle()`'s
#[must_use]
pub fn reconstruct(points: &[Point], radius: f32) -> Option<Vec<Triangle>> {
    let mut grid = Grid::new(points, radius);

    match find_seed_triangle(&grid, radius) {
        None => {
            eprintln!("No seed triangle found");
            None
        }
        Some(SeedResult { f, ball_center }) => {
            let mut triangles: Vec<Triangle> = Vec::new();
            let mut edges: Vec<Rc<RefCell<MeshEdge>>> = Vec::new();
            output_triangle(&f, &mut triangles);

            let seed = f.0;

            let e0 = Rc::new(RefCell::new(MeshEdge::new(
                &seed[0],
                &seed[1],
                &seed[2].clone(),
                ball_center,
            )));
            edges.push(e0.clone());

            let e1 = Rc::new(RefCell::new(MeshEdge::new(
                &seed[1],
                &seed[2],
                &seed[0].clone(),
                ball_center,
            )));
            edges.push(e1.clone());

            let e2 = Rc::new(RefCell::new(MeshEdge::new(
                &seed[2],
                &seed[0],
                &seed[1].clone(),
                ball_center,
            )));
            edges.push(e2.clone());

            e0.borrow_mut().prev = Some(e2.clone());
            e1.borrow_mut().next = Some(e2.clone());
            e0.borrow_mut().next = Some(e1.clone());
            e2.borrow_mut().prev = Some(e1.clone());
            e1.borrow_mut().prev = Some(e0.clone());
            e2.borrow_mut().next = Some(e0.clone());

            seed[0].borrow_mut().edges = vec![e0.clone(), e2.clone()];
            seed[1].borrow_mut().edges = vec![e0.clone(), e1.clone()];
            seed[2].borrow_mut().edges = vec![e1.clone(), e2.clone()];

            let mut front = vec![e0, e1, e2];
            if DEBUG {
                save_triangles_ascii(&PathBuf::from("seed.stl"), &triangles)
                    .expect("Failed(debug) to write seed to file");
            }

            while let Some(e_ij) = get_active_edge(&mut front) {
                if DEBUG {
                    save_triangles_ascii(
                        &PathBuf::from("current_active_edge.stl"),
                        &[Triangle([
                            e_ij.clone().borrow().a.borrow().pos,
                            e_ij.clone().borrow().a.borrow().pos,
                            e_ij.clone().borrow().b.borrow().pos,
                        ])],
                    )
                    .expect("Failed(debug) to write front to file");
                }

                let o_k = ball_pivot(&e_ij.clone(), &mut grid, radius);
                if DEBUG {
                    save_triangles_ascii(&PathBuf::from("current_mesh.stl"), &triangles)
                        .expect("Failed(debug) writing current mesh to file");
                }

                let mut boundary_test = false;
                if let Some(o_k) = &o_k {
                    let nu = not_used(&o_k.p.borrow());
                    let of = on_front(&o_k.p.borrow());
                    if nu || of {
                        boundary_test = true;

                        output_triangle(
                            &MeshFace([
                                e_ij.clone().borrow().a.clone(),
                                o_k.p.clone(),
                                e_ij.clone().borrow().b.clone(),
                            ]),
                            &mut triangles,
                        );

                        let (e_ik, e_kj) = join(&e_ij, &o_k.p, o_k.center, &mut front, &mut edges);
                        if let Some(e_ki) = find_reverse_edge_on_front(&e_ik.clone()) {
                            glue(&e_ik, &e_ki, &front);
                        }

                        if let Some(e_jk) = find_reverse_edge_on_front(&e_kj.clone()) {
                            glue(&e_kj.clone(), &e_jk.clone(), &front);
                        }
                    }
                }
                if !boundary_test {
                    if DEBUG {
                        if let Some(o_k_value) = o_k {
                            save_points(
                                &PathBuf::from("current_boundary.ply"),
                                &vec![o_k_value.p.borrow().pos],
                            )
                            .expect("could not save current boundary");
                        }
                    }
                    // Tarpaulin: This is uncovered.
                    e_ij.borrow_mut().status = EdgeStatus::Boundary;
                }
            }

            if DEBUG {
                let mut boundary_edges = vec![];

                for e in front {
                    if e.borrow().status == EdgeStatus::Boundary {
                        boundary_edges.push(Triangle([
                            e.borrow().a.borrow().pos,
                            e.borrow().a.borrow().pos,
                            e.borrow().b.borrow().pos,
                        ]));
                    }
                }
                save_triangles_ascii(&PathBuf::from("boundary_edges.stl"), &boundary_edges)
                    .expect("Failed writing boundary_edges to file");
            }

            Some(triangles)
        }
    }
}
