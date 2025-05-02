use core::f32;
use core::panic;
use std::ops::Div;
use std::path::PathBuf;
use std::vec;

use glam::IVec3;
use glam::Vec3;
use glam::ivec3;

use crate::Cell;
use crate::io::save_points;
use crate::io::save_triangles_ascii;
use crate::mesh::EdgeStatus;
use crate::mesh::MeshEdge;
use crate::mesh::MeshFace;
use crate::mesh::MeshPoint;

use crate::Point;
use crate::Triangle;

#[derive(Clone, Debug)]
pub(crate) struct Grid {
    cell_size: f32,
    dims: IVec3,
    cells: Vec<Cell>,
    lower: Vec3,
    // upper: Vec3,
}

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
        let cells = vec![Cell::default(); (dims.x * dims.y * dims.z) as usize];

        let mut grid = Grid {
            cell_size,
            dims,
            cells,
            lower,
            // upper,
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
                    if (index.x < 0 || index.x >= self.dims.x)
                        || (index.y < 0 || index.y >= self.dims.y)
                        || (index.z < 0 || index.z >= self.dims.z)
                    {
                        continue;
                    }

                    // TODO cell_size is defined at the top, to appease the borrow checker
                    let cell_size = self.cell_size;
                    for p in self.cell(index) {
                        if (p.pos - point).length_squared() < cell_size * cell_size
                            && !ignore.contains(&p.pos)
                        {
                            result.push(p.clone());
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

fn ball_is_empty(ball_center: &Vec3, points: &[MeshPoint], radius: f32) -> bool {
    !points.iter().any(|p| {
        let length_squared = (p.pos - ball_center).length_squared();
        // TODO epsilon
        length_squared < radius * radius - 1e-4
    })
}

pub(crate) struct SeedResult {
    pub(crate) f: MeshFace,
    pub(crate) ball_center: Vec3,
}

pub(crate) fn find_seed_triangle(grid: &Grid, radius: f32) -> Option<SeedResult> {
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
                        neighborhood[i].clone(),
                        neighborhood[j].clone(),
                        neighborhood[k].clone(),
                    ]);

                    if f.normal().dot(avg_normal) < 0.0 {
                        continue;
                    }
                    let ball_center = compute_ball_center(&f, radius);
                    if let Some(ball_center) = ball_center {
                        if ball_is_empty(&ball_center, &neighborhood, radius) {
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

pub(crate) fn get_active_edge(front: &mut Vec<MeshEdge>) -> Option<MeshEdge> {
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
            // cleanup non-active edges from front
            front.pop();
        }
    }
}

pub(crate) struct PivotResult {
    pub(crate) p: MeshPoint,
    pub(crate) center: Vec3,
}

thread_local! {
  static COUNTER: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
  static COUNTER2: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
}

pub(crate) fn ball_pivot(e: &MeshEdge, grid: &mut Grid, radius: f32) -> Option<PivotResult> {
    println!("entry: ball pivot");
    let m = (e.a.pos + e.b.pos) / 2.0;
    let old_center_vec = (e.center - m).normalize();

    let neighborhood = grid.spherical_neighborhood(&m, &[e.a.pos, e.b.pos, e.opposite.pos]);

    if let Err(e) = COUNTER.try_with(|counter| {
        counter.set(counter.get() + 1);
    }) {
        // Elsewhere COUNTER's destructor has been called!!!``
        eprintln!("Access error incrementing debug counter: {:?}", e);
    };

    println!("counter {}", COUNTER.get());
    if COUNTER.get() > 5 {
        panic!("counter >10 with a tetrahedral");
    }
    let debug = true;
    if debug {
        save_triangles_ascii(
            &PathBuf::from(format!("{}_pivot_edge.stl", COUNTER.get())),
            &[Triangle([e.a.pos, e.b.pos, e.opposite.pos])],
        )
        .expect("Err - writing to pivot_edge");

        let mut points = Vec::with_capacity(neighborhood.len());
        for n in &neighborhood {
            points.push(Point::new(n.pos))
        }
        save_points(
            &PathBuf::from(format!("{}_neighborhood.ply", COUNTER.get())),
            &points,
        )
        .expect("Failed to save points");
    }

    let mut small_angle = f32::MAX;
    let mut points_with_small_angle = None;
    let mut center_of_smallest = Vec3::ZERO;
    let mut ss = String::new();

    if debug {
        println!(
            "{}.pivoting edge a={} b={} op={}. testing {} neighbors",
            COUNTER.get(),
            e.a.pos,
            e.b.pos,
            e.opposite.pos,
            neighborhood.len()
        );
    }

    let mut i = 0;
    let mut smallest_number = 0;
    println!("ball pivot about to start neighborhood loop");
    'next_neighborhood: for p in &neighborhood {
        println!("neighborhood loop");
        i += i;
        let new_face_normal = Triangle([e.b.pos, e.a.pos, p.pos]).normal();

        // this check is not in the paper: all points' normals must point into the
        // same half-space
        if p.normal.is_some_and(|n| n.dot(new_face_normal) < 0.0) {
            continue;
        }

        let c = if let Some(c) =
            compute_ball_center(&MeshFace([e.b.clone(), e.a.clone(), p.clone()]), radius)
        {
            c
        } else {
            if debug {
                ss.push_str(&format!("{i}.     {:?} center computation failed\n", p.pos));
            }
            continue;
        };

        if debug {
            if let Err(e) = COUNTER2.try_with(|counter2| {
                counter2.set(counter2.get() + 1);
            }) {
                // Elsewhere COUNTER2's destructor has been called!!!``
                eprintln!("Access error incrementing debug counter: {:?}", e);
            }
            save_triangles_ascii(
                &PathBuf::from(format!("{}_{}_face.stl", COUNTER.get(), COUNTER2.get())),
                &[Triangle([e.a.pos, e.b.pos, p.pos])],
            )
            .expect("Failed(debug) to write face to file");
            save_points(
                &PathBuf::from(format!(
                    "{}_{}_ball_center.ply",
                    COUNTER.get(),
                    COUNTER2.get()
                )),
                &vec![Point::new(c)],
            )
            .expect("Failed(debug) to write ball_center file");
        }

        // this check is not in the paper: the ball center must always be above the
        // triangle
        let new_center_vec = (c - m).normalize();
        let new_center_face_dot = (new_center_vec).dot(new_face_normal);
        if new_center_face_dot < 0_f32 {
            if debug {
                // ss << i << ".    " << p->pos << " ball center " << c.value() << " underneath triangle\n";
                ss.push_str(&format!(
                    "{i}.    {:?} ball center {:?} underneath triangle\n",
                    p.pos, c
                ));
            }
            continue;
        }
        // this check is not in the paper: points to which we already have an inner
        // edge are not considered
        // for (const auto* ee : p->edges) {

        for ee in &p.edges {
            // const auto* otherPoint = ee->a == p ? ee->b : ee->a;
            let other_point = if ee.a == p.clone() { &ee.b } else { &ee.a };
            if ee.status == EdgeStatus::Inner && *other_point == e.a || *other_point == e.b {
                if debug {
                    ss.push_str(&format!("{i}.    {:?} inner edge exists\n", p.pos));
                }
                // This was a GOTO into the original c++ source.
                println!("following goto");
                continue 'next_neighborhood;
            }
        }

        {
            let mut angle = (old_center_vec).dot(new_center_vec).clamp(-1.0, 1.0).acos();
            if new_center_vec.cross(old_center_vec).dot(e.a.pos - e.b.pos) < 0.0_f32 {
                angle += std::f32::consts::PI;
            }
            if angle < small_angle {
                small_angle = angle;
                points_with_small_angle = Some(p.clone());
                center_of_smallest = c;
                smallest_number = i;
            }

            if debug {
                ss.push_str(&format!(
                    "{i}.   {}  center {:?}  angle {:?} next center face dot {}\n",
                    p.pos, c, angle, new_center_face_dot
                ));
            }
        }

        if small_angle != f32::MAX {
            if ball_is_empty(&center_of_smallest, &neighborhood, radius) {
                if debug {
                    ss.push_str(&format!("       picking point {smallest_number}\n"));
                    match &points_with_small_angle {
                        Some(candidate_point) => {
                            save_points(
                                &PathBuf::from(format!("{}_candidate.ply", COUNTER.get())),
                                &vec![Point::new(candidate_point.pos)],
                            )
                            .expect("Failed(debug) to write ball_center file");
                        }
                        None => {
                            eprintln!(
                                "debug: trying to display a candidate point which doe not exist"
                            );
                        }
                    }
                }

                return Some(PivotResult {
                    p: points_with_small_angle.unwrap(),
                    center: center_of_smallest,
                });
            } else if debug {
                ss.push_str(&format!(
                    "found candidate {smallest_number} but bail is not empty \n"
                ));
            }
        }
    }
    None
}

pub(crate) const fn not_used(p: &MeshPoint) -> bool {
    !p.used
}

pub(crate) fn on_front(p: &MeshPoint) -> bool {
    p.edges.iter().any(|e| e.status == EdgeStatus::Active)
}

// Removed edge from consideration
const fn remove(e: &mut MeshEdge) {
    e.status = EdgeStatus::Inner;
}

pub(crate) fn output_triangle(f: &MeshFace, triangles: &mut Vec<Triangle>) {
    triangles.push(Triangle([f.0[0].pos, f.0[1].pos, f.0[2].pos]));
}

pub(crate) fn join(
    e_ij: &mut MeshEdge,
    o_k: &mut MeshPoint,
    o_k_ball_center: Vec3,
    front: &mut Vec<MeshEdge>,
    edges: &mut Vec<MeshEdge>,
) -> (MeshEdge, MeshEdge) {
    // auto& e_ik = edges.emplace_back(MeshEdge{e_ij->a, o_k, e_ij->b, o_k_ballCenter});
    let mut e_ik = MeshEdge::new(&e_ij.a, o_k, &e_ij.b.clone(), o_k_ball_center);
    edges.push(e_ik.clone());
    let mut e_kj = MeshEdge::new(o_k, &e_ij.b, &e_ij.a.clone(), o_k_ball_center);
    edges.push(e_kj.clone());

    // e_ik
    e_ik.next = Some(Box::new(e_kj.clone()));
    e_ik.prev = e_ij.prev.clone();
    match &mut e_ij.prev {
        Some(prev) => prev.next = Some(Box::new(e_ik.clone())),
        None => panic!("e_ij.prev is None"),
    }
    e_ij.a.edges.push(e_ik.clone());

    // e_kj
    e_kj.prev = Some(Box::new(e_ik.clone()));
    e_kj.next = e_ij.next.clone();
    match &mut e_ij.next {
        Some(next) => next.prev = Some(Box::new(e_kj.clone())),
        None => panic!("e_ij.prev is None"),
    }
    e_ij.b.edges.push(e_kj.clone());

    o_k.used = true;
    o_k.edges.push(e_ik.clone());
    o_k.edges.push(e_kj.clone());

    front.push(e_ik.clone());
    front.push(e_kj.clone());
    remove(e_ij);

    (e_ik, e_kj)
}

pub(crate) fn glue(a: &mut MeshEdge, b: &mut MeshEdge, front: &[MeshEdge]) {
    // TODO replace this boolean with a proper check
    let debug = true;
    if debug {
        let mut front_triangles = vec![];
        for e in front {
            if e.status == EdgeStatus::Active {
                // This looks buggy the cpp version repeats e.a.pos.
                // So a line not a triangle.
                front_triangles.push(Triangle([e.a.pos, e.a.pos, e.b.pos]));
            }
            save_triangles_ascii(&PathBuf::from("glue_front.stl"), &front_triangles)
                .expect("Err debug failing writing glue_front.stl");
            save_triangles_ascii(
                &PathBuf::from("glue_edges.stl"),
                &[Triangle([a.a.pos, a.a.pos, a.b.pos])],
            )
            .expect("Err debug failing writing glue_edge.stl");
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

pub(crate) fn find_reverse_edge_on_front(edge: &MeshEdge) -> Option<MeshEdge> {
    for e in &edge.a.edges {
        if e.a == edge.a {
            return Some(e.clone());
        }
    }
    None
}
