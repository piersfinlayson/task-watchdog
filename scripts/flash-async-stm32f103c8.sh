#!/bin/bash
set -e
set -x

# Include common build script stuff
source scripts/build-common.sh

# Build embassy example ...
EXAMPLE=embassy

# ... for STM32F103C8 (blue pill).  To support another board, find it's value
# in the embassy-stm32 Cargo.toml file and use it here.  However, some boards
# require a custom memory.x file, in which case you'll need to create a new
# feature in `examples/Cargo.toml`, add the new memory.x to `examples/link`
# and add support for it in `examples/build.rs`.  See the embassy examples
# for your board for a sample memory.x and build.rs implementation.
BOARD=stm32f103c8

cargo run --manifest-path=$EXAMPLES_MANIFEST_PATH --bin $EXAMPLE --target $STM32_TARGET --no-default-features --features stm32-embassy,defmt-embassy-stm32,embassy-stm32/$BOARD