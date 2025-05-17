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

pub(crate) mod grid;
/// Load and Save points and meshes.
pub mod io;
pub(crate) mod mesh;
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

thread_local! {
  static COUNTER3: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };

}

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
    pos: Vec3,
    normal: Vec3,
}

// This is used in testing only.
// This allows normal to default to zero which in production is
// seldom the case.
// #[cfg(test)]
// impl Point {
//     const fn new(pos: Vec3) -> Self {
//         Self { pos, normal: vec![0.0; 3] }
//     }
// }

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

            let mut seed = f.0;
            // println!("seed {}", seed[0]);
            // println!("seed {}", seed[1]);
            // println!("seed {}", seed[2]);
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

            seed[0].edges = vec![e0.clone(), e2.clone()];
            seed[1].edges = vec![e0.clone(), e1.clone()];
            seed[2].edges = vec![e1.clone(), e2.clone()];

            let mut front = vec![e0, e1, e2];
            let debug = true;
            if debug {
                save_triangles_ascii(&PathBuf::from("seed.stl"), &triangles)
                    .expect("Failed(debug) to write seed to file");
            }

            let debug = true;
            println!("initial front {} ", front.len());
            while let Some(e_ij) = get_active_edge(&mut front) {
                if let Err(e) = COUNTER3.try_with(|counter3| {
                    counter3.set(counter3.get() + 1);
                }) {
                    // Elsewhere COUNTER's destructor has been called!!!``
                    eprintln!("Access error incrementing debug counter: {e:?}");
                }

                debug_assert!((COUNTER3.get() <= 5), "counter >5 with a tetrahedral");

                println!("active edge e_ij ");
                let ae = e_ij.clone();
                println!(
                    "reconstruct: e_ij a {} {} {}",
                    ae.borrow().a.pos.x,
                    ae.borrow().a.pos.y,
                    ae.borrow().a.pos.z
                );
                println!(
                    "reconstruct: e_ij b {} {} {}",
                    ae.borrow().b.pos.x,
                    ae.borrow().b.pos.y,
                    ae.borrow().b.pos.z
                );

                if debug {
                    save_triangles_ascii(
                        &PathBuf::from("current_active_edge.stl"),
                        &[Triangle([
                            e_ij.clone().borrow().a.pos,
                            e_ij.clone().borrow().a.pos,
                            e_ij.clone().borrow().b.pos,
                        ])],
                    )
                    .expect("Failed(debug) to write front to file");
                }

                let o_k = ball_pivot(&e_ij.clone(), &mut grid, radius);
                if debug {
                    save_triangles_ascii(&PathBuf::from("current_mesh.stl"), &triangles)
                        .expect("Failed(debug) writing current mesh to file");
                }

                let mut boundary_test = false;
                if let Some(o_k) = &o_k {
                    println!(
                        "reconstruct boundary test ok = {} {} {}",
                        o_k.p.borrow().pos.x,
                        o_k.p.borrow().pos.y,
                        o_k.p.borrow().pos.z
                    );
                    let nu = not_used(&o_k.p.borrow());
                    println!("reconstruct nu = {nu:#?}");
                    // println!("reconstruct edges = {}", o_k.p.borrow().edges);
                    let ok_edges = o_k.p.borrow().edges.clone();
                    println!("reconstruct edges.len = {}", ok_edges.len());
                    for o_k_e in ok_edges {
                        println!("reconstruct edges = {:?}", o_k_e.borrow());
                    }
                    let of = on_front(&o_k.p.borrow());
                    println!("reconstruct of = {of:#?}");
                    if nu || of {
                        boundary_test = true;

                        output_triangle(
                            &MeshFace([
                                e_ij.clone().borrow().a.clone(),
                                o_k.p.borrow().clone(),
                                e_ij.clone().borrow().b.clone(),
                            ]),
                            &mut triangles,
                        );

                        let (e_ik, e_kj) = join(
                            &e_ij.clone(),
                            &mut o_k.p.borrow().clone(),
                            o_k.center,
                            &mut front,
                            &mut edges,
                        );
                        println!("checking glue");
                        if let Some(e_ki) = find_reverse_edge_on_front(&e_ik.clone()) {
                            glue(&e_ik, &e_ki, &front);
                        }

                        if let Some(e_jk) = find_reverse_edge_on_front(&e_kj.clone()) {
                            glue(&e_kj.clone(), &e_jk.clone(), &front);
                        }
                    }
                }
                if !boundary_test {
                    println!("not checking glue");
                    if debug {
                        save_points(
                            &PathBuf::from("current_boundary.ply"),
                            &vec![o_k.unwrap().p.borrow().pos],
                        )
                        .expect("could not save current boundary");
                    }
                    e_ij.borrow_mut().status = EdgeStatus::Boundary;
                }

                println!("looping front {} ", front.len());
                for f in &front {
                    println!(
                        "a {} {} {}",
                        f.borrow().a.pos.x,
                        f.borrow().a.pos.y,
                        f.borrow().a.pos.z
                    );
                    println!(
                        "b {} {} {}",
                        f.borrow().b.pos.x,
                        f.borrow().b.pos.y,
                        f.borrow().b.pos.z
                    );
                    println!();
                }
            }

            if debug {
                let mut boundary_edges = vec![];

                for e in front {
                    if e.borrow().status == EdgeStatus::Boundary {
                        boundary_edges.push(Triangle([
                            e.borrow().a.pos,
                            e.borrow().a.pos,
                            e.borrow().b.pos,
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
