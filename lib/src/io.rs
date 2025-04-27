use std::io::BufRead;
use std::io::Write;
use std::path::PathBuf;

use glam::Vec3;

use crate::{Point, Triangle};

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
                .unwrap()
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

pub fn save_triangles(path: &PathBuf, triangles: &[Triangle]) {
    if path.parent().is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).expect("Failed to create directories");
    }
    let mut file = std::fs::File::create(path).expect("Failed to create file");
    let header = "STL whatever";
    file.write_all(header.as_bytes())
        .expect("Failed to write header");

    let count = triangles.len() as u32;
    file.write_all(&count.to_le_bytes())
        .expect("Failed to write count");

    for triangle in triangles {
        let normal: [f32; 3] = triangle.normal().into();
        let normal_bytes = normal.map(|f| f.to_le_bytes()).concat();
        file.write_all(&normal_bytes)
            .expect("Failed to write normal");

        let triangle_bytes = triangle
            .0
            .map(|v| v.to_array())
            .iter()
            .flatten()
            .map(|f| f.to_le_bytes())
            .collect::<Vec<_>>()
            .concat();
        file.write_all(&triangle_bytes)
            .expect("Failed to write triangle");

        file.write_all(&0_f32.to_le_bytes())
            .expect("Failed to write attribute count");
    }

    // file.seek(std::io::SeekFrom::Start(0)).expect("Failed to seek");
    file.write_all(&count.to_le_bytes())
        .expect("Failed to write count again");

    file.flush().expect("Failed to flush file");
    file.sync_all().expect("Failed to sync file");
}

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
