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
cargo build --target $LOCAL_TARGET --features defmt,embassy,defmt-embassy
cargo build --target $LOCAL_TARGET --features alloc,embassy
cargo build --target $LOCAL_TARGET --features defmt,alloc,embassy,defmt-embassy

# Build all valid combinations of features for the library using the RP2040 target
TARGET=$RP2040_TARGET
cargo build --target $TARGET
cargo build --target $TARGET --no-default-features --features rp2040
cargo build --target $TARGET --no-default-features --features rp2040,defmt
cargo build --target $TARGET --no-default-features --features rp2040,alloc
cargo build --target $TARGET --no-default-features --features rp2040,defmt,alloc
cargo build --target $TARGET --no-default-features --features embassy,rp2040
cargo build --target $TARGET --no-default-features --features embassy,rp2040,defmt
cargo build --target $TARGET --no-default-features --features embassy,rp2040,alloc
cargo build --target $TARGET --no-default-features --features embassy,rp2040,defmt,alloc
cargo build --target $TARGET --no-default-features --features rp2040-hal

# Build all valid combinations of features for the library using the RP2350 target
TARGET=$RP2350_TARGET
cargo build --target $TARGET
cargo build --target $TARGET --no-default-features --features rp2350
cargo build --target $TARGET --no-default-features --features rp2350,defmt
cargo build --target $TARGET --no-default-features --features rp2350,alloc
cargo build --target $TARGET --no-default-features --features rp2350,defmt,alloc
cargo build --target $TARGET --no-default-features --features embassy,rp2350
cargo build --target $TARGET --no-default-features --features embassy,rp2350,defmt
cargo build --target $TARGET --no-default-features --features embassy,rp2350,alloc
cargo build --target $TARGET --no-default-features --features embassy,rp2350,defmt,alloc
cargo build --target $TARGET --no-default-features --features rp2350-hal

# Build all valid combinations of features for the library using the STM32 target
TARGET=$STM32_TARGET
BOARD=stm32f103c8
cargo build --target $TARGET
cargo build --target $TARGET --no-default-features --features stm32,embassy-stm32/$BOARD
cargo build --target $TARGET --no-default-features --features stm32,defmt,embassy-stm32/$BOARD
cargo build --target $TARGET --no-default-features --features stm32,alloc,embassy-stm32/$BOARD
cargo build --target $TARGET --no-default-features --features stm32,defmt,alloc,embassy-stm32/$BOARD
cargo build --target $TARGET --no-default-features --features embassy,stm32,embassy-stm32/$BOARD
cargo build --target $TARGET --no-default-features --features embassy,stm32,defmt,embassy-stm32/$BOARD
cargo build --target $TARGET --no-default-features --features embassy,stm32,alloc,embassy-stm32/$BOARD
cargo build --target $TARGET --no-default-features --features embassy,stm32,defmt,alloc,embassy-stm32/$BOARD