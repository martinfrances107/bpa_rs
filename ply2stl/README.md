# `bpa_rs`

Rust 2021 Edition.

## Ply2Stl

Mesh reconstruction program.

Constructs a mesh surface from a point cloud ply file.

The output is a STL file.

To run the binary

```bash
cd ply2stl
cargo run -- --help
```

```bash
Usage: ply2stl [OPTIONS] --input <INPUT> --radius <RADIUS>

Options:
  -i, --input <INPUT>    point cloud file
  -r, --radius <RADIUS>
  -o, --output <OUTPUT>  output mesh file mesh
  -h, --help             Print help
```
