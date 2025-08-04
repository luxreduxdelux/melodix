#!/bin/bash

if [ $BUILD_LINUX -eq 1 ]; then
    # create Linux binary.
    cargo build --release
fi

if [ $BUILD_WINDOWS -eq 1 ]; then
    # TO-DO get the x86_64 tool-chain if missing.
    # create Windows binary.
    cargo build --release --target x86_64-pc-windows-gnu
fi