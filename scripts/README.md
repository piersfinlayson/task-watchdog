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


