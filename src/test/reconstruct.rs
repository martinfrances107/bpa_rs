use std::path::PathBuf;

use glam::Vec3;

use crate::Point;
use crate::Triangle;
use crate::grid::reconstruct;
use crate::io::save_points;
use crate::io::save_triangles;
use crate::load_xyz;
use crate::mesh::MeshFace;
use crate::mesh::MeshPoint;

fn create_spherical_cloud(slices: i32, stacks: i32) -> Vec<Point> {
    let mut points = vec![Point {
        pos: Vec3::new(0.0, 0.0, -1.0),
        normal: Vec3::new(0.0, 0.0, -1.0),
    }];

    for slice in 0..slices {
        for stack in 1..stacks {
            let yaw = (slice as f32 / slices as f32) * 2.0 * std::f32::consts::PI;
            let z = (stack as f32 / stacks as f32 - 0.5).sin();
            let r = (1.0 - z * z).sqrt();

            let x = r * yaw.sin();
            let y = r * yaw.cos();

            let pos = Vec3::new(x, y, z);
            // This makes no sense, but the original C++ code does this
            // could there be a implicit clone?.
            let normal = pos - Vec3::new(0.0, 0.0, -1.0).normalize();
            points.push(Point { pos, normal });
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
    if let Err(e) = save_points(PathBuf::from("sphere_36_18_cloud.ply"), &cloud) {
        eprintln!("Error saving points: {}", e);
    }
    let mesh = measure_reconstruct(&cloud, 0.3f32);
    assert!(mesh.is_some());
    if let Some(triangles) = mesh {
        save_triangles(&PathBuf::from("sphere_36_18_mesh.stl"), &triangles);
    }
}

#[test]
fn sphere_100_50() {
    let cloud = create_spherical_cloud(100, 50);
    if let Err(e) = save_points(PathBuf::from("sphere_100_50_cloud.ply"), &cloud) {
        eprintln!("Error saving points: {}", e);
    }
    let mesh = measure_reconstruct(&cloud, 0.1f32);

    assert!(mesh.is_some());
    if let Some(triangles) = mesh {
        save_triangles(&PathBuf::from("sphere_100_50_mesh.stl"), &triangles);
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

    if let Err(e) = save_points(PathBuf::from("tetrahedron_cloud.ply"), &cloud) {
        eprintln!("Error saving points: {}", e);
    }

    let mesh = measure_reconstruct(&cloud, 2f32);
    assert!(mesh.is_some());
    if let Some(triangles) = mesh {
        save_triangles(&PathBuf::from("tetrahedron_cloud.stl"), &triangles);
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

    if let Err(e) = save_points(PathBuf::from("cube_cloud.ply"), &cloud) {
        eprintln!("Error saving points: {}", e);
    }

    let mesh = measure_reconstruct(&cloud, 2f32);
    assert!(mesh.is_some());
    if let Some(triangles) = mesh {
        save_triangles(&PathBuf::from("cube_mesh.stl"), &triangles);
    }
}

#[test]
fn bunny() {
    println!("bunny {:#?}", std::env::current_dir());
    let cloud = load_xyz(&PathBuf::from("./src/test/data/bunny.xyz"));
    let mesh = measure_reconstruct(&cloud, 0.002f32);
    assert!(mesh.is_some());
    if let Some(triangles) = mesh {
        save_triangles(&PathBuf::from("bunny_mesh.stl"), &triangles);
    }
}
