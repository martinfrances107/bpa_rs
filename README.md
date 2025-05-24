# Ball Pivoting Algorithm (BPA)

Rust 2024 Edition.

<div align="center">

<a href="https://crates.io/crates/bpa_rs"><img alt="crates.io" src="https://img.shields.io/crates/v/bpa_rs.svg"/></a>
<a href="https://docs.rs/bpa_rs" rel="nofollow noopener noreferrer"><img src="https://docs.rs/bpa_rs/badge.svg" alt="Documentation"></a>
<a href="https://crates.io/crates/bpa"><img src="https://img.shields.io/crates/d/bpa_rs.svg" alt="Download" /></a>

</div>

Mesh Reconstruction from a Point Cloud.

This a port of this c++ application [bpa](<https://github.com/bernhardmgruber/bpa>)

From this paper.

> "The Ball-Pivoting Algorithm for Surface Reconstruction"

by Fausto Bernardini, Joshua Mittleman, Holly Rushmeier, Claudio Silva and Gabriel Taubin

 <image src="https://github.com/martinfrances107/bpa_rs/blob/main/images/Reconstructed.png?raw=true">

## How to use the library

* Call reconstruct() with your point cloud data.
* Select the ball radius.
* The resultant mesh can then be further processed
* Saved the mesh as a STL file.

Selection of the appropriate radius is a **critical** parameter,  that must be set on a per cloud basis.

* Too small and as the ball rolls it will miss edges.
* Too large will result in a loss of detail.

Here is an skeleton outline of how the library can be used.:-

```rust
    let cloud =
        load_xyz(&PathBuf::from("../data/bunny.xyz")).expect("Cannot load bunny");

    // Construct a mesh from a point cloud.
    match reconstruct(&cloud, 0.002f32) {
        Some(ref triangles) => {
            // triangles is a vector of Triangles
            // where
            //
            // struct Triangle([Vec3; 3]);

            // Process the mesh.
            todo!();

            // Save the triangle as a STL file.
            save_triangles(&PathBuf::from("output.stl"), triangles)
                .expect("Err debug failing writing glue_front.stl");

        }
        None => {
            println!("Did not generate a mesh.");

        }
    }
```

## Testing

### Verification

The original libraries test with  tetrahedron, cubes, spheres and bunny point cloud. Those tests has been recreated.

### Snapshots

The original only tests for the existence of the test meshes. This port snapshots those meshed making any
further development stable.

### Benchmarking

reconstruct() and compute_ball_center() have a criterion test harness..
This version appears to run 40% faster than the cpp version, but I think there is some work be done to enhance performance.

## Further work

Add a WASM example showing a mesh being reconstructed in a web browser.
