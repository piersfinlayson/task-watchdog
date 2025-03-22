//! This build script handles:
//! - Copying `memory.x` to the output directory to allow the firmware to be
//!   created.
//!
//! ## Build-time information
//!
//! ## `memory.x` file handling
//!
//! This build script copies the appropriate memory.x file from the `link/`
//! dir into a directory where the linker can find it at link time.

// Copyright (c) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT or Apache-2.0 license, at your option.

// memory.x handling derived from embassy-rs examples.

fn main() {
    // Re-run this build script if anything in git changes.
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/");

    // Re-run this build script of DEFMT_LOG changes.
    #[cfg(feature = "defmt")]
    println!("cargo:rerun-if-env-changed=DEFMT_LOG");

    // RP2040 and RP235X use different memory.x files.  Ensure the build
    // script is re-run if the appropriate memory.x file changes.  Note that
    // neither file should be called memory.x, as then the linker will pick up
    // that file from our root directory, instead of the version we put in
    // OUT_DIR, below.
    #[cfg(feature = "rp2040-memory-x")]
    let memory_x = {
        println!("cargo:rerun-if-changed=../link/memory.rp2040.x");
        include_bytes!("../link/memory.rp2040.x")
    };
    #[cfg(feature = "rp2040-hal-memory-x")]
    let memory_x = {
        println!("cargo:rerun-if-changed=../link/memory.rp2040-hal.x");
        include_bytes!("../link/memory.rp2040-hal.x")
    };
    #[cfg(feature = "rp2350-memory-x")]
    let memory_x = {
        println!("cargo:rerun-if-changed=../link/memory.rp235x.x");
        include_bytes!("../link/memory.rp235x.x")
    };

    #[cfg(any(any(feature = "rp2040-memory-x", feature = "rp2040-hal-memory-x"), feature = "rp2350-memory-x"))]
    {
        use std::env;
        use std::fs::File;
        use std::io::Write;
        use std::path::PathBuf;
        
        // Put `memory.x` in our output directory and ensure it's on the linker
        // search path.
        let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
        File::create(out.join("memory.x"))
            .unwrap()
            .write_all(memory_x)
            .unwrap();
        println!("cargo:rustc-link-search={}", out.display());
    }

    #[cfg(any(all(feature = "rp2040-memory-x", not(feature = "rp2040-hal")), feature = "rp2350-memory-x"))]
    {
        // Set RP specific linker arguments for the binary.
        println!("cargo:rustc-link-arg-bins=-Tdevice.x");
    }

    // Set embassy linker arguments for the binary.
    println!("cargo:rustc-link-arg=-v");
    #[cfg(not(feature = "esp32"))]
    {
        println!("cargo:rustc-link-arg-bins=--nmagic");
        println!("cargo:rustc-link-arg-bins=-Tlink.x");
    }
    #[cfg(feature = "esp32")]
    {
        println!("cargo:rustc-link-arg-bins=-Wl,-Tlinkall.x");
        println!("cargo:rustc-link-arg=-nostartfiles");
    }

    // Required for defmt
    #[cfg(feature = "defmt")]
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");

    // Only RP2040 uses this linker file.
    #[cfg(all(feature = "rp2040-memory-x", not(feature = "rp2040-hal")))]
    println!("cargo:rustc-link-arg-bins=-Tlink-rp.x");
}
