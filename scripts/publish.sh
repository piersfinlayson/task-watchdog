#!/bin/bash
cargo publish --features rp2040-embassy,defmt-embassy-rp --target thumbv6m-none-eabi $1 $2