use core::cell::RefCell;
use core::f32;
use core::panic;
use std::fmt::Write;
use std::ops::Div;
use std::path::PathBuf;
use std::rc::Rc;
use std::vec;

use glam::IVec3;
use glam::Vec3;
use glam::ivec3;

use crate::Cell;
use crate::DEBUG;
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

        let mut grid = Self {
            cell_size,
            dims,
            cells,
            lower,
            // upper,
        };

        for p in points {
            let actual_cell = grid.cell(grid.cell_index(&p.pos));
            actual_cell.push(Rc::new(RefCell::new(MeshPoint::from(p))));
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

    fn spherical_neighborhood(
        &mut self,
        point: &Vec3,
        ignore: &[Vec3],
    ) -> Vec<Rc<RefCell<MeshPoint>>> {
        let center_index = self.cell_index(point);
        // Just an estimate.
        let capacity = self.cell(center_index).len() * 27;
        let mut result = Vec::with_capacity(capacity);
        for x_off in [-1, 0, 1] {
            for y_off in [-1, 0, 1] {
                for z_off in [-1, 0, 1] {
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
                        let p_pos = p.borrow().pos;
                        if (p_pos - point).length_squared() < cell_size * cell_size
                            && !ignore.contains(&p_pos)
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

/// Computes the circumcenter of a triangle in 3D space.
///
/// The circumcenter is the center of the circle that passes through all three
/// vertices of the triangle.
///
/// from
/// <https://gamedev.stackexchange.com/questions/60630/how-do-i-find-the-circumcenter-of-a-triangle-in-3d>
#[must_use]
pub fn compute_ball_center(f: &MeshFace, radius: f32) -> Option<Vec3> {
    let ac = f.0[2].borrow().pos - f.0[0].borrow().pos;
    let ab = f.0[1].borrow().pos - f.0[0].borrow().pos;
    let ab_cross_ac = ab.cross(ac);

    let to_circum_circle_center = (ab_cross_ac.cross(ab) * ac.dot(ac)
        + ac.cross(ab_cross_ac) * ab.dot(ab))
        / (2.0 * ab_cross_ac.dot(ab_cross_ac));

    let circum_circle_center = f.0[0].borrow().pos + to_circum_circle_center;

    let height_squared = radius.mul_add(
        radius,
        -to_circum_circle_center.dot(to_circum_circle_center),
    );
    if height_squared.is_sign_negative() {
        return None;
    }

    Some(circum_circle_center + f.normal() * height_squared.sqrt())
}

fn ball_is_empty(ball_center: &Vec3, points: &[Rc<RefCell<MeshPoint>>], radius: f32) -> bool {
    let threshold = radius.mul_add(radius, -1e-4);
    !points.iter().any(|p| {
        let length_squared = (p.borrow().pos - ball_center).length_squared();
        // TODO epsilon
        length_squared < threshold
    })
}

pub(crate) struct SeedResult {
    pub(crate) f: MeshFace,
    pub(crate) ball_center: Vec3,
}

pub(crate) fn find_seed_triangle(grid: &Grid, radius: f32) -> Option<SeedResult> {
    for cell in &grid.cells {
        let avg_normal = cell
            .iter()
            .fold(Vec3::new(0.0, 0.0, 0.0), |acc, p| acc + p.borrow().normal)
            .normalize();

        for p1 in cell {
            let mut neighborhood = grid
                .clone()
                .spherical_neighborhood(&p1.borrow().pos, &[p1.borrow().pos]);

            neighborhood.sort_by(|a, b| {
                if (a.borrow().pos - p1.borrow().pos).length_squared()
                    < (b.borrow().pos - p1.borrow().pos).length_squared()
                {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            });

            for p2 in neighborhood.clone() {
                for p3 in &neighborhood {
                    if p2.as_ptr() == p3.as_ptr() {
                        continue;
                    }

                    // only accept triangles which's normal points into the same
                    // half-space as the average normal of this cell's points
                    let f = MeshFace([p1.clone(), p2.clone(), p3.clone()]);

                    if f.normal().dot(avg_normal) < 0.0 {
                        continue;
                    }
                    let ball_center = compute_ball_center(&f, radius);
                    if let Some(ball_center) = ball_center {
                        if ball_is_empty(&ball_center, &neighborhood, radius) {
                            p1.borrow_mut().used = true;
                            p2.borrow_mut().used = true;
                            p3.borrow_mut().used = true;
                            return Some(SeedResult { f, ball_center });
                        }
                    }
                }
            }
        }
    }
    None
}

pub(crate) fn get_active_edge(
    front: &mut Vec<Rc<RefCell<MeshEdge>>>,
) -> Option<Rc<RefCell<MeshEdge>>> {
    loop {
        {
            match front.last() {
                None => {
                    // exit loop
                    return None;
                }
                Some(e) => {
                    if e.borrow().status == EdgeStatus::Active {
                        return Some(e.clone());
                    }
                }
            }
            // cleanup non-active edges from front
            front.pop();
        }
    }
}

#[derive(Debug)]
pub(crate) struct PivotResult {
    pub(crate) p: Rc<RefCell<MeshPoint>>,
    pub(crate) center: Vec3,
}

thread_local! {
  static COUNTER: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
  static COUNTER2: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
}

pub(crate) fn ball_pivot(
    e: &Rc<RefCell<MeshEdge>>,
    grid: &mut Grid,
    radius: f32,
) -> Option<PivotResult> {
    let m = (e.borrow().a.borrow().pos + e.borrow().b.borrow().pos) / 2.0;
    let old_center_vec = (e.borrow().center - m).normalize();

    let neighborhood = grid.spherical_neighborhood(
        &m,
        &[
            e.borrow().a.borrow().pos,
            e.borrow().b.borrow().pos,
            e.borrow().opposite.borrow().pos,
        ],
    );

    if let Err(e) = COUNTER.try_with(|counter| {
        counter.set(counter.get() + 1);
    }) {
        // Elsewhere COUNTER's destructor has been called!!!``
        eprintln!("Access error incrementing debug counter: {e:?}");
    }

    if DEBUG {
        save_triangles_ascii(
            &PathBuf::from(format!("{}_pivot_edge.stl", COUNTER.get())),
            &[Triangle([
                e.borrow().a.borrow().pos,
                e.borrow().a.borrow().pos,
                e.borrow().b.borrow().pos,
            ])],
        )
        .expect("Err - writing to pivot_edge");

        let mut points: Vec<Vec3> = Vec::with_capacity(neighborhood.len());
        for n in &neighborhood {
            points.push(n.borrow().pos);
        }
        save_points(
            &PathBuf::from(format!("{}_neighborhood.ply", COUNTER.get())),
            &points,
        )
        .expect("Failed to save points");
    }

    let mut smallest_angle = f32::MAX;
    let mut point_with_smallest_angle = None;
    let mut center_of_smallest = Vec3::ZERO;
    let mut ss = String::new();

    if DEBUG {
      let mut ss = String::new();
    }

    if DEBUG {
        let mut ss = String::new();
        writeln!(
            ss,
            "{}.pivoting edge a={} b={} op={}. testing {} neighbors",
            COUNTER.get(),
            e.borrow().a.borrow().pos,
            e.borrow().b.borrow().pos,
            e.borrow().opposite.borrow().pos,
            neighborhood.len()
        )
        .expect("could not write debug");
    }

    let mut i = 0;
    let mut smallest_number = 0;
    'next_neighborhood: for p in &neighborhood {
        i += 1;
        let new_face_normal = Triangle([
            e.borrow().b.borrow().pos,
            e.borrow().a.borrow().pos,
            p.borrow().pos,
        ])
        .normal();

        // this check is not in the paper: all points' normals must point into the
        // same half-space
        if new_face_normal.dot(p.borrow().normal) < 0.0 {
            continue;
        }

        let Some(c) = compute_ball_center(
            &MeshFace([e.borrow().b.clone(), e.borrow().a.clone(), p.clone()]),
            radius,
        ) else {
            if DEBUG {
                writeln!(
                    &mut ss,
                    "{i}.     {:?} center computation failed",
                    p.borrow().pos
                )
                .expect("could not write debug");
            }
            continue;
        };

        if DEBUG {
            if let Err(e) = COUNTER2.try_with(|counter2| {
                counter2.set(counter2.get() + 1);
            }) {
                // Elsewhere COUNTER2's destructor has been called!!!``
                eprintln!("Access error incrementing debug counter: {e:?}");
            }
            save_triangles_ascii(
                &PathBuf::from(format!("{}_{}_face.stl", COUNTER.get(), COUNTER2.get())),
                &[Triangle([
                    e.borrow().a.borrow().pos,
                    e.borrow().b.borrow().pos,
                    p.borrow().pos,
                ])],
            )
            .expect("Failed(debug) to write face to file");
            save_points(
                &PathBuf::from(format!(
                    "{}_{}_ball_center.ply",
                    COUNTER.get(),
                    COUNTER2.get()
                )),
                &vec![c],
            )
            .expect("Failed(debug) to write ball_center file");
        }

        // this check is not in the paper: the ball center must always be above the
        // triangle
        let new_center_vec = (c - m).normalize();
        let new_center_face_dot = (new_center_vec).dot(new_face_normal);
        if new_center_face_dot < 0_f32 {
            if DEBUG {
                writeln!(
                    &mut ss,
                    "{i}.    {:?} ball center {c:?} underneath triangle",
                    p.borrow().pos
                )
                .expect("could not write debug");
            }
            continue;
        }
        // this check is not in the paper: points to which we already have an inner
        // edge are not considered
        for ee in &p.borrow().edges {
            // const auto* otherPoint = ee->a == p ? ee->b : ee->a;
            let other_point = if ee.borrow().a.as_ptr() == p.as_ptr() {
                &ee.borrow().b
            } else {
                &ee.borrow().a
            };
            if ee.borrow().status == EdgeStatus::Inner
                && (other_point.as_ptr() == e.borrow().a.as_ptr()
                    || other_point.as_ptr() == e.borrow().b.as_ptr())
            {
                if DEBUG {
                    writeln!(&mut ss, "{i}.    {:?} inner edge exists", p.borrow().pos)
                        .expect("could to write debug");
                }
                // This was a GOTO into the original c++ source.
                continue 'next_neighborhood;
            }
        }

        let mut angle = (old_center_vec).dot(new_center_vec).clamp(-1.0, 1.0).acos();
        if new_center_vec
            .cross(old_center_vec)
            .dot(e.borrow().a.borrow().pos - e.borrow().b.borrow().pos)
            < 0.0_f32
        {
            angle += std::f32::consts::PI;
        }
        if angle < smallest_angle {
            if DEBUG {
              writeln!(&mut ss, "ball pivot angle < smallest angle").expect("could not write debug");
            }
            smallest_angle = angle;
            point_with_smallest_angle = Some(p.clone());
            center_of_smallest = c;
            smallest_number = i;
        }

        if DEBUG {
            writeln!(
                    &mut ss,
                    "{i}.   {}  center {c:?} angle {angle:?} next center face dot {new_center_face_dot}",
                    p.borrow().pos,
                )
                .expect("Failed to output debug");
        }
    }

    if smallest_angle != f32::MAX {
        if ball_is_empty(&center_of_smallest, &neighborhood, radius) {
            if DEBUG {
                writeln!(&mut ss, "       picking point {smallest_number}")
                    .expect("Could not render debug");
                match &point_with_smallest_angle {
                    Some(candidate_point) => {
                        save_points(
                            &PathBuf::from(format!("{}_candidate.ply", COUNTER.get())),
                            &vec![candidate_point.borrow().pos],
                        )
                        .expect("Failed(debug) to write ball_center file");
                    }
                    None => {
                        eprintln!("debug: trying to display a candidate point which doe not exist");
                    }
                }
                println!("{ss}");
            }

            return Some(PivotResult {
                p: point_with_smallest_angle.unwrap(),
                center: center_of_smallest,
            });
        } else if DEBUG {
            writeln!(
                &mut ss,
                "        found candidate {smallest_number} but bail int not empty",
            )
            .expect("failed writing debug");
        }
    }

    if DEBUG {
        println!("{ss}");
    }

    None
}

pub(crate) const fn not_used(p: &MeshPoint) -> bool {
    !p.used
}

pub(crate) fn on_front(p: &MeshPoint) -> bool {
    p.edges
        .iter()
        .any(|e| e.borrow().status == EdgeStatus::Active)
}

// Removed edge from consideration
fn remove(e: &Rc<RefCell<MeshEdge>>) {
    e.borrow_mut().status = EdgeStatus::Inner;
}

pub(crate) fn output_triangle(f: &MeshFace, triangles: &mut Vec<Triangle>) {
    triangles.push(Triangle([
        f.0[0].borrow().pos,
        f.0[1].borrow().pos,
        f.0[2].borrow().pos,
    ]));
}

pub(crate) fn join(
    e_ij: &Rc<RefCell<MeshEdge>>,
    o_k: &Rc<RefCell<MeshPoint>>,
    o_k_ball_center: Vec3,
    front: &mut Vec<Rc<RefCell<MeshEdge>>>,
    edges: &mut Vec<Rc<RefCell<MeshEdge>>>,
) -> (Rc<RefCell<MeshEdge>>, Rc<RefCell<MeshEdge>>) {
    let e_ik = Rc::new(RefCell::new(MeshEdge::new(
        &e_ij.borrow().a,
        o_k,
        &e_ij.borrow().b.clone(),
        o_k_ball_center,
    )));
    edges.push(e_ik.clone());
    let e_kj = Rc::new(RefCell::new(MeshEdge::new(
        o_k,
        &e_ij.borrow().b,
        &e_ij.borrow().a.clone(),
        o_k_ball_center,
    )));
    edges.push(e_kj.clone());

    // e_ik
    e_ik.borrow_mut().next = Some(e_kj.clone());
    e_ik.borrow_mut().prev.clone_from(&e_ij.borrow().prev);
    match &e_ij.borrow().prev {
        Some(prev) => prev.borrow_mut().next = Some(e_ik.clone()),
        None => panic!("e_ij.prev Must be defined at this point"),
    }
    e_ij.borrow().a.borrow_mut().edges.push(e_ik.clone());

    // e_kj
    e_kj.borrow_mut().prev = Some(e_ik.clone());
    e_kj.borrow_mut().next.clone_from(&e_ij.borrow().next);
    match &mut e_ij.borrow().next.clone() {
        Some(next) => next.borrow_mut().prev = Some(e_kj.clone()),
        None => panic!("e_ij.prev is None"),
    }
    e_ij.borrow().b.borrow_mut().edges.push(e_kj.clone());

    let mut o_k_inner = o_k.borrow_mut();
    o_k_inner.used = true;
    o_k_inner.edges.push(e_ik.clone());
    o_k_inner.edges.push(e_kj.clone());

    front.push(e_ik.clone());
    front.push(e_kj.clone());
    remove(e_ij);

    (e_ik, e_kj)
}

pub(crate) fn glue(
    a: &Rc<RefCell<MeshEdge>>,
    b: &Rc<RefCell<MeshEdge>>,
    front: &[Rc<RefCell<MeshEdge>>],
) {
    if DEBUG {
        let mut front_triangles = vec![];
        for e in front {
            if e.borrow().status == EdgeStatus::Active {
                // This looks buggy the cpp version repeats e.a.pos.
                // So a line not a triangle.
                front_triangles.push(Triangle([
                    e.borrow().a.borrow().pos,
                    e.borrow().a.borrow().pos,
                    e.borrow().b.borrow().pos,
                ]));
            }
            save_triangles_ascii(&PathBuf::from("glue_front.stl"), &front_triangles)
                .expect("Err debug failing writing glue_front.stl");
            save_triangles_ascii(
                &PathBuf::from("glue_edges.stl"),
                &[Triangle([
                    a.borrow().a.borrow().pos,
                    a.borrow().a.borrow().pos,
                    a.borrow().b.borrow().pos,
                ])],
            )
            .expect("Err debug failing writing glue_edge.stl");
        }
    }
    // case 1
    if a.borrow().next.clone().unwrap().as_ptr() == b.as_ptr()
        && a.borrow().prev.clone().unwrap().as_ptr() == b.as_ptr()
        && b.borrow().next.clone().unwrap().as_ptr() == a.as_ptr()
        && b.borrow().prev.clone().unwrap().as_ptr() == a.as_ptr()
    {
        remove(&a.clone());
        remove(&b.clone());
        return;
    }

    // case 2
    if a.borrow().next.clone().unwrap().as_ptr() == b.as_ptr()
        && b.borrow().prev.clone().unwrap().as_ptr() == a.as_ptr()
    {
        a.clone()
            .borrow()
            .prev
            .as_ref()
            .unwrap()
            .borrow_mut()
            .next
            .clone_from(&b.borrow().next);
        b.clone()
            .borrow()
            .next
            .as_ref()
            .unwrap()
            .borrow_mut()
            .prev
            .clone_from(&a.borrow().prev);
        remove(&a.clone());
        remove(&b.clone());
        return;
        // }
    }

    if a.borrow().prev.clone().unwrap().as_ptr() == b.as_ptr()
        && b.borrow().next.clone().unwrap().as_ptr() == a.as_ptr()
    {
        a.clone().borrow_mut().next.clone_from(&b.borrow().next);
        b.clone().borrow_mut().prev.clone_from(&a.borrow().prev);
        remove(&a.clone());
        remove(&b.clone());
        return;
    }

    // case 3/4
    if let Some(a_prev) = &mut a.borrow().prev.clone() {
        a_prev.borrow_mut().next.clone_from(&b.borrow().next);
    }

    if let Some(b_next) = &mut b.borrow().next.clone() {
        b_next.borrow_mut().prev.clone_from(&a.borrow().prev);
    }

    if let Some(a_next) = &mut a.borrow().next.clone() {
        a_next.borrow_mut().prev.clone_from(&b.borrow().prev);
    }

    if let Some(b_prev) = &mut b.borrow().prev.clone() {
        b_prev.borrow_mut().next.clone_from(&a.borrow().next);
    }
    remove(a);
    remove(b);
}

pub(crate) fn find_reverse_edge_on_front(
    edge: &Rc<RefCell<MeshEdge>>,
) -> Option<Rc<RefCell<MeshEdge>>> {
    for e in &edge.borrow().a.borrow().edges {
        if e.borrow().a.as_ptr() == edge.borrow().b.as_ptr() {
            return Some(e.clone());
        }
    }
    None
}
