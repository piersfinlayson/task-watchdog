//! Synchronous API example for task-watchdog on a non-Embassy system, 
//! supporting the RP2040 (Pico) and RP2350 (Pico 2).
//! 
//! To run this example, connect a Debug Probe to your host and to the Pico
//! or Pico 2 under test, and from the repository root, run one of:
//!
//! ```bash
//! scripts/flash-sync-pico.sh
//! scripts/flash-sync-pico2.sh
//! ```

// Copyright (c) 2025 Piers Finlayson <piers@piers.rocks>
//
// Apache 2.0 or MIT licensed, at your option.

#![no_std]
#![no_main]

use defmt::Format as Debug;
use defmt::{info, warn};
use {defmt_rtt as _, panic_probe as _};
use task_watchdog::{Watchdog, WatchdogConfig, Id};
#[cfg(feature = "rp2040-hal")]
use rp2040_hal as hal;
#[cfg(feature = "rp2350-hal")]
use rp235x_hal as hal;
use task_watchdog::rp_hal::{RpHalTaskWatchdog, RpHalClock};
use hal::clocks::init_clocks_and_plls;
use hal::{pac, entry, Sio};
use hal::watchdog::Watchdog as RpHalWatchdog;
use hal::timer::{Timer as RpHalTimer};
use hal::fugit::{Duration as RpHalDuration};
use hal::gpio::Pins;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use cortex_m_rt as _;
#[cfg(feature = "rp2040-hal")]
use rp2040_boot2 as _;

// Create a boot2 section, so binary will boot
#[cfg(feature = "rp2040-hal")]
#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

#[cfg(feature = "rp2350-hal")]
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

// Create a type for the Watchdog to make it easier to use.
type WatchdogType = Watchdog<TaskId, NUM_TASK_IDS, RpHalTaskWatchdog, RpHalClock>;

// Define task IDs for our system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskId {
    Main,
    Sensors,
    Communication,
    Failing,
}
impl Id for TaskId {}
const NUM_TASK_IDS: usize = 4;

#[entry]
fn main() -> ! {
    // Get peripherals
    let mut p = pac::Peripherals::take().unwrap();

    // Initialize clocks and PLLs
    let mut watchdog = RpHalWatchdog::new(p.WATCHDOG);
    const XOSC_CRYSTAL_FREQ: u32 = 12_000_000; // Typically found in BSP crates
    let clocks = init_clocks_and_plls(XOSC_CRYSTAL_FREQ, p.XOSC, p.CLOCKS, p.PLL_SYS, p.PLL_USB, &mut p.RESETS, &mut watchdog).ok().unwrap();

    // Do some logging.
    info!("task-watchdog example: rp-sync");
    info!("Watchdog will restart due to failing_task after 20 seconds");

    // Initialize the timer
    #[cfg(feature = "rp2040-hal")]
    let mut timer = RpHalTimer::new(p.TIMER, &mut p.RESETS, &clocks);
    #[cfg(feature = "rp2350-hal")]
    let mut timer = RpHalTimer::new_timer0(p.TIMER0, &mut p.RESETS, &clocks);

    // Blink the LED to indicate the system has booted
    let sio = Sio::new(p.SIO);
    let pins = Pins::new(p.IO_BANK0, p.PADS_BANK0, sio.gpio_bank0, &mut p.RESETS);
    let mut led = pins.gpio25.into_push_pull_output();
    led.set_high().unwrap();
    timer.delay_ms(100);
    led.set_low().unwrap();

    // Create our watchdog hardware abstraction
    let hw_watchdog = RpHalTaskWatchdog::new(watchdog);

    // Configure the watchdog
    let config = WatchdogConfig {
        hardware_timeout: RpHalDuration::<u64, 1, 1_000_000>::millis(5000),
        check_interval: RpHalDuration::<u64, 1, 1_000_000>::millis(1000),
    };

    // Create our task watchdog
    let mut watchdog: Watchdog<TaskId, NUM_TASK_IDS, RpHalTaskWatchdog, RpHalClock> =
        Watchdog::new(hw_watchdog, config, RpHalClock::new(timer));

    // Register our tasks
    let _ = watchdog.register_task(&TaskId::Main, RpHalDuration::<u64, 1, 1_000_000>::millis(2000));
    let _ = watchdog.register_task(&TaskId::Sensors, RpHalDuration::<u64, 1, 1_000_000>::millis(3000));
    let _ = watchdog.register_task(&TaskId::Communication, RpHalDuration::<u64, 1, 1_000_000>::millis(5000));
    let _ = watchdog.register_task(&TaskId::Failing, RpHalDuration::<u64, 1, 1_000_000>::millis(5000));

    // Start the watchdog
    watchdog.start();

    // Main application loop
    loop {
        // Feed the watchdog for the main task
        watchdog.feed(&TaskId::Main);
        info!("Main task fed the watchdog");

        // Schedule sensor task
        read_sensors(&mut watchdog);

        // Schedule communication task
        communicate(&mut watchdog);

        // Schedule failing task
        failing(&mut watchdog);

        // Check if any tasks are starved and handle accordingly
        let _ = watchdog.check();

        // Delay before next iteration
        timer.delay_ms(1000);
    }
}

fn read_sensors(watchdog: &mut WatchdogType) {
    // Implement your sensor reading logic
    watchdog.feed(&TaskId::Sensors);
    info!("Sensor task fed the watchdog");
}

fn communicate(watchdog: &mut WatchdogType) {
    // Implement your communication logic

    // Feed the watchdog
    watchdog.feed(&TaskId::Communication);
    info!("Communication task fed the watchdog");
}

fn failing(watchdog: &mut WatchdogType) {
    static mut COUNT: u32 = 0;

    // Implement your failing task logic

    // Feed the watchdog - stop after 15 iterations
    let feed = unsafe { 
        COUNT += 1;
        if COUNT < 15 {
            true
        } else {
            false
        } 
    };
    if feed {
        watchdog.feed(&TaskId::Failing);
        info!("Failing task fed the watchdog");
    } else {
        // Stop feeding the watchdog
        warn!("Failing task not feeding the watchdog");
    }
}
