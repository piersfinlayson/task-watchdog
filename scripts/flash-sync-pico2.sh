#!/bin/bash
set -e
set -x

# Include common build script stuff
source scripts/build-common.sh

# Build embassy example ...
EXAMPLE=rp-sync

# ... for Pico
TARGET=$RP2350_TARGET

cargo run --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features rp2350-hal,defmt