use core::error::Error;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;

use glam::Vec3;
use log::info;

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
        writeln!(
            writer,
            "      vertex {} {} {}",
            t.0[0].x, t.0[0].y, t.0[0].z
        )?;
        writeln!(
            writer,
            "      vertex {} {} {}",
            t.0[1].x, t.0[1].y, t.0[1].z
        )?;
        writeln!(
            writer,
            "      vertex {} {} {}",
            t.0[2].x, t.0[2].y, t.0[2].z
        )?;
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

/// Return a point cloud stored in file.
///
/// # Errors
///   If the file cannot be opened.
///
/// # Panics
///   When there is a unreadable value in the file.
pub fn load_ply(path: &PathBuf) -> std::io::Result<Vec<Point>> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);

    let header = parse_ply_header(&mut reader)
        .map_err(|_| std::io::Error::other("did not decode header correctly"))?;

    println!("{header:#?}");
    let vertex_count = header.vertex_count;
    let col_count = header.ordered_properties.len();

    let mut points = Vec::new();

    for next in reader.lines() {
        let line = next.map_err(|_| std::io::Error::other("no more lines"))?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        assert!(
            parts.len() == col_count,
            "Did not parse the expected number of cols."
        );

        let mut nx = 0_f32;
        let mut ny = 0_f32;
        let mut nz = 0_f32;
        let mut x = 0_f32;
        let mut y = 0_f32;
        let mut z = 0_f32;
        for (i, (value, _value_type)) in header.ordered_properties.iter().enumerate() {
            if value == "x" {
                x = parts[i].parse().unwrap();
            }
            if value == "y" {
                y = parts[i].parse().unwrap();
            }
            if value == "z" {
                z = parts[i].parse().unwrap();
            }
            if value == "nx" {
                nx = parts[i].parse().unwrap();
            }
            if value == "ny" {
                ny = parts[i].parse().unwrap();
            }
            if value == "nz" {
                nz = parts[i].parse().unwrap();
            }
            // drop comment labels such as r,g,b
        }
        let point = Point {
            pos: Vec3::new(x, y, z),
            normal: Vec3::new(nx, ny, nz),
        };
        // println!("{point:#?}");
        points.push(Point {
            pos: Vec3::new(x, y, z),
            normal: Vec3::new(nx, ny, nz),
        });
    }
    info!("load_ply - extracted points");
    Ok(points)
}

// The file type of the PLY file.
#[derive(Debug)]
enum Format {
    Ascii(f32),
    BinaryLittleEndian(f32),
    BinaryBigEndian(f32),
}

/// Possible types of properties in a PLY file.
///
/// "The type can be specified with one of
///   char uchar short ushort int uint float double,
/// or one of
///   int8 uint8 int16 uint16 int32 uint32 float32 float64"
///
/// As described here <https://en.wikipedia.org/wiki/PLY_(file_format)>
#[derive(Debug)]
enum Type {
    INT8,
    Char,
    Uint8,
    Uchar,
    Int16,
    Short,
    Uint16,
    Int,
    Int32,
    Ushort,
    Uint,
    Uint32,
    Float,
    Float32,
    Double,
    Float64,
}

#[derive(Debug)]
struct UnknownType;

impl std::fmt::Display for UnknownType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown type")
    }
}

impl Error for UnknownType {}

///   char uchar short ushort int uint float double,
/// or one of
///   int8 uint8 int16 uint16 int32 uint32 float32 float64"
///
impl TryFrom<&str> for Type {
    type Error = UnknownType;
    fn try_from(input: &str) -> Result<Self, Self::Error> {
        match input {
            "char" => Ok(Self::Char),
            "int8" => Ok(Self::INT8),

            "uchar" => Ok(Self::Uchar),
            "uint8" => Ok(Self::Uint8),

            "short" => Ok(Self::Short),
            "int16" => Ok(Self::Int16),

            "ushort" => Ok(Self::Ushort),
            "uint16" => Ok(Self::Uint16),

            "int" => Ok(Self::Int),
            "int32" => Ok(Self::Int32),

            "uint" => Ok(Self::Uint),
            "uint32" => Ok(Self::Uint32),

            "float" => Ok(Self::Float),
            "float32" => Ok(Self::Float32),

            "double" => Ok(Self::Double),
            "float64" => Ok(Self::Float64),

            _ => Err(UnknownType),
        }
    }
}
/// The header of a PLY file
#[derive(Debug)]
struct Header {
    /// The format of the PLY file.
    pub format: Format,
    /// The number of vertices in the PLY file.
    pub vertex_count: u64,
    /// The columns of the data section (label, type)
    pub ordered_properties: Vec<(String, Type)>,
}

enum HeaderError {
    InvalidFile,
    Malformed,
}

// Extract data from a PLY header
//header format
// ply
// format ascii 1.0
// comment This is a comment!
// element vertex 779966
// property float x
// property float y
// property float z
// end_header
//
// The second line is one of
// format ascii 1.0
// format binary_little_endian 1.0
// format binary_big_endian 1.0
//
fn parse_ply_header(buffer: &mut BufReader<File>) -> Result<Header, HeaderError> {
    info!("Reading header");
    // Return error is the first line is not "ply"
    let mut line = String::new();
    buffer
        .read_line(&mut line)
        .expect("Failed looking for header token");

    assert!(
        line.starts_with("ply"),
        "Does not container the FILE descriptor of a ply file."
    );

    let mut format: Option<Format> = None;
    let mut ordered_properties = vec![];

    let mut vertex_count: u64 = 0;

    for line in buffer.lines().map(|l| l.unwrap()) {
        info!("parse_ply_header: loop");
        let line = line.trim();
        info!("parse_ply_header: loop {line}");
        // If the line is "end_header", return the header
        if line == "end_header" {
            info!("end_header seen");
            match format {
                Some(format) => {
                    info!("Parsing header complete.");
                    return Ok(Header {
                        format,
                        vertex_count,
                        ordered_properties,
                    });
                }
                None => {
                    panic!("At the end of the header the format is unknown or invalid");
                }
            }
        }

        if line.starts_with("comment") {
            // Ignore comments
            continue;
        }

        if line.starts_with("element face") {
            // Ignore faces
            continue;
        }

        if line.starts_with("element vertex") {
            // Extract the vertex count
            let parts: Vec<&str> = line.split_whitespace().collect();
            assert!(parts.len() == 3, "Failed to parse: {line}");
            vertex_count = parts[2].parse::<u64>().expect("unrecognized count");
            continue;
        }

        if line == "format ascii 1.0" {
            format = Some(Format::Ascii(1.0));
        }
        if line.starts_with("property") {
            // Extract the property
            let parts: Vec<&str> = line.split_whitespace().collect();
            assert!(parts.len() == 3, "Failed to parse: {line}");
            let prop_type = Type::try_from(parts[1]).expect("Unknown type");
            let label = parts[2].to_string();
            ordered_properties.push((label, prop_type));
            continue;
        }
    }

    Err(HeaderError::Malformed)
}
