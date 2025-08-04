#!/bin/bash

# create Linux binary.
cargo build --release

# create Windows binary.
cargo build --release --target x86_64-pc-windows-gnu