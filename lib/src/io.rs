use std::io::BufRead;
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;

use glam::Vec3;

use crate::{Point, Triangle};

static ATTRIBUTE_COUNT: [u8; 2] = [0; 2];

/// Write triangles to file.
///
/// # Errors
///   When the file cannot be created or written to.
///
/// # Panics
///   When the number of triangles exceeds that allow by the stl format.
pub fn save_triangles(path: &PathBuf, triangles: &[Triangle]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = std::fs::File::create(path)?;

    let mut writer = BufWriter::new(file);

    // Header
    writer.write_all(&[b' '; 80])?;

    let count = u32::try_from(triangles.len())
        .expect("stl file format cannot contain more than 4,294,967,295 triangles");
      writer.write_all(&count.to_le_bytes())?;

    for t in triangles {
        // Normals
        let normal = (t.0[0] - t.0[1]).cross(t.0[0] - t.0[2]).normalize();
        let normal_bytes = normal.to_array().map(f32::to_le_bytes).concat();
        writer.write_all(&normal_bytes)?;
        // Triangles
        let triangle_bytes =
            t.0.map(|v| v.to_array())
                .iter()
                .flatten()
                .map(|f| f.to_le_bytes())
                .collect::<Vec<_>>()
                .concat();
              writer.write_all(&triangle_bytes)?;

        // Attribute count
        writer.write_all(&ATTRIBUTE_COUNT)?;
    }

    Ok(())
}

/// Write triangles as a STL file (in ascii format).
///
/// Use only when debugging.
///
/// # Errors
///
/// # Panics
///
pub fn save_triangles_ascii(path: &PathBuf, triangles: &[Triangle]) -> std::io::Result<()> {
    if path.parent().is_some() {
        std::fs::create_dir_all(path.parent().unwrap())?;
    }
    let file = std::fs::File::create(path)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "solid {}", path.to_str().unwrap())?;

    for t in triangles {
        let normal = (t.0[0] - t.0[1]).cross(t.0[0] - t.0[2]).normalize();
        writeln!(
          writer,
            "  facet normal {} {} {}",
            normal.x, normal.y, normal.z
        )?;
        writeln!(writer, "    outer loop")?;
        writeln!(writer, "      vertex {} {} {}", t.0[0].x, t.0[0].y, t.0[0].z)?;
        writeln!(writer, "      vertex {} {} {}", t.0[1].x, t.0[1].y, t.0[1].z)?;
        writeln!(writer, "      vertex {} {} {}", t.0[2].x, t.0[2].y, t.0[2].z)?;
        writeln!(writer, "    endloop")?;
        writeln!(writer, "  endfacet")?;
    }
    writeln!(writer, "endsolid")?;

    Ok(())
}

/// Write Point cloud to file.
///
/// outout point and normal.
///
/// # Errors
///   Problems writing to file.
pub fn save_points_and_normals(
    path: &PathBuf,
    points: &Vec<Point>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = std::fs::File::create(path)?;
    let mut writer = BufWriter::new(file);
    writeln!(writer, "ply")?;
    writeln!(writer, "format binary_little_endian 1.0")?;
    writeln!(writer, "element vertex {}", points.len())?;
    writeln!(writer, "property float x")?;
    writeln!(writer, "property float y")?;
    writeln!(writer, "property float z")?;
    writeln!(writer, "property float nx")?;
    writeln!(writer, "property float ny")?;
    writeln!(writer, "property float nz")?;
    writeln!(writer, "end_header")?;
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
                .to_array()
                .iter()
                .flat_map(|f| f.to_le_bytes())
                .collect::<Vec<u8>>(),
        );
    }
    writer.write_all(&buffer)?;

    Ok(())
}

/// Write Point cloud to file.
///
/// # Errors
///   Problems writing to file.
pub fn save_points(path: &PathBuf, points: &Vec<Vec3>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = std::fs::File::create(path)?;
    let mut writer = BufWriter::new(file);
    writeln!(writer, "ply")?;
    writeln!(writer, "format binary_little_endian 1.0")?;
    writeln!(writer, "element vertex {}", points.len())?;
    writeln!(writer, "property float x")?;
    writeln!(writer, "property float y")?;
    writeln!(writer, "property float z")?;
    writeln!(writer, "end_header")?;
    let mut buffer: Vec<u8> = Vec::new();
    for point in points {
        buffer.extend_from_slice(
            &point
                .to_array()
                .iter()
                .flat_map(|f| f.to_le_bytes())
                .collect::<Vec<u8>>(),
        );
    }
    writer.write_all(&buffer)?;

    Ok(())
}

/// Return a point cloud stored in file.
///
/// # Errors
///   If the file cannot be opened.
///
/// # Panics
///   When there is a unreadable value in the file.
pub fn load_xyz(path: &PathBuf) -> std::io::Result<Vec<Point>> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let mut points = Vec::new();
    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }
        let x: f32 = parts[0].parse().expect("Failed to parse x");
        let y: f32 = parts[1].parse().expect("Failed to parse y");
        let z: f32 = parts[2].parse().expect("Failed to parse z");
        let nx: f32 = parts[3].parse().expect("Failed to parse normal x");
        let ny: f32 = parts[4].parse().expect("Failed to parse normal y");
        let nz: f32 = parts[5].parse().expect("Failed to parse normal z");
        points.push(Point {
            pos: Vec3::new(x, y, z),
            normal: Vec3::new(nx, ny, nz),
        });
    }
    Ok(points)
}
