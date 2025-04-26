use glam::IVec3;
use glam::Vec3;
use glam::ivec3;

use crate::Cell;
use crate::mesh::EdgeStatus;
use crate::mesh::MeshEdge;
use crate::mesh::MeshFace;
use crate::mesh::MeshPoint;
use crate::save_triangles;

use crate::Point;
use crate::Triangle;

#[derive(Clone, Debug)]
struct Grid {
    cell_size: f32,
    dims: IVec3,
    cells: Vec<Cell>,
    lower: Vec3,
    upper: Vec3,
}

use core::panic;
use std::collections::VecDeque;
use std::ops::Div;
use std::path::PathBuf;

impl Grid {
    pub fn new(points: &[Point], radius: f32) -> Self {
        let cell_size = 2_f32 * radius;
        let mut lower = points.first().expect("Vec with no points").pos;
        let mut upper = points.first().expect("Vec with no points(2)").pos;
        for p in points {
            for i in 0..3 {
                lower[i] = lower[i].min(p.pos[i]);
                upper[i] = upper[i].max(p.pos[i]);
            }
        }

        let ceil_float = (upper - lower).ceil().div(cell_size);
        let candidate_dim: IVec3 = ivec3(
            ceil_float[0] as i32,
            ceil_float[1] as i32,
            ceil_float[2] as i32,
        );
        let dims = candidate_dim.max(ivec3(1, 1, 1));
        let cells = Vec::with_capacity((dims.x * dims.y * dims.z) as usize);

        let mut grid = Grid {
            cell_size,
            dims,
            cells,
            lower,
            upper,
        };

        for p in points {
            grid.cell(grid.cell_index(&p.pos)).push(MeshPoint::from(p));
        }

        grid
    }

    fn cell_index(&self, point: &Vec3) -> IVec3 {
        let diff = (point - self.lower) / self.cell_size;
        let index = ivec3(diff.x as i32, diff.y as i32, diff.z as i32);
        index.clamp(ivec3(0, 0, 0), self.dims - 1)
    }

    fn cell(&mut self, index: IVec3) -> &mut Cell {
        let index = index.z * self.dims.x * self.dims.y + index.y * self.dims.x + index.x;
        &mut self.cells[index as usize]
    }

    fn spherical_neighborhood(&mut self, point: &Vec3, ignore: &[Vec3]) -> Vec<MeshPoint> {
        let center_index = self.cell_index(point);
        // Just an estimate.
        let capacity = self.cell(center_index).len() * 27;
        let mut result = Vec::with_capacity(capacity);
        for x_off in -1..=1 {
            for y_off in -1..=1 {
                for z_off in -1..=1 {
                    let index = center_index + ivec3(x_off, y_off, z_off);
                    if index.x < 0
                        || index.x >= self.dims.x
                        || index.y < 0
                        || index.y >= self.dims.y
                        || index.z < 0
                        || index.z >= self.dims.z
                    {
                        continue;
                    }

                    // TODO cell_size is defined at the top, to appease the borrow checker
                    // is this a breaking change from the C++ code?
                    let cell_size = self.cell_size;
                    for p in self.cell(index) {
                        let len = (p.pos - point).length_squared();
                        if len < cell_size * cell_size {
                            let find = ignore.iter().find(|&x| *x == p.pos);
                            if find.is_none() {
                                result.push(p.clone());
                            }
                        }
                    }
                }
            }
        }
        result
    }
}

// from
// https://gamedev.stackexchange.com/questions/60630/how-do-i-find-the-circumcenter-of-a-triangle-in-3d
pub(crate) fn compute_ball_center(f: &MeshFace, radius: f32) -> Option<Vec3> {
    let ac = f.0[2].pos - f.0[0].pos;
    let ab = f.0[1].pos - f.0[0].pos;
    let ab_cross_ac = ab.cross(ac);

    let to_circum_circle_center = (ab_cross_ac.cross(ab) * ac.dot(ac)
        + ac.cross(ab_cross_ac) * ab.dot(ab))
        / (2.0 * ab_cross_ac.dot(ab_cross_ac));

    let circum_circle_center = f.0[0].pos + to_circum_circle_center;

    let height_squared = radius * radius - to_circum_circle_center.dot(to_circum_circle_center);
    if height_squared.is_sign_negative() {
        return None;
    }

    Some(circum_circle_center + f.normal() * height_squared.sqrt())
}

fn is_ball_empty(ball_center: &Vec3, points: &[MeshPoint], radius: f32) -> bool {
    !points.iter().any(|p| {
        let length_squared = (p.pos - ball_center).length_squared();
        // TODO epsilon
        length_squared < radius * radius - 1e-4
    })
}

struct SeedResult {
    f: MeshFace,
    ball_center: Vec3,
}

fn find_seed_triangle(grid: &mut Grid, radius: f32) -> Option<SeedResult> {
    for c_i in 0..grid.cells.len() {
        let avg_normal = grid.cells[c_i]
            .iter()
            .fold(Vec3::new(0.0, 0.0, 0.0), |acc, p| match p.normal {
                Some(n) => acc + n,
                None => acc,
            });

        for i in 0..grid.cells[c_i].len() {
            let mut neighborhood = grid
                .clone()
                .spherical_neighborhood(&grid.cells[c_i][i].pos, &[grid.cells[c_i][i].pos]);

            neighborhood.sort_by(|a, b| {
                if (a.pos - grid.cells[c_i][i].pos).length_squared()
                    < (b.pos - grid.cells[c_i][i].pos).length_squared()
                {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            });

            for j in 0..neighborhood.len() {
                for k in 0..neighborhood.len() {
                    if neighborhood[j] == neighborhood[k] {
                        continue;
                    }

                    // only accept triangles which's normal points into the same
                    // half-space as the average normal of this cell's points
                    let f = MeshFace([
                        grid.cells[c_i][i].clone(),
                        neighborhood[j].clone(),
                        neighborhood[k].clone(),
                    ]);

                    if f.normal().dot(avg_normal) < 0.0 {
                        continue;
                    }
                    let ball_center = compute_ball_center(&f, radius);
                    if let Some(ball_center) = ball_center {
                        if is_ball_empty(&ball_center, &neighborhood, radius) {
                            grid.cells[c_i][1].used = true;
                            (neighborhood[i]).used = true;
                            (neighborhood[j]).used = true;
                            (neighborhood[k]).used = true;
                            return Some(SeedResult { f, ball_center });
                        }
                    }
                }
            }
        }
    }
    None
}

fn get_active_edge(front: &mut Vec<MeshEdge>) -> Option<MeshEdge> {
    loop {
        {
            match front.last() {
                None => {
                    // exit loop
                    return None;
                }
                Some(e) => {
                    if e.status == EdgeStatus::Active {
                        return Some(e.clone());
                    }
                }
            }
        }

        front.pop();
    }
}

struct PivotResult {
    p: MeshPoint,
    ball_center: Vec3,
}

fn ball_pivot(e: &MeshEdge, grid: &mut Grid, radius: f32) -> Option<PivotResult> {
    todo!();
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

fn output_triangle(f: &MeshFace, triangles: &mut Vec<Triangle>) {
    triangles.push(Triangle([f.0[0].pos, f.0[1].pos, f.0[2].pos]));
}

fn join(
    e_ij: &mut MeshEdge,
    o_k: &mut MeshPoint,
    o_k_ball_center: Vec3,
    front: &mut [MeshEdge],
    edges: &VecDeque<MeshEdge>,
) -> (MeshEdge, MeshEdge) {
    // auto& e_ik = edges.emplace_back(MeshEdge{e_ij->a, o_k, e_ij->b, o_k_ballCenter});
    let mut e_ik = MeshEdge::new(&e_ij.a, &o_k, e_ij.b.clone(), o_k_ball_center);
    let mut e_kj = MeshEdge::new(&o_k, &e_ij.b, e_ij.a.clone(), o_k_ball_center);

    // e_ik
    e_ik.next = Some(Box::new(e_kj.clone()));
    e_ik.prev = e_ik.prev;
    match &mut e_ij.prev {
        Some(prev) => prev.next = Some(Box::new(e_ik.clone())),
        None => panic!("e_ij.prev is None"),
    }
    match &mut e_ij.a {
        MeshPoint { edges, .. } => edges.push(e_ik.clone()),
        _ => panic!("e_ij.a.edges is None"),
    }

    // e_kj
    e_kj.prev = Some(Box::new(e_ik.clone()));
    e_kj.next = e_ij.next.clone();
    match &mut e_ij.next {
        Some(next) => next.prev = Some(Box::new(e_ik.clone())),
        None => panic!("e_ij.prev is None"),
    }
    match &mut e_ij.b {
        MeshPoint { edges, .. } => edges.push(e_kj.clone()),
        _ => panic!("e_ij.a.edges is None"),
    }

    o_k.used = true;
    o_k.edges.push(e_ik.clone());

    todo!();
}

fn glue<'a>(a: &'a mut MeshEdge, b: &'a mut MeshEdge, front: &mut [MeshEdge]) {
    // TODO replace this boolean with a proper check
    let debug = true;
    if debug {
        let mut front_triangles = vec![];
        for e in front.iter() {
            if e.status == EdgeStatus::Active {
                // This looks buggy the cpp version repeats e.a.pos.
                // So a line not a triangle.
                front_triangles.push(Triangle([e.a.pos, e.a.pos, e.b.pos]));
            }
            save_triangles(&PathBuf::from("glue_front.stl"), &front_triangles);
            save_triangles(
                &PathBuf::from("glue_edges.stl"),
                &[Triangle([a.a.pos, a.a.pos, a.b.pos])],
            );
        }
    }
    // case 1
    if let (Some(a_prev), Some(b_next)) = (a.prev.clone(), b.next.clone()) {
        if a_prev.as_ref() == b && b_next.as_ref() == a {
            a.next = b.next.clone();
            b.prev = a.prev.clone();
            remove(a);
            remove(b);
            return;
        }
    }

    // case 2
    if let (Some(a_next), Some(b_prev)) = (&a.next, &b.prev) {
        if a_next.as_ref() == b && b_prev.as_ref() == a {
            a.prev = b.prev.clone();
            b.next = a.next.clone();
            remove(a);
            remove(b);
            return;
        }
    }

    if let (Some(a_prev), Some(b_next)) = (&a.prev, &b.next) {
        if a_prev.as_ref() == b && b_next.as_ref() == a {
            a.next = b.next.clone();
            b.prev = a.prev.clone();
            remove(a);
            remove(b);
            return;
        }
    }

    // case 3/4
    // a->prev->next = b->next;
    if let Some(a_prev) = &mut a.prev {
        a_prev.next = b.next.clone();
    }
    // b->next->prev = a->prev;
    if let Some(b_next) = &mut b.next {
        b_next.prev = a.prev.clone();
    }
    // a->next->prev = b->prev;
    if let Some(a_next) = &mut a.next {
        a_next.prev = b.prev.clone();
    }
    // b->prev->next = a->next;
    if let Some(b_prev) = &mut b.prev {
        b_prev.next = a.next.clone();
    }
    remove(a);
    remove(b);
}

fn find_reverse_edge_on_front(edge: &MeshEdge) -> Option<MeshEdge> {
    for e in &edge.a.edges {
        if e.a == edge.a {
            return Some(e.clone());
        }
    }
    None
}

pub(crate) fn reconstruct(points: &[Point], radius: f32) -> Option<Vec<Triangle>> {
    let mut grid = Grid::new(points, radius);

    match find_seed_triangle(&mut grid, radius) {
        None => {
            eprintln!("No seed triangle found");
            None
        }
        Some(SeedResult { f, ball_center }) => {
            let mut triangles: Vec<Triangle> = Vec::new();
            let edges: VecDeque<MeshEdge> = VecDeque::new();
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

            let mut front = vec![e0, e1, e2];
            let debug = true;
            if debug {
                save_triangles(&PathBuf::from("seed.stl"), &triangles);
            }

            let debug = true;
            loop {
                let e_ij = get_active_edge(&mut front);
                if e_ij.is_none() {
                    break;
                }

                if debug {
                    save_triangles(&PathBuf::from("front.stl"), &triangles);
                }

                let o_k = ball_pivot(&e_ij.clone().unwrap(), &mut grid, radius);
                if debug {
                    save_triangles(&PathBuf::from("current_mesh.stl"), &triangles);
                }

                let mut boundary_test = false;
                if let Some(o_k) = o_k {
                    if not_used(&o_k.p) || on_front(&o_k.p) {
                        boundary_test = true;

                        output_triangle(
                            &MeshFace([e_ij.clone().unwrap().a, o_k.p, e_ij.unwrap().b]),
                            &mut triangles,
                        );
                    }
                }
                if !boundary_test {
                    if debug {
                        // save_points(&PathBuf::from("current_boundary.ply"), &vec![Point::new(o_k.clone().unwrap().p.pos)]);
                        todo!();
                    }
                    todo!();
                    // e_ij.unwrap().status = EdgeStatus::Boundary;
                }
            }

            if debug {
                let mut boundary_edges = vec![];

                for e in front.iter() {
                    if e.status == EdgeStatus::Boundary {
                        boundary_edges.push(Triangle([e.a.pos, e.a.pos, e.b.pos]));
                    }
                }
                save_triangles(&PathBuf::from("boundary_edges.stl"), &boundary_edges);
            }
            Some(triangles)
        }
    }
}
