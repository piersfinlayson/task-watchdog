#!/bin/bash
set -e
set -x

# Include common build script stuff
source scripts/build-common.sh

# Build embassy example ...
EXAMPLE=embassy

# ... for nRF52840.
BOARD=nrf52840

cargo run --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $NRF_TARGET --no-default-features --features nrf-embassy,defmt-embassy-nrf,embassy-nrf/$BOARD