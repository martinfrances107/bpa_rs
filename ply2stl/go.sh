!# /usr/bin/bash
rm *.ply *.stl
time cargo run --release -- -i ../data/bunny.xyz -r 0.002

