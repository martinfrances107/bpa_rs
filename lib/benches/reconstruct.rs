use bpa_rs::{Point, reconstruct};
use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use glam::Vec3;


// TODO this breaks D.R.Y its twin is in `lib/src/test/reconstruct.rs`
fn create_spherical_cloud(slices: i32, stacks: i32) -> Vec<Point> {
  let mut points = vec![Point {
      pos: Vec3::new(0.0, 0.0, -1.0),
      normal: Vec3::new(0.0, 0.0, -1.0),
  }];

  for slice in 0..slices {
      for stack in 1..stacks {
          let yaw = (slice as f64 / slices as f64) * 2.0 * std::f64::consts::PI;
          let z = ((stack as f64 / stacks as f64 - 0.5) * std::f64::consts::PI).sin();
          let r = (1.0 - z * z).sqrt();

          let x = (r * yaw.sin()) as f32;
          let y = (r * yaw.cos()) as f32;

          let v = Vec3::new(x as f32, y as f32, z as f32);
          // This makes no sense, but the original C++ code does this
          // could there be a implicit clone?.
          let normal = v - Vec3::new(0.0, 0.0, 0.0).normalize();
          points.push(Point { pos: v, normal });
      }
  }

  points.push(Point {
      pos: Vec3::new(0.0, 0.0, 1.0),
      normal: Vec3::new(0.0, 0.0, 1.0),
  });

  points
}


pub fn sphere_benchmark(c: &mut Criterion) {
  let cloud_100_50 = create_spherical_cloud(100, 50);
  let cloud_36_18 = create_spherical_cloud(36, 18);

    c.bench_function("sphere", |b| {
        b.iter(|| {
            let mesh1 = reconstruct(black_box(&cloud_100_50), black_box(0.1f32));
            assert!(mesh1.is_some(), "Mesh1 should be generated");

            let mesh2 = reconstruct(black_box(&cloud_36_18), black_box(0.3f32));
            assert!(mesh2.is_some(), "Mesh2 should be generated");
        })
    });
}

criterion_group!(
  name = sphere;
  config = Criterion::default().sample_size(15);
  targets = sphere_benchmark
);
criterion_main!(sphere);
