use bpa_rs::{Point, reconstruct};
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use glam::Vec3;

pub fn tetrahedron_benchmark(c: &mut Criterion) {
    let cloud = vec![
        Point {
            pos: Vec3::new(0.0, 0.0, 0.0),
            normal: Vec3::new(-1.0, -1.0, -1.0).normalize(),
        },
        Point {
            pos: Vec3::new(0.0, 1.0, 0.0),
            normal: Vec3::new(0.0, 1.0, 0.0).normalize(),
        },
        Point {
            pos: Vec3::new(1.0, 0.0, 0.0),
            normal: Vec3::new(1.0, 0.0, 0.0).normalize(),
        },
        Point {
            pos: Vec3::new(0.0, 0.0, 1.0),
            normal: Vec3::new(0.0, 0.0, 1.0).normalize(),
        },
    ];

    c.bench_function("tetrahedron", |b| {
        b.iter(|| {
            let mesh = reconstruct(black_box(&cloud), black_box(2_f32));
            assert!(mesh.is_some(), "Mesh should be generated");
        })
    });
}

criterion_group!(benches, tetrahedron_benchmark);
criterion_main!(benches);
