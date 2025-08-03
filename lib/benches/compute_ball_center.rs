use core::cell::RefCell;
use std::hint::black_box;
use std::rc::Rc;

use criterion::{Criterion, criterion_group, criterion_main};
use glam::Vec3;

use bpa_rs::grid::compute_ball_center;
use bpa_rs::mesh::MeshFace;
use bpa_rs::mesh::MeshPoint;

pub fn isosceles(criterion: &mut Criterion) {
    let a = Rc::new(RefCell::new(MeshPoint::new(Vec3::new(0.0, 0.0, 0.0))));
    let b = Rc::new(RefCell::new(MeshPoint::new(Vec3::new(10.0, 0.0, 0.0))));
    let c = Rc::new(RefCell::new(MeshPoint::new(Vec3::new(0.0, 10.0, 0.0))));

    let f = MeshFace([a, b, c]);

    criterion.bench_function("isosceles", |b| {
        b.iter(|| {
            let center = compute_ball_center(&f, black_box(10.0));
            assert_eq!(center, Some(Vec3::new(5.0, 5.0, 7.07106781)));
        })
    });
}

criterion_group!(benches, isosceles);
criterion_main!(benches);
