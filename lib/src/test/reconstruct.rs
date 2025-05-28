use std::path::PathBuf;

use glam::Vec3;
use insta::assert_debug_snapshot;

use crate::Point;
use crate::Triangle;
use crate::io::load_xyz;
use crate::reconstruct;

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

fn measure_reconstruct(points: &Vec<Point>, radius: f32) -> Option<Vec<Triangle>> {
    let start = std::time::Instant::now();
    let result = reconstruct(points, radius);
    let end = std::time::Instant::now();
    let seconds = (end - start).as_secs_f64();
    // original C++ code uses std::cerr
    match result {
        Some(ref mesh) => {
            println!(
                "Points: {}, Triangles: {}, T/s: {}",
                points.len(),
                mesh.len(),
                mesh.len() as f64 / seconds
            );
            result
        }
        None => {
            println!("No mesh found");
            None
        }
    }
}

#[test]
fn sphere_36_18() {
    let cloud = create_spherical_cloud(36, 18);
    // if let Err(e) = save_points_and_normals(&PathBuf::from("sphere_36_18_cloud.ply"), &cloud) {
    //     eprintln!("Error saving points: {}", e);
    // }

    match measure_reconstruct(&cloud, 0.3_f32) {
        Some(ref triangles) => {
            assert_debug_snapshot!(triangles);
        }
        None => {
            // Must generate a mesh.
            debug_assert!(false);
        }
    }
}

#[test]
fn sphere_100_50() {
    let cloud = create_spherical_cloud(100, 50);
    // if let Err(e) = save_points_and_normals(&PathBuf::from("sphere_100_50_cloud.ply"), &cloud) {
    //     eprintln!("Error saving points: {}", e);
    // }
    match measure_reconstruct(&cloud, 0.1_f32) {
        Some(ref triangles) => {
            assert_debug_snapshot!(triangles);
        }
        None => {
            // Must generate a mesh.
            debug_assert!(false);
        }
    }
}

#[test]
fn tetrahedron() {
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

    match measure_reconstruct(&cloud, 2f32) {
        Some(ref triangles) => {
            assert_debug_snapshot!(triangles);
        }
        None => {
            // Must generate a mesh.
            debug_assert!(false);
        }
    }
}

#[test]
fn cube() {
    let cloud = vec![
        Point {
            pos: Vec3::new(-1.0, -1.0, -1.0),
            normal: Vec3::new(-1.0, -1.0, -1.0).normalize(),
        },
        Point {
            pos: Vec3::new(-1.0, 1.0, -1.0),
            normal: Vec3::new(-1.0, 1.0, -1.0).normalize(),
        },
        Point {
            pos: Vec3::new(1.0, 1.0, -1.0),
            normal: Vec3::new(1.0, 1.0, -1.0).normalize(),
        },
        Point {
            pos: Vec3::new(1.0, -1.0, -1.0),
            normal: Vec3::new(1.0, -1.0, -1.0).normalize(),
        },
        Point {
            pos: Vec3::new(-1.0, -1.0, 1.0),
            normal: Vec3::new(-1.0, -1.0, 1.0).normalize(),
        },
        Point {
            pos: Vec3::new(-1.0, 1.0, 1.0),
            normal: Vec3::new(-1.0, 1.0, 1.0).normalize(),
        },
        Point {
            pos: Vec3::new(1.0, 1.0, 1.0),
            normal: Vec3::new(1.0, 1.0, 1.0).normalize(),
        },
        Point {
            pos: Vec3::new(1.0, -1.0, 1.0),
            normal: Vec3::new(1.0, -1.0, 1.0).normalize(),
        },
    ];

    match measure_reconstruct(&cloud, 2f32) {
        Some(ref triangles) => {
            assert_debug_snapshot!(triangles);
        }
        None => {
            // Must generate a mesh.
            debug_assert!(false);
        }
    }
}

#[test]
fn bunny() {
    println!("bunny {:#?}", std::env::current_dir());
    let cloud =
        load_xyz(&PathBuf::from("../data/bunny.xyz")).expect("Cannot load bunny for test to begin");

    match measure_reconstruct(&cloud, 0.002f32) {
        Some(ref triangles) => {
            assert_debug_snapshot!(triangles);
        }
        None => {
            // Must generate a mesh.
            debug_assert!(false);
        }
    }
}
