use std::io::BufRead;
use std::io::Write;
use std::path::PathBuf;

use glam::Vec3;

use crate::{Point, Triangle};

static ATTRIBUTE_COUNT: [u8; 2] = [0; 2];

/// Write Point cloud to file.
pub fn save_points(path: &PathBuf, points: &Vec<Point>) -> Result<(), Box<dyn std::error::Error>> {
    if path.parent().is_none() {
        std::fs::create_dir_all(path.parent().unwrap()).expect("Failed to create directories");
    }

    let mut file = std::fs::File::create(path).expect("Failed to create file");
    writeln!(file, "ply")?;
    writeln!(file, "format binary_little_endian 1.0")?;
    writeln!(file, "element vertex {}", points.len())?;
    writeln!(file, "property float x")?;
    writeln!(file, "property float y")?;
    writeln!(file, "property float z")?;
    writeln!(file, "property float nx")?;
    writeln!(file, "property float ny")?;
    writeln!(file, "property float nz")?;
    writeln!(file, "end_header")?;
    let mut buffer: Vec<u8> = Vec::new();
    for point in points {
        buffer.extend_from_slice(
            &point
                .pos
                .to_array()
                .iter()
                .flat_map(|f| f.to_le_bytes())
                .collect::<Vec<u8>>(),
        );
        buffer.extend_from_slice(
            &point
                .normal
                .unwrap_or_else(|| {
                    println!("Saving a point without a normal ( defaulting to zero ) ");
                    Vec3::default()
                })
                .to_array()
                .iter()
                .flat_map(|f| f.to_le_bytes())
                .collect::<Vec<u8>>(),
        );
    }
    file.write_all(&buffer).expect("Failed to write points");
    file.flush().expect("Failed to flush file");
    file.sync_all().expect("Failed to sync file");

    Ok(())
}

/// Write triangles to file.
pub fn save_triangles(path: &PathBuf, triangles: &[Triangle]) -> std::io::Result<()> {
    if path.parent().is_some() {
        std::fs::create_dir_all(path.parent().unwrap())?;
    }
    let mut file = std::fs::File::create(path)?;

    // Header
    file.write_all(&[b' '; 80])?;

    let count = triangles.len() as u32;
    file.write_all(&count.to_le_bytes())?;

    for t in triangles {
        // Normals
        let normal = (t.0[0] - t.0[1]).cross(t.0[0] - t.0[2]).normalize();
        println!("saving triangle: normal {:?}", normal);
        let normal_bytes = normal.to_array().map(|f| f.to_le_bytes()).concat();
        file.write_all(&normal_bytes)?;
        println!("saving triangle: triangle {:#?}", t);
        // Triangles
        let triangle_bytes =
            t.0.map(|v| v.to_array())
                .iter()
                .flatten()
                .map(|f| f.to_le_bytes())
                .collect::<Vec<_>>()
                .concat();
        file.write_all(&triangle_bytes)?;

        // Attribute count
        file.write_all(&ATTRIBUTE_COUNT)?;
    }

    file.flush()?;
    file.sync_all()?;

    Ok(())
}

/// Write triangles as a STL file (in ascii format).
///
/// Use only when debugging.
pub fn save_triangles_ascii(path: &PathBuf, triangles: &[Triangle]) -> std::io::Result<()> {
    if path.parent().is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).expect("Failed to create directories");
    }
    let mut file = std::fs::File::create(path).expect("Failed to create file");

    writeln!(file, "solid {}", path.to_str().unwrap())?;

    for t in triangles {
        let normal = (t.0[0] - t.0[1]).cross(t.0[0] - t.0[2]).normalize();
        writeln!(
            file,
            "  facet normal {} {} {}",
            normal.x, normal.y, normal.z
        )?;
        writeln!(file, "    outer loop")?;
        writeln!(file, "      vertex {} {} {}", t.0[0].x, t.0[0].y, t.0[0].z)?;
        writeln!(file, "      vertex {} {} {}", t.0[1].x, t.0[1].y, t.0[1].z)?;
        writeln!(file, "      vertex {} {} {}", t.0[2].x, t.0[2].y, t.0[2].z)?;
        writeln!(file, "    endloop")?;
        writeln!(file, "  endfacet")?;
    }
    writeln!(file, "endsolid")?;

    Ok(())
}

/// Return a point cloud stored in file.
pub fn load_xyz(path: &PathBuf) -> Vec<Point> {
    let file = std::fs::File::open(path).expect("Failed to open file");
    let reader = std::io::BufReader::new(file);
    let mut points = Vec::new();
    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }
        let x: f32 = parts[0].parse().expect("Failed to parse x");
        let y: f32 = parts[1].parse().expect("Failed to parse y");
        let z: f32 = parts[2].parse().expect("Failed to parse z");
        points.push(Point {
            pos: Vec3::new(x, y, z),
            normal: Some(Vec3::ZERO),
        });
    }
    points
}
