# `xyz2stl`

Rust 2024 Edition.

## xyz2Stl

Mesh reconstruction program.

Constructs a mesh surface from an ASCII point cloud (xyz file).

The output is a STL file.

To run the binary

```bash
cd xyz2stl
cargo run -- --help
```

```bash
Usage: xyz2stl [OPTIONS] --input <INPUT> --radius <RADIUS>

Options:
  -i, --input <INPUT>    point cloud file
  -r, --radius <RADIUS>
  -o, --output <OUTPUT>  output mesh file mesh
  -h, --help             Print help
```

if no --output tags specified. The specified input file will be used, with the extension changed to .stl

A sample xyz file is provided in the git repository associated with this crate.

each line contains 6 floats in ascii x, y, z, nx, ny, nz

where p,y,z are the points in 3d-space and nx,ny,nz is a normal vector

here are the first three lines of bunny.xyz

```bash
-0.037830 0.127940 0.004475 1.223420 6.106969 -0.789864
-0.044779 0.128887 0.001905 1.351736 5.963559 -1.435807
-0.068010 0.151244 0.037195 0.367206 5.014972 3.728925
```

Here is the result of running

```bash
cd example/xyz2stl
cargo run --release -- -i ../data/bunny.xyz -r 0.002
 ```

35.9K points are loaded, the mesh is then reconstructed and output typically within 1.5 seconds.

![bunny](https://github.com/martinfrances107/bpa_rs/blob/main/images/Reconstructed.png?raw=true")

## Contributions

Contributions are welcome. In particular if your xyz file has issues please file a github issue.

## Known Issues

A wildly inappropriate radius will hang the program.
  It would be good add an optional timeout.
