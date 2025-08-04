#!/bin/bash


if [ $PACKAGE_LINUX -eq 1 ]; then
    # TO-DO this is packaging the entire directory and not the executable itself.
    # package Linux binary.
    tar -czvf "${BOONDLE_NAME}_${BOONDLE_VERSION}.tar.gz" "../target/release/melodix"
fi

if [ $PACKAGE_WINDOWS -eq 1 ]; then
    # package Windows binary.
    zip "${BOONDLE_NAME}_${BOONDLE_VERSION}.zip" "../target/x86_64-pc-windows-gnu/release/melodix"
fi