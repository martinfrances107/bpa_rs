
use crate::Point;
use crate::grid::reconstruct;
use crate::mesh::MeshFace;
use crate::mesh::MeshPoint;

use glm::Vector3;
use glm::normalize;

fn create_spherical_cloud(slices: i32, stacks: i32) -> Vec<Point> {
    let mut points = vec![Point {
        pos: Vector3::<f32>::new(0.0, 0.0, -1.0),
        normal: Vector3::<f32>::new(0.0, 0.0, -1.0),
    }];

    for slice in 0..slices {
        for stack in 1..stacks {
            let yaw = (slice as f32 / slices as f32) * 2.0 * std::f32::consts::PI;
            let z = (stack as f32 / stacks as f32 - 0.5).sin();
            let r = (1.0 - z * z).sqrt();

            let x = r * yaw.sin();
            let y = r * yaw.cos();

            let pos = Vector3::<f32>::new(x, y, z);
            // This makes no sense, but the original C++ code does this
            // could there be a implicit clone?.
            let normal = normalize(pos - Vector3::<f32>::new(0.0, 0.0, -1.0));
            points.push(Point { pos, normal });
        }
    }

    points.push(Point {
        pos: Vector3::new(0.0, 0.0, 1.0),
        normal: Vector3::new(0.0, 0.0, 1.0),
    });

    points
}

fn measure_reconstruct(points: Vec<Point>, radius: f32) -> Vec<MeshFace<'static>> {
    let start = std::time::Instant::now();
    let result = reconstruct(points, radius);
    let end = std::time::Instant::now();
    let seconds = (end - start).as_secs_f64();
    // original C++ code uses std::cerr
    // println!("Points: {}, Triangles: {}, T/s: {}", points.len(), result.len(), result.len() as f64 / seconds);

    todo!()
}

// TEST_CASE("sphere_36_18", "[reconstruct]") {
// 	const auto cloud = createSphericalCloud(36, 18);
// 	savePoints("sphere_36_18_cloud.ply", cloud);
// 	const auto mesh = measuredReconstruct(cloud, 0.3f);
// 	CHECK(!mesh.empty());
// 	saveTriangles("sphere_36_18_mesh.stl", mesh);
// }
#[test]
fn sphere_36_18() {
    let cloud = create_spherical_cloud(36, 18);
    // save_points("sphere_36_18_cloud.ply", cloud);
    let mesh = measure_reconstruct(cloud, 0.3f32);
    assert!(!mesh.is_empty());
    // save_triangles("sphere_36_18_mesh.stl", mesh);
}
