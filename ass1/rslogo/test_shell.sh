#!/bin/sh
cargo run -- logo_examples/0_00_empty_file.lg output1.svg 200 200
6991 rslogo logo_examples/0_00_empty_file.lg output2.svg 200 200
echo "diff in 0_00" diff output1.svg output2.svg

