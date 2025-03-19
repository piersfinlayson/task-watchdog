#!/bin/bash
set -e
set -x

# Build all valid combinations of features for the library, for all supported targets
scripts/build-lib.sh

# Build all examples for all supported targets
scripts/build-examples.sh
