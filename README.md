# `bpa_rs`

Rust 2021 Edition.

## Ball Pivoting Algorithm (BPA)

**THIS IS A DEVELOPMENT branch .. IT IS NOT YET FUNCTIONAL**

Mesh Reconstruction from a Point Cloud.

This a port of this c++ application [bpa](<https://github.com/bernhardmgruber/bpa>)

The Ball-Pivoting Algorithm for Surface Reconstruction by Fausto Bernardini, Joshua Mittleman, Holly Rushmeier, Claudio Silva and Gabriel Taubin

This project is separated into a library and a "ply2stl" binary file

To run the binary

```bash
cd ply2stl
cargo run -- --help
```

```rust
Usage: ply2stl [OPTIONS] --input <INPUT> --radius <RADIUS>

Options:
  -i, --input <INPUT>    point cloud file
  -r, --radius <RADIUS>
  -o, --output <OUTPUT>  output mesh file mesh
  -h, --help             Print help
```
