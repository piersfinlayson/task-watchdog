# Scripts

All scripts expect to be run in the root directory of the repository.

## Building

There are scripts provided to build all supported combinations of features/targets for
* the task-watchdog crate
* all examples.

As well as preparing to check in the code, these scripts are useful for reviewing supported feature combinations.

Example usage, to build all library and example feature/target combinations:

```bash
scripts/build-all.sh
```

This requires the installation of the following targets:

```bash
rustup target add thumbv6m-none-eabi         # RP2040/Pico
rustup target add thumbv8m.main-none-eabihf  # RP235x/Pico 2
rustup target add thumbv7m-none-eabi         # STM32
```

## Flashing examples

Helper scripts are provided to flash the [embassy](examples/src/embassy.rs) example to the Pico and Pico 2.  These use the default features (defmt but no alloc).  Other feature combinations are available.  See [build-examples.sh](build-examples.sh).

Example to flash the embassy example to a Pico via a Debug Probe:

```bash
scripts/flash-embassy-pico.sh
```

## ESP32

At the time of writing, ESP32 support in Rust requires additional tools to be installed.  Your best resource is the [ESP on Rust Book](https://docs.esp-rs.org/book/).

The tl;dr is:

```bash
cargo install espup
cargo install espflash
cargo install cargo-espflash
```

To manually run ESP32 builds you will need to source the ESP build environment in your shell:

```bash
. ~/export-esp.sh
```

And, instead of using the regular `cargo`, use:
```bash
~/.rustup/toolchains/esp/bin/cargo
```

The build scripts will souce the ESP build environment and use the correct verion of cargo (if installed).
