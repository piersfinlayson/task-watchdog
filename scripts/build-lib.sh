#!/bin/bash
set -e
set -x

# Include common build script stuff
source scripts/build-common.sh

# Build all valid combinations of features for the library using the local target
cargo build --target $LOCAL_TARGET
cargo build --target $LOCAL_TARGET --features defmt
cargo build --target $LOCAL_TARGET --features alloc
cargo build --target $LOCAL_TARGET --features embassy
cargo build --target $LOCAL_TARGET --features defmt,alloc
cargo build --target $LOCAL_TARGET --features embassy,defmt-embassy
cargo build --target $LOCAL_TARGET --features alloc,embassy
cargo build --target $LOCAL_TARGET --features alloc,embassy,defmt-embassy

# Build all valid combinations of features for the library using the RP2040 target
TARGET=$RP2040_TARGET
cargo build --target $TARGET
cargo build --target $TARGET --no-default-features --features rp2040-embassy
cargo build --target $TARGET --no-default-features --features rp2040-embassy,defmt-embassy-rp
cargo build --target $TARGET --no-default-features --features rp2040-embassy,alloc
cargo build --target $TARGET --no-default-features --features rp2040-embassy,defmt-embassy-rp,alloc
cargo build --target $TARGET --no-default-features --features rp2040-hal
cargo build --target $TARGET --no-default-features --features rp2040-hal,defmt
cargo build --target $TARGET --no-default-features --features rp2040-hal,defmt,alloc

# Build all valid combinations of features for the library using the RP2350 target
TARGET=$RP2350_TARGET
cargo build --target $TARGET --no-default-features --features rp2350-embassy
cargo build --target $TARGET --no-default-features --features rp2350-embassy,defmt-embassy-rp
cargo build --target $TARGET --no-default-features --features rp2350-embassy,alloc
cargo build --target $TARGET --no-default-features --features rp2350-embassy,defmt-embassy-rp,alloc
cargo build --target $TARGET --no-default-features --features rp2350-hal
cargo build --target $TARGET --no-default-features --features rp2350-hal,defmt
cargo build --target $TARGET --no-default-features --features rp2350-hal,defmt,alloc

# Build all valid combinations of features for the library using the STM32 target
TARGET=$STM32_TARGET
BOARD=stm32f103c8
cargo build --target $TARGET
cargo build --target $TARGET --no-default-features --features stm32-embassy,embassy-stm32/$BOARD
cargo build --target $TARGET --no-default-features --features stm32-embassy,embassy-stm32/$BOARD
cargo build --target $TARGET --no-default-features --features stm32-embassy,defmt-embassy-stm32,embassy-stm32/$BOARD
cargo build --target $TARGET --no-default-features --features stm32-embassy,alloc,embassy-stm32/$BOARD
cargo build --target $TARGET --no-default-features --features stm32-embassy,defmt-embassy-stm32,alloc,embassy-stm32/$BOARD

# Build all valid combinations of features for the library using the ESP32 target
TARGET=$ESP32_TARGET
export RUSTC=$ESP_RUSTC
export CARGO=$ESP_CARGO
source $HOME/export-esp.sh
$CARGO -Z unstable-options -Z build-std=core build --no-default-features --features esp32-embassy --target $TARGET
$CARGO -Z unstable-options -Z build-std=core,alloc build --no-default-features --features esp32-embassy,alloc --target $TARGET
$CARGO -Z unstable-options -Z build-std=core build --no-default-features --features esp32-embassy,defmt-embassy-esp32 --target $TARGET
$CARGO -Z unstable-options -Z build-std=core,alloc build --no-default-features --features esp32-embassy,defmt-embassy-esp32,alloc --target $TARGET
unset RUSTC
unset CARGO