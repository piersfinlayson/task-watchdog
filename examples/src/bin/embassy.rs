//! Example application using the task-watchdog crate with Embassy, supporting:
//! - the RP2040 or RP2350 (Pico and Pico 2)
//! - STM32 (STM32F103C8, blue pill)
//! - nrF (nRF52840)
//! - ESP32 (Lolin D32 Pro)
//!
//! To run this example, connect a Debug Probe to your host and to the device
//! under test, and from the repository root, run one of:
//!
//! ```bash
//! scripts/flash-async-pico.sh
//! scripts/flash-async-pico2.sh
//! scripts/flash-async-stm32f103c8.sh
//! scripts/flash-async-nrf52840.sh
//! ```
//!
//! On ESP32, connect your device via USB and run:
//!
//! ```bash
//! scripts/flash-async-esp32.sh
//! ```
//!
//! Other options are available, including with and without defmt, and with
//! dynamic memory allocation.  See [`scripts/build-examples.sh`] for the
//! supported combinations.

// Copyright (c) 2025 Piers Finlayson <piers@piers.rocks>
//
// Apache 2.0 or MIT licensed, at your option.

#![no_std]
#![no_main]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(not(feature = "defmt"))]
use core::fmt::Debug;
#[cfg(feature = "defmt")]
use defmt::Format as Debug;
#[cfg(feature = "defmt")]
use defmt::{info, warn};
#[cfg(feature = "defmt")]
use defmt_rtt as _;
#[cfg(not(feature = "esp32"))]
use embassy_executor::main as embassy_main;
use embassy_executor::Spawner;
#[cfg(any(feature = "rp2040", feature = "rp2350"))]
use embassy_rp::config::Config;
#[cfg(any(feature = "rp2040", feature = "rp2350"))]
use embassy_rp::gpio::{Level, Output};
#[cfg(feature = "stm32")]
use embassy_stm32::gpio::{Level, Output, Speed};
#[cfg(feature = "nrf")]
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_time::{Duration, Timer};
#[cfg(feature = "alloc")]
use embedded_alloc::LlffHeap as Heap;
#[cfg(feature = "esp32")]
use esp_backtrace as _;
#[cfg(feature = "esp32")]
use esp_hal::gpio::{Level, Output, OutputConfig};
#[cfg(feature = "esp32")]
use esp_hal_embassy::main as embassy_main;
#[cfg(all(feature = "esp32", not(feature = "defmt")))]
use esp_println::println;
#[cfg(not(feature = "esp32"))]
use panic_probe as _;
use static_cell::StaticCell;

#[cfg(feature = "esp32")]
use task_watchdog::embassy_esp32::{watchdog_run, WatchdogRunner};
#[cfg(any(feature = "rp2040", feature = "rp2350"))]
use task_watchdog::embassy_rp::{watchdog_run, WatchdogRunner};
#[cfg(feature = "stm32")]
use task_watchdog::embassy_stm32::{watchdog_run, WatchdogRunner};
#[cfg(feature = "nrf")]
use task_watchdog::embassy_nrf::{watchdog_run, WatchdogRunner};
use task_watchdog::{Id, WatchdogConfig};

/// If we're not using cfg(feature = "defmt") we need logging macros.
#[cfg(all(feature = "esp32", not(feature = "defmt")))]
macro_rules! info {
    ($fmt:expr $(, $args:expr)* $(,)?) => {
        println!(concat!("INFO: ", $fmt) $(, $args)*)
    };
}
#[cfg(all(feature = "esp32", not(feature = "defmt")))]
macro_rules! warn {
    ($fmt:expr $(, $args:expr)* $(,)?) => {
        println!(concat!("WARN: ", $fmt) $(, $args)*)
    };
}
#[cfg(all(not(feature = "esp32"), not(feature = "defmt")))]
macro_rules! info {
    ($($tt:tt)*) => {};
}
#[cfg(all(not(feature = "esp32"), not(feature = "defmt")))]
macro_rules! warn {
    ($($tt:tt)*) => {};
}

// We need a heap in the alloc case.
#[cfg(feature = "alloc")]
#[global_allocator]
static HEAP: Heap = Heap::empty();

/// Define a type alias for the WatchdogRunner type.  This is a
/// convenience to make use of the WatchdogRunnerType easier.
/// I - whatever type you are using to identify tasks, implementing to Id
///     trait
/// N - the number of tasks to monitor (only used in the no alloc case)
#[cfg(feature = "alloc")]
type WatchdogRunnerType = WatchdogRunner<TaskId>;
#[cfg(not(feature = "alloc"))]
type WatchdogRunnerType = WatchdogRunner<TaskId, NUM_TASK_IDS>;

// Create a static to hold our Watchdog object, so it can be shared between
// tasks.
static WATCHDOG: StaticCell<WatchdogRunnerType> = StaticCell::new();

/// Define a type that implements the `Id` trait, which is used to identify
/// tasks.  This is typically an enum, but can be any type that implements the
/// the `Id` trait.
///
/// The `Id` trait includes `Ord` (and therefore `PartialOrd`) when
/// cfg(feature = "alloc") is enabled.  If you are using cfg(not(feature =
/// "alloc") then you do not need to implement `Ord` or `PartialOrd`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum TaskId {
    Main,
    Sensor,
    Network,
    Failing,
}
impl Id for TaskId {}

/// In the cfg(not(feature = "alloc")) case, we need to define the number of
/// task IDs that will be used.  This is used to statically allocate the
/// `WatchdogRunner` object.
#[cfg(not(feature = "alloc"))]
const NUM_TASK_IDS: usize = 4;

/// Our main entry point.
#[embassy_main]
async fn main(spawner: Spawner) {
    // Initialize the allocator in the cfg(feature = "alloc") case.
    #[cfg(feature = "alloc")]
    {
        #![allow(static_mut_refs)]
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }

    // Initialize the embassy peripherals.
    #[cfg(any(feature = "rp2040", feature = "rp2350"))]
    let p = embassy_rp::init(Config::default());
    #[cfg(feature = "stm32")]
    let p = embassy_stm32::init(Default::default());
    #[cfg(feature = "nrf")]
    let p = embassy_nrf::init(Default::default());
    #[cfg(feature = "esp32")]
    let p = esp_hal::init(esp_hal::Config::default());

    // On ESP32 need to initialize the timer0 peripheral for embassy.
    #[cfg(feature = "esp32")]
    let timg0 = {
        let timg1 = esp_hal::timer::timg::TimerGroup::new(p.TIMG1);
        esp_hal_embassy::init(timg1.timer0);
        esp_hal::timer::timg::TimerGroup::new(p.TIMG0)
    };

    // Do some logging.
    info!("task-watchdog example: async");
    #[cfg(feature = "rp2040")]
    info!("Running on RP2040");
    #[cfg(feature = "rp2350")]
    info!("Running on RP2350");
    #[cfg(feature = "stm32")]
    info!("Running on STM32");
    #[cfg(feature = "nrf")]
    info!("Running on nRF");
    #[cfg(feature = "esp32")]
    info!("Running on ESP32");
    #[cfg(feature = "alloc")]
    info!("The alloc feature is enabled");
    #[cfg(not(feature = "alloc"))]
    info!("The alloc feature is disabled");
    info!("Watchdog will restart due to failing_task after 25 seconds");

    // Flash the on-board LED to show we're alive.  This is helpful if you do
    // not have defmt enabled, to verify the device has reset after 25s (as it
    // will reboot and flash the LED again).
    #[cfg(any(feature = "rp2040", feature = "rp2350"))]
    {
        // On a Pico W or Pico 2 W the LED won't flash, as the LED is attached
        // to the WiFi chip.
        let mut led = Output::new(p.PIN_25, Level::High);
        led.set_high();
        Timer::after_millis(100).await;
        led.set_low();
    }
    #[cfg(feature = "stm32")]
    {
        // On the STM32F103C8 (blue pill), the LED is on PC13.  It's active
        // low.
        let mut led = Output::new(p.PC13, Level::High, Speed::Low);
        led.set_low();
        Timer::after_millis(100).await;
        led.set_high();
    }
    #[cfg(feature = "nrf")]
    {
        // On the ProMicro V1940 nRF52840, the LED is on P0_15.  It's active high.
        let mut led = Output::new(p.P0_15, Level::Low, OutputDrive::Standard);
        led.set_high();
        Timer::after_millis(100).await;
        led.set_low();
    }
    #[cfg(feature = "esp32")]
    {
        // On the Lolin D32 Pro, the on board LED is GPIO5.  It's active low.
        let mut led = Output::new(p.GPIO5, Level::High, OutputConfig::default());
        led.set_low();
        Timer::after_millis(100).await;
        led.set_high();
    }

    // Create task-watchdog config.  Set the hardware watchdog timeout to 5s,
    // and the check interval to 1s.
    let config = WatchdogConfig {
        hardware_timeout: Duration::from_millis(5000),
        check_interval: Duration::from_millis(1000),
    };

    // Create and configure the watchdog runner.
    #[cfg(any(feature = "rp2040", feature = "rp2350"))]
    let watchdog = WatchdogRunner::new(p.WATCHDOG, config);
    #[cfg(feature = "stm32")]
    let watchdog = WatchdogRunner::new(p.IWDG, config);
    #[cfg(feature = "nrf")]
    let watchdog = WatchdogRunner::new(p.WDT, config);
    #[cfg(feature = "esp32")]
    let watchdog = WatchdogRunner::new(timg0, config);

    // Make watchdog static so it can be shared between tasks
    let watchdog = WATCHDOG.init(watchdog);

    // Log the last reset reason
    info!("Last reset reason: {:?}", watchdog.reset_reason().await);

    // Register our tasks.  You can also do this from within the task itself,
    // and you can deregister the task if the task is exiting, or if it is
    // deliberately going to not feed the watchdog for a period of time.
    watchdog
        .register_task(&TaskId::Main, Duration::from_millis(3000))
        .await;
    watchdog
        .register_task(&TaskId::Sensor, Duration::from_millis(5000))
        .await;
    watchdog
        .register_task(&TaskId::Network, Duration::from_millis(10000))
        .await;

    // Spawn the watchdog task.  From this point onwards, it will expect all
    // registered tasks to feed the watchdog, at least as frequently as the
    // max_duration interval passed on register_task().
    spawner.spawn(watchdog_task(watchdog)).unwrap();

    // Spawn our application tasks
    spawner.spawn(main_task(watchdog)).unwrap();
    spawner.spawn(sensor_task(watchdog)).unwrap();
    spawner.spawn(network_task(watchdog)).unwrap();

    // This task will intentionally stop feeding the watchdog after 30
    // seconds.
    spawner.spawn(failing_task(watchdog)).unwrap();
}

// A task to run the watchdog.  In the cfg(not(feature = "alloc")) case, which
// is typical for an embassy application, your code must supply a wrapper
// around the watchdog_run() function as shown.
#[embassy_executor::task]
async fn watchdog_task(watchdog: &'static WatchdogRunnerType) -> ! {
    watchdog_run(watchdog.create_task()).await
}

// A common task function which feeds the watchdog periodically, on behalf
// of that task.
async fn common_task(
    watchdog: &'static WatchdogRunnerType,
    task_id: TaskId,
    duration: Duration,
) -> ! {
    info!("{:?} task started", task_id);

    loop {
        // Do some work here ...

        // Feed the watchdog
        watchdog.feed(&task_id).await;
        info!("{:?} task fed the watchdog", task_id);

        // Sleep for a while
        Timer::after(duration).await;
    }
}

// A regular task that properly feeds the watchdog
#[embassy_executor::task]
async fn main_task(watchdog: &'static WatchdogRunnerType) -> ! {
    common_task(watchdog, TaskId::Main, Duration::from_millis(2000)).await
}

// A sensor reading task that feeds the watchdog
#[embassy_executor::task]
async fn sensor_task(watchdog: &'static WatchdogRunnerType) -> ! {
    common_task(watchdog, TaskId::Sensor, Duration::from_millis(2000)).await
}

// A network communication task
#[embassy_executor::task]
async fn network_task(watchdog: &'static WatchdogRunnerType) -> ! {
    common_task(watchdog, TaskId::Network, Duration::from_millis(5000)).await
}

// A task that will intentionally stop feeding the watchdog after 30 seconds
#[embassy_executor::task]
async fn failing_task(watchdog: &'static WatchdogRunnerType) -> ! {
    info!("Failing task started - will stop feeding after 15 seconds");

    // Register a new task
    watchdog
        .register_task(&TaskId::Failing, Duration::from_millis(5000))
        .await;

    // Feed it regularly for 15 seconds
    for _ in 0..15 {
        watchdog.feed(&TaskId::Failing).await;
        info!("Failing task fed the watchdog");
        Timer::after(Duration::from_millis(1000)).await;
    }

    // Stop feeding - this should trigger a reset after the timeout
    warn!("Failing task has stopped feeding the watchdog!");

    // We give 15s to reboot - 5s for task-watchdog to decide Failing task
    // has stopped feeding the watchdog, and hence to stop feeding the hardware
    // watchdog itself, and then 5s for the hardware watchdog to wait before
    // resetting the system.
    Timer::after(Duration::from_millis(15000)).await;
    panic!("System should have reset by now!");
}
