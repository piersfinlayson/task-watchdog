#!/bin/bash
set -e
set -x

# Include common build script stuff
source scripts/build-common.sh

# Build embassy example ...
EXAMPLE=embassy
export RUSTC=$ESP_RUSTC
export CARGO=$ESP_CARGO
source $HOME/export-esp.sh
$CARGO -Z unstable-options -Z build-std=core build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $ESP32_TARGET --no-default-features --features esp32-embassy,esp32-println
unset RUSTC
unset CARGO
espflash flash -S --chip esp32 --monitor target/$ESP32_TARGET/debug/embassy
