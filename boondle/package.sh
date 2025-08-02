#!/bin/bash

cd "$(dirname "$0")"

zip "${BOONDLE_NAME}_${BOONDLE_VERSION}.zip" "../target/release/melodix"