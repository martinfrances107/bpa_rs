use crate::grid::compute_ball_center;
use crate::mesh::{MeshFace, MeshPoint};
use glam::Vec3;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn isosceles() {
    let a = Rc::new(RefCell::new(MeshPoint::new(Vec3::new(0.0, 0.0, 0.0))));
    let b = Rc::new(RefCell::new(MeshPoint::new(Vec3::new(10.0, 0.0, 0.0))));
    let c = Rc::new(RefCell::new(MeshPoint::new(Vec3::new(0.0, 10.0, 0.0))));

    let f = MeshFace([a, b, c]);

    let center = compute_ball_center(&f, 10.0);
    assert_eq!(center, Some(Vec3::new(5.0, 5.0, 7.07106781)));
}

#[test]
fn isosceles_larger_radius() {
    let a = Rc::new(RefCell::new(MeshPoint::new(Vec3::new(0.0, 0.0, 0.0))));
    let b = Rc::new(RefCell::new(MeshPoint::new(Vec3::new(10.0, 0.0, 0.0))));
    let c = Rc::new(RefCell::new(MeshPoint::new(Vec3::new(0.0, 10.0, 0.0))));

    let f = MeshFace([a, b, c]);

    let center = compute_ball_center(&f, 100.0);
    assert_eq!(center, Some(Vec3::new(5.0, 5.0, 99.7496872)));
}

#[test]
fn equilateral() {
    let a = Rc::new(RefCell::new(MeshPoint::new(Vec3::new(0.0, 0.0, 0.0))));
    let b = Rc::new(RefCell::new(MeshPoint::new(Vec3::new(10.0, 0.0, 0.0))));
    let c = Rc::new(RefCell::new(MeshPoint::new(Vec3::new(
        5.0,
        (3.0f32).sqrt() * 10.0 / 2.0,
        0.0,
    ))));

    let f = MeshFace([a, b, c]);

    let center = compute_ball_center(&f, 10.0);
    assert_eq!(center, Some(Vec3::new(4.99999952, 2.88675070, 8.16496658)));
}

#[test]
fn radius_too_small() {
    let a = Rc::new(RefCell::new(MeshPoint::new(Vec3::new(0.0, 0.0, 0.0))));
    let b = Rc::new(RefCell::new(MeshPoint::new(Vec3::new(10.0, 0.0, 0.0))));
    let c = Rc::new(RefCell::new(MeshPoint::new(Vec3::new(0.0, 10.0, 0.0))));

    let f = MeshFace([a, b, c]);

    let center = compute_ball_center(&f, 1.0);
    assert_eq!(center, None);
}
