#!/bin/bash
set -e
set -x

# Include common build script stuff
source scripts/build-common.sh

# Build Pico embassy example ...
EXAMPLE=embassy

# ... for the RP2040
TARGET=$RP2040_TARGET
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features rp2040-embassy
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features rp2040-embassy,defmt-embassy-rp
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features rp2040-embassy,alloc
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features rp2040-embassy,defmt-embassy-rp,alloc

# ... for the RP2350
TARGET=$RP2350_TARGET
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features rp2350-embassy
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features rp2350-embassy,defmt-embassy-rp
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features rp2350-embassy,alloc
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features rp2350-embassy,defmt-embassy-rp,alloc


# Build STM32 embassy example ...
EXAMPLE=embassy
# ... for STM32F103C8 (blue pill).  
BOARD=stm32f103c8
TARGET=$STM32_TARGET
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features stm32-embassy,embassy-stm32/$BOARD
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features stm32-embassy,defmt-embassy-stm32,embassy-stm32/$BOARD
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features stm32-embassy,alloc,embassy-stm32/$BOARD
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features stm32-embassy,defmt-embassy-stm32,alloc,embassy-stm32/$BOARD

# Build ESP32 embassy example ...
EXAMPLE=embassy
TARGET=$ESP32_TARGET
export RUSTC=$ESP_RUSTC
export CARGO=$ESP_CARGO
source $HOME/export-esp.sh
$CARGO -Z unstable-options -Z build-std=core build --manifest-path $EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --no-default-features --features esp32-embassy,esp32-println --target $TARGET
$CARGO -Z unstable-options -Z build-std=core,alloc build --manifest-path $EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --no-default-features --features esp32-embassy,esp32-println,alloc --target $TARGET
$CARGO -Z unstable-options -Z build-std=core build --manifest-path $EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --no-default-features --features esp32-embassy,defmt-embassy-esp32 --target $TARGET
$CARGO -Z unstable-options -Z build-std=core,alloc build --manifest-path $EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --no-default-features --features esp32-embassy,defmt-embassy-esp32,alloc --target $TARGET
unset RUSTC
unset CARGO

# Build rp-sync example ...
EXAMPLE=rp-sync
# ... for the RP2040
TARGET=$RP2040_TARGET
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features rp2040-hal,defmt
TARGET=$RP2350_TARGET
# ... and for the RP2350
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features rp2350-hal,defmt

# Build intro example ...
EXAMPLE=intro
# ... for the RP2040
TARGET=$RP2040_TARGET
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features rp2040-embassy
# ... for the RP2350
TARGET=$RP2350_TARGET
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features rp2350-embassy

