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
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features rp2040,embassy
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features rp2040,embassy,defmt,defmt-embassy-rp,defmt-embassy
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features rp2040,embassy,alloc
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features rp2040,embassy,defmt,defmt-embassy-rp,alloc,defmt-embassy

# ... for the RP2350
TARGET=$RP2350_TARGET
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features rp2350,embassy
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features rp2350,embassy,defmt,defmt-embassy-rp,defmt-embassy
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features rp2350,embassy,alloc
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features rp2350,embassy,defmt,defmt-embassy-rp,defmt-embassy,alloc


# Build STM32 embassy example ...
EXAMPLE=embassy
# ... for STM32F103C8 (blue pill).  
BOARD=stm32f103c8
TARGET=$STM32_TARGET

# The stm32-embassy example doesn't support no defmt or alloc 
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features embassy,defmt,defmt-embassy-stm32,defmt-embassy,stm32,embassy-stm32/$BOARD


# Build rp-sync example ...
EXAMPLE=rp-sync
# ... for the RP2040
TARGET=$RP2040_TARGET

# The rp-sync example doesn't support no defmt, alloc or the Pico 2
cargo build --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $TARGET --no-default-features --features rp2040-hal,cortex-m,defmt
