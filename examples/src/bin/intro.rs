//! This example is from the crate documentation.
//! 
//! It is a minimal example of using the task-watchdog crate with embassy and
//! the RP2040/RP2350.  (The STM32 is very similar, but not included to keep
//! it as simple as possible.)
//! 
//! To build this example, from the repository root, for the RP2040/Pico, run:
//! 
//! ```bash
//! cargo build --manifest-path examples/Cargo.toml --bin intro --no-default-features --features rp2040-embassy --target thumbv6m-none-eabi
//! ```
//! 
//! And for the RP2350/Pico 2, run:
//! 
//! ```bash
//! cargo build --manifest-path examples/Cargo.toml --bin intro --no-default-features --features rp2040-embassy --target thumbv8m.main-none-eabi
//! ```
//! 

// Copyright (c) 2025 Piers Finlayson <piers@piers.rocks>
//
// Apache 2.0 or MIT licensed, at your option.

#![no_std]
#![no_main]

use task_watchdog::{WatchdogConfig, Id};
use task_watchdog::embassy_rp::{WatchdogRunner, watchdog_run};
use embassy_time::{Duration, Timer};
use embassy_rp::config::Config;
use embassy_executor::Spawner;
use static_cell::StaticCell;
use panic_probe as _;

// Create a static to hold the task-watchdog object, so it has static
// lifetime and can be shared with tasks.
static WATCHDOG: StaticCell<WatchdogRunner<TaskId, NUM_TASKS>> = StaticCell::new();

// Create an object to contain our task IDs.  It must implement the Id
// trait, which, for simply TaskId types means deriving the following
// traits:
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TaskId {
    Main,
    Second,
}
impl Id for TaskId {}  // Nothing else to implement as we derived the required traits
const NUM_TASKS: usize = 2;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize the hardare peripherals
    let p = embassy_rp::init(Config::default());

    // Set up watchdog configuration, with a 5s hardware watchdog timeout, and
    // with the task watchdog checking tasks every second.
    let config = WatchdogConfig {
        hardware_timeout: Duration::from_millis(5000),
        check_interval: Duration::from_millis(1000),
    };

    // Create the watchdog runner and store it in the static cell
    let watchdog = WatchdogRunner::new(p.WATCHDOG, config);
    let watchdog = WATCHDOG.init(watchdog);

    // Register our tasks with the task-watchdog.  Each can have a different timeout.
    watchdog.register_task(&TaskId::Main, Duration::from_millis(2000)).await;
    watchdog.register_task(&TaskId::Second, Duration::from_millis(4000)).await;

    // Spawn tasks that will feed the watchdog
    spawner.must_spawn(main_task(watchdog));
    spawner.must_spawn(second_task(watchdog));

    // Finally spawn the watchdog - this will start the hardware watchdog, and feed it
    // for as long as _all_ tasks are healthy.
    spawner.must_spawn(watchdog_task(watchdog));
}

// Provide a simple embassy task for the watchdog
#[embassy_executor::task]
async fn watchdog_task(watchdog: &'static WatchdogRunner<TaskId, NUM_TASKS>) -> ! {
    watchdog_run(watchdog.create_task()).await
}

// Implement your main task
#[embassy_executor::task]
async fn main_task(watchdog: &'static WatchdogRunner<TaskId, NUM_TASKS>) -> !{
   loop {
        // Feed the watchdog
        watchdog.feed(&TaskId::Main).await;

        // Do some work
        Timer::after(Duration::from_millis(1000)).await;
   }
}

// Implement your second task
#[embassy_executor::task]
async fn second_task(watchdog: &'static WatchdogRunner<TaskId, NUM_TASKS>) -> !{
   loop {
        // Feed the watchdog
        watchdog.feed(&TaskId::Second).await;

        // Do some work
        Timer::after(Duration::from_millis(2000)).await;
   }
}
