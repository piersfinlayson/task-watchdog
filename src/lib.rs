//! # task-watchdog
//!
//! A robust, flexible watchdog management library for embedded systems that
//! multiplexes multiple task watchdogs into a single hardware watchdog timer,
//! preventing system lockups when tasks fail to respond
//!
//! This crate provides a task registration pattern that monitors multiple
//! tasks and ensures they are all still active, feeding the hardware
//! watchdog only if all tasks are healthy.
//!
//! Tasks can be dynamically registered and deregistered when the system is
//! running, to allow tasks that are created after startup to be monitoring,
//! and to prevent tasks that are expected to block/pause from causing the
//! device to restart.
//!
//! ![Multiplexed Task Diagram](https://raw.githubusercontent.com/piersfinlayson/task-watchdog/refs/heads/main/docs/images/multiplex.svg)
//!
//! ## Key Features
//!
//! - **Hardware Agnostic API**: Implements a consistent interface across
//!   different embedded microcontrollers with extensible trait system for
//!   hardware watchdog and clock types
//! - **Task Multiplexing**: Consolidates multiple independent task watchdogs
//!   into a single hardware watchdog, triggering if any task fails to check in
//! - **Dynamic Task Management**: Tasks can be registered and deregistered
//!   at runtime, allowing for flexible monitoring configurations
//! - **Async and Sync Support**: Works with both synchronous (via device
//!   HALs) and asynchronous (Embassy) execution environments
//! - **No-Alloc Mode**: Functions in both `alloc` and `no_alloc` modes for
//!   environments with or without heap availability
//! - **Configurable Timeouts**: Individual timeout durations for each
//!   registered task
//! - **`no_std` Compatible**: Designed for resource-constrained embedded
//!   environments without an operating system
//!
//! ## Usage
//!
//! The following is a complete, minimal, example for using the task-watchdog
//! crate using embassy-rs on an RP2040 or RP2350 (Pico or Pico 2).
//! It uses static allocation (no alloc), and creates two tasks with
//! different timeouts, both of which are policed by task-watchdog, and in
//! turn, the hardware watchdog.
//!
//! ```rust
//! #![no_std]
//! #![no_main]
//!
//! use task_watchdog::{WatchdogConfig, Id};
//! use task_watchdog::embassy_rp::{WatchdogRunner, watchdog_run};
//! use embassy_time::{Duration, Timer};
//! use embassy_rp::config::Config;
//! use embassy_executor::Spawner;
//! use static_cell::StaticCell;
//! use panic_probe as _;
//!
//! // Create a static to hold the task-watchdog object, so it has static
//! // lifetime and can be shared with tasks.
//! static WATCHDOG: StaticCell<WatchdogRunner<TaskId, NUM_TASKS>> = StaticCell::new();
//!
//! // Create an object to contain our task IDs.  It must implement the Id
//! // trait, which, for simply TaskId types means deriving the following
//! // traits:
//! #[derive(Clone, Copy, PartialEq, Eq, Debug)]
//! enum TaskId {
//!     Main,
//!     Second,
//! }
//! impl Id for TaskId {}  // Nothing else to implement as we derived the required traits
//! const NUM_TASKS: usize = 2;
//!
//! #[embassy_executor::main]
//! async fn main(spawner: Spawner) {
//!     // Initialize the hardare peripherals
//!     let p = embassy_rp::init(Config::default());
//!
//!     // Set up watchdog configuration, with a 5s hardware watchdog timeout, and
//!     // with the task watchdog checking tasks every second.
//!     let config = WatchdogConfig {
//!         hardware_timeout: Duration::from_millis(5000),
//!         check_interval: Duration::from_millis(1000),
//!     };
//!
//!     // Create the watchdog runner and store it in the static cell
//!     let watchdog = WatchdogRunner::new(p.WATCHDOG, config);
//!     let watchdog = WATCHDOG.init(watchdog);
//!
//!     // Register our tasks with the task-watchdog.  Each can have a different timeout.
//!     watchdog.register_task(&TaskId::Main, Duration::from_millis(2000)).await;
//!     watchdog.register_task(&TaskId::Second, Duration::from_millis(4000)).await  ;
//!
//!     // Spawn tasks that will feed the watchdog
//!     spawner.must_spawn(main_task(watchdog));
//!     spawner.must_spawn(second_task(watchdog));
//!
//!     // Finally spawn the watchdog - this will start the hardware watchdog, and feed it
//!     // for as long as _all_ tasks are healthy.
//!     spawner.must_spawn(watchdog_task(watchdog));
//! }
//!
//! // Provide a simple embassy task for the watchdog
//! #[embassy_executor::task]
//! async fn watchdog_task(watchdog: &'static WatchdogRunner<TaskId, NUM_TASKS>) -> ! {
//!     watchdog_run(watchdog.create_task()).await
//! }
//!
//! // Implement your main task
//! #[embassy_executor::task]
//! async fn main_task(watchdog: &'static WatchdogRunner<TaskId, NUM_TASKS>) -> !{
//!    loop {
//!         // Feed the watchdog
//!         watchdog.feed(&TaskId::Main).await;
//!
//!         // Do some work
//!         Timer::after(Duration::from_millis(1000)).await;
//!    }
//! }
//!
//! // Implement your second task
//! #[embassy_executor::task]
//! async fn second_task(watchdog: &'static WatchdogRunner<TaskId, NUM_TASKS>) -> !{
//!    loop {
//!         // Feed the watchdog
//!         watchdog.feed(&TaskId::Second).await;
//!
//!         // Do some work
//!         Timer::after(Duration::from_millis(2000)).await;
//!    }
//! }
//!
//! ```
//! See the [`README`](https://github.com/piersfinlayson/task-watchdog/blob/main/README.md) and the [examples](https://github.com/piersfinlayson/task-watchdog/tree/main/examples/src) for more usage examples.
//!
//! ## Targets
//!
//! For embedded devices you need to install and specify your target when
//! building.  Use:
//! - RP2040 - `thumbv6m-none-eabi`
//! - RP2350 - `thumbv8m.main-none-eabihf`
//! - STM32 - `thumbv7m-none-eabi`
//!
//! ## Feature Flags
//!
//! The following feature flags are supported
//!
//! ### Embassy support:
//!
//! - `rp2040-embassy`: Enable the RP2040-specific embassy implementation
//! - `rp2350-embassy`: Enable the RP2350-specific embassy implementation
//! - `stm32`: Enable the STM32-specific embassy implementation
//! - `defmt-embassy-rp`: Enable logging with defmt for the RP2040 and RP2350 embassy implementation
//! - `defmt-embassy-stm32`: Enable logging with defmt for the STM32 embassy implementation
//!
//! ### HAL/sync support:
//!
//! - `rp2040-hal`: Enable the RP2040 HAL implementation
//! - `rp2350-hal`: Enable the RP2350 HAL implementation
//! - `defmt`: Enable logging with defmt, for use with the HAL implementations
//!
//! ### Other
//!
//! - `alloc`: Enable features that require heap allocation but simplifies
//! usage
//!
//! ### Example Feature/Target combination
//!
//! This builds the library for RP2040 with embassy and defmt support:
//!
//! ```bash
//! cargo build --features rp2040-embassy,defmt-embassy-rp --target thumbv6m-none-eabi
//! ```
//!
//! ## Embassy Objects
//!
//! If you want to use an include, off the shelf implementation that works with
//! Embassy the objects, you need to use are:
//!
//! - [`WatchdogConfig`] - Used to configure the task-watchdog.
//! - [`embassy_rp::WatchdogRunner`] - Create with the hardware watchdog
//! peripheral and `WatchdogConfig`, and then use to operate the task-watchdog, including task management.  There is also an `embassy_stm32::WatchdogRunner` for STM32.
//! - [`Id`] - Trait for task identifiers.  If you use an enum, derive the
//! `Clone`, `Copy`, `PartialEq`, `Eq` and `Debug`/`Format` traits, and then
//! implement `Id` for the enum.  The Id implementation can be empty, if you
//! derive the required implementations.  You must also derive orimplement
//! `PartialOrd` and `Ord` if you use the `alloc` feature.
//! - [`embassy_rp::watchdog_run()`] - Create and spawn a simple embassy task
//! that just calls this function.  This task will handle policing your other
//! tasks and feeding the hardware watchdog.  There is also an
//! `embassy_stm32::watchdog_run()` for STM32.
//!

// Copyright (c) 2025 Piers Finlayson <piers@piers.rocks>
//
// Apache 2.0 or MIT licensed, at your option.

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;

#[cfg(feature = "defmt")]
#[allow(unused_imports)]
use defmt::{debug, error, info, trace, warn};

// A replacement for the defmt logging macros, when defmt is not provided
#[cfg(not(feature = "defmt"))]
mod log_impl {
    #![allow(unused_macros)]
    #![allow(unused_imports)]
    // Macros are defined as _ to avoid conflicts with built-in attribute
    // names
    macro_rules! _trace {
        ($($arg:tt)*) => {};
    }
    macro_rules! _debug {
        ($($arg:tt)*) => {};
    }
    macro_rules! _info {
        ($($arg:tt)*) => {};
    }
    macro_rules! _warn {
        ($($arg:tt)*) => {};
    }
    macro_rules! _error {
        ($($arg:tt)*) => {};
    }
    pub(crate) use _debug as debug;
    pub(crate) use _error as error;
    pub(crate) use _info as info;
    pub(crate) use _trace as trace;
    pub(crate) use _warn as warn;
}
#[cfg(not(feature = "defmt"))]
use log_impl::*;

/// Represents a hardware-level watchdog that can be fed and reset the system.
pub trait HardwareWatchdog<C: Clock> {
    /// Start the hardware watchdog with the given timeout.
    fn start(&mut self, timeout: C::Duration);

    /// Feed the hardware watchdog to prevent a system reset.
    fn feed(&mut self);

    /// Trigger a hardware reset.
    fn trigger_reset(&mut self) -> !;

    /// Get the reason for the last reset, if available.
    fn reset_reason(&self) -> Option<ResetReason>;
}

/// Represents the reason for a system reset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ResetReason {
    /// Reset was forced by software.
    Forced,

    /// Reset was caused by watchdog timeout.
    TimedOut,
}

/// Configuration for the watchdog.
#[derive(Debug, Clone, Copy)]
pub struct WatchdogConfig<C: Clock> {
    /// Timeout to start the hardware watchdog with.
    pub hardware_timeout: C::Duration,

    /// Interval at which to check if tasks have fed the watchdog.  Must be
    /// less than the hardware timeout, or the hardware watchdog will reset
    /// the system, before the task-watchdog has a chance to check tasks and
    /// feed it.
    pub check_interval: C::Duration,
}

impl<C: Clock> WatchdogConfig<C> {
    /// Create a new configuration with specified timeout values
    pub fn new(hardware_timeout_ms: u64, check_interval_ms: u64, clock: &C) -> Self {
        Self {
            hardware_timeout: clock.duration_from_millis(hardware_timeout_ms),
            check_interval: clock.duration_from_millis(check_interval_ms),
        }
    }

    /// Create a default configuration with standard timeout values:
    /// - Hardware timeout: 5000ms
    /// - Check interval: 1000ms
    pub fn default(clock: &C) -> Self {
        Self::new(5000, 1000, clock)
    }
}

#[cfg(all(feature = "alloc", feature = "defmt"))]
/// Trait for task identifiers.
pub trait Id: PartialEq + Eq + Ord + defmt::Format + Clone + Copy {}
#[cfg(all(feature = "alloc", not(feature = "defmt")))]
/// Trait for task identifiers.
pub trait Id: PartialEq + Eq + Ord + core::fmt::Debug + Clone + Copy {}
#[cfg(all(not(feature = "alloc"), feature = "defmt"))]
/// Trait for task identifiers.
///
/// You need an object implementing this trait (likely by using derive()) in
/// order to identify tasks to the watchdog.  This can be any object you
/// like implementing this trait, although an `enum` would probably be a
/// good choice.
pub trait Id: PartialEq + Eq + defmt::Format + Clone + Copy {}
#[cfg(all(not(feature = "alloc"), not(feature = "defmt")))]
/// Trait for task identifiers.
pub trait Id: PartialEq + Eq + core::fmt::Debug + Clone + Copy {}

/// Represents a task monitored by the watchdog.
#[derive(Debug, Clone)]
pub struct Task<I: Id, C: Clock> {
    /// The task identifier.
    #[allow(dead_code)]
    id: I,

    /// The last time the task was fed.
    last_feed: C::Instant,

    /// Maximum duration between feeds.
    max_duration: C::Duration,
}

impl<I: Id, C: Clock> Task<I, C> {
    /// Creates a new Task object for registration with the watchdog.
    pub fn new(id: I, max_duration: C::Duration, clock: &C) -> Self {
        Self {
            id,
            last_feed: clock.now(),
            max_duration,
        }
    }

    /// Feed the task to indicate it's still active.
    fn feed(&mut self, clock: &C) {
        self.last_feed = clock.now();
    }

    /// Check if this task has starved the watchdog.
    fn is_starved(&self, clock: &C) -> bool {
        clock.has_elapsed(self.last_feed, &self.max_duration)
    }
}

/// A trait for time-keeping implementations.
pub trait Clock {
    /// A type representing a specific instant in time.
    type Instant: Copy;

    /// A type representing a duration of time
    type Duration: Copy;

    /// Get the current time.
    fn now(&self) -> Self::Instant;

    /// Calculate the duration elapsed since the given instant.
    fn elapsed_since(&self, instant: Self::Instant) -> Self::Duration;

    /// Check if a duration has passed since the given instant.
    fn has_elapsed(&self, instant: Self::Instant, duration: &Self::Duration) -> bool;

    /// Create a duration from milliseconds.
    fn duration_from_millis(&self, millis: u64) -> Self::Duration;
}

/// A watchdog that monitors multiple tasks and resets the system if any task fails to feed.
#[cfg(feature = "alloc")]
pub struct Watchdog<I: Id, W: HardwareWatchdog<C>, C: Clock> {
    /// The hardware watchdog.
    hw_watchdog: W,

    /// Tasks being monitored.
    tasks: BTreeMap<I, Task<I, C>>,

    /// Configuration.
    config: WatchdogConfig<C>,

    /// Clock for time-keeping.
    clock: C,
}

#[cfg(feature = "alloc")]
impl<I: Id, W: HardwareWatchdog<C>, C: Clock> Watchdog<I, W, C> {
    /// Create a new watchdog with the given hardware watchdog and configuration.
    pub fn new(hw_watchdog: W, config: WatchdogConfig<C>, clock: C) -> Self {
        Self {
            hw_watchdog,
            tasks: BTreeMap::new(),
            config,
            clock,
        }
    }

    /// Register a task with the watchdog.
    pub fn register_task(&mut self, id: &I, max_duration: C::Duration) {
        let task = Task::new(*id, max_duration, &self.clock);
        self.tasks.insert(*id, task);
        debug!("Registered task: {:?}", id);
    }

    /// Deregister a task from the watchdog.
    pub fn deregister_task(&mut self, id: &I) {
        #[allow(clippy::if_same_then_else)]
        if self.tasks.remove(id).is_some() {
            debug!("Deregistered task: {:?}", id);
        } else {
            debug!("Attempted to deregister unknown task: {:?}", id);
        }
    }

    /// Feed the watchdog for a specific task.
    pub fn feed(&mut self, id: &I) {
        if let Some(task) = self.tasks.get_mut(id) {
            task.feed(&self.clock);
        } else {
            warn!("Attempt to feed unknown task: {:?}", id);
        }
    }

    /// Start the watchdog.
    pub fn start(&mut self) {
        // Feed all registered tasks
        for task in self.tasks.values_mut() {
            task.feed(&self.clock);
        }

        // Start the hardware watchdog
        self.hw_watchdog.start(self.config.hardware_timeout);

        info!("Watchdog started");
    }

    /// Check if any tasks have starved the watchdog and take appropriate action.
    pub fn check(&mut self) -> bool {
        // Check if any tasks have starved
        let mut starved = false;
        for task in self.tasks.values() {
            if task.is_starved(&self.clock) {
                error!("Task {:?} has starved the watchdog", task.id);
                starved = true;
            }
        }

        // Either feed the hardware watchdog or return that we have a starved task
        if !starved {
            self.hw_watchdog.feed();
        }

        starved
    }

    /// Trigger a system reset.
    pub fn trigger_reset(&mut self) -> ! {
        warn!("Triggering watchdog reset");
        self.hw_watchdog.trigger_reset()
    }

    /// Get the reason for the last reset.
    pub fn reset_reason(&self) -> Option<ResetReason> {
        self.hw_watchdog.reset_reason()
    }
}

/// A version of the Watchdog that doesn't require heap allocation.
/// This uses a fixed-size array for task storage.
#[cfg(not(feature = "alloc"))]
pub struct Watchdog<I, const N: usize, W, C>
where
    I: Id,
    W: HardwareWatchdog<C>,
    C: Clock,
{
    /// The hardware watchdog.
    hw_watchdog: W,

    /// Tasks being monitored.
    tasks: [Option<Task<I, C>>; N],

    /// Configuration.
    config: WatchdogConfig<C>,

    /// Clock for time-keeping.
    clock: C,
}

/// Errors that can occur when interacting with the watchdog.
pub enum Error {
    /// No slots available to register a task.
    NoSlotsAvailable,
}

#[cfg(not(feature = "alloc"))]
impl<I: Id, W: HardwareWatchdog<C>, C: Clock, const N: usize> Watchdog<I, N, W, C> {
    /// Create a new watchdog with the given hardware watchdog and configuration.
    ///
    /// Arguments:
    /// * `hw_watchdog` - The hardware watchdog to use.
    /// * `config` - The configuration for the watchdog.
    /// * `clock` - The clock implementation to use for time-keeping.
    pub fn new(hw_watchdog: W, config: WatchdogConfig<C>, clock: C) -> Self {
        Self {
            hw_watchdog,
            tasks: [const { None }; N],
            config,
            clock,
        }
    }

    /// Register a task with the watchdog.
    ///
    /// The task will be monitored by the watchdog.
    ///
    /// Arguments:
    /// * `id` - The task identifier.
    /// * `max_duration` - The maximum duration between feeds.  If there is
    ///                    a gap longer than this, the watchdog will trigger.
    ///
    /// # Errors
    ///
    /// If there are no available slots to register the task, an error will be
    /// returned.
    pub fn register_task(&mut self, id: &I, max_duration: C::Duration) -> Result<(), Error> {
        // Find an empty slot
        for slot in &mut self.tasks {
            if slot.is_none() {
                *slot = Some(Task::new(*id, max_duration, &self.clock));
                debug!("Registered task: {:?}", id);
                return Ok(());
            }
        }

        // No empty slots available
        error!("Failed to register task: {:?} - no slots available", id);
        Err(Error::NoSlotsAvailable)
    }

    /// Deregister a task from the watchdog.
    ///
    /// The task will no longer be monitored by the watchdog.
    ///
    /// Arguments:
    /// * `id` - The task identifier.
    pub fn deregister_task(&mut self, id: &I) {
        for slot in &mut self.tasks {
            if let Some(task) = slot {
                if core::mem::discriminant(&task.id) == core::mem::discriminant(id) {
                    *slot = None;
                    debug!("Deregistered task: {:?}", id);
                    return;
                }
            }
        }

        info!("Attempted to deregister unknown task: {:?}", id);
    }

    /// Feed the watchdog for a specific task.
    pub fn feed(&mut self, id: &I) {
        let fed = self.tasks.iter_mut().flatten().any(|task| {
            if core::mem::discriminant(&task.id) == core::mem::discriminant(id) {
                task.feed(&self.clock);
                true
            } else {
                false
            }
        });

        if !fed {
            warn!("Attempt to feed unknown task: {:?}", id);
        }
    }

    /// Start the watchdog.
    ///
    /// This starts the hardware watchdog.  You must run the watchdog task
    /// now to monitor the tasks.
    pub fn start(&mut self) {
        // Feed all registered tasks
        self.tasks.iter_mut().flatten().for_each(|task| {
            task.feed(&self.clock);
        });

        // Start the hardware watchdog
        self.hw_watchdog.start(self.config.hardware_timeout);

        info!("Watchdog started");
    }

    /// Check if any tasks have starved the watchdog and take appropriate action.
    pub fn check(&mut self) -> bool {
        // Check if any tasks have starved
        let mut starved = false;
        self.tasks.iter_mut().flatten().for_each(|task| {
            if task.is_starved(&self.clock) {
                error!("Task {:?} has starved the watchdog", task.id);
                starved = true;
            }
        });

        // Either feed the hardware watchdog or return that we have a starved
        // task
        if !starved {
            self.hw_watchdog.feed();
        }

        starved
    }

    /// Trigger a system reset.
    pub fn trigger_reset(&mut self) -> ! {
        warn!("Triggering watchdog reset");
        self.hw_watchdog.trigger_reset()
    }

    /// Get the reason for the last reset.
    pub fn reset_reason(&self) -> Option<ResetReason> {
        self.hw_watchdog.reset_reason()
    }
}

/// A system clock implementation using core time types, which allows
/// task-watchdog to work with different clock implementations.
pub struct CoreClock;

impl Clock for CoreClock {
    type Instant = u64; // Simple millisecond counter
    type Duration = core::time::Duration;

    fn now(&self) -> Self::Instant {
        // In real code, this would use a hardware timer
        // This is just a simple example
        static mut MILLIS: u64 = 0;
        unsafe {
            MILLIS += 1;
            MILLIS
        }
    }

    fn elapsed_since(&self, instant: Self::Instant) -> Self::Duration {
        let now = self.now();
        let elapsed_ms = now.saturating_sub(instant);
        core::time::Duration::from_millis(elapsed_ms)
    }

    fn has_elapsed(&self, instant: Self::Instant, duration: &Self::Duration) -> bool {
        self.elapsed_since(instant) >= *duration
    }

    fn duration_from_millis(&self, millis: u64) -> Self::Duration {
        core::time::Duration::from_millis(millis)
    }
}

/// A system clock implementation for Embassy.
#[cfg(feature = "embassy")]
pub struct EmbassyClock;

#[cfg(feature = "embassy")]
impl Clock for EmbassyClock {
    type Instant = embassy_time::Instant;
    type Duration = embassy_time::Duration;

    fn now(&self) -> Self::Instant {
        embassy_time::Instant::now()
    }

    fn elapsed_since(&self, instant: Self::Instant) -> Self::Duration {
        embassy_time::Instant::now() - instant
    }

    fn has_elapsed(&self, instant: Self::Instant, duration: &Self::Duration) -> bool {
        (embassy_time::Instant::now() - instant) >= *duration
    }

    fn duration_from_millis(&self, millis: u64) -> Self::Duration {
        embassy_time::Duration::from_millis(millis)
    }
}

/// An syncronous implementation of task-watchdog for use with the RP2040
/// and RP2350 HALs.
///
/// This module requires either the `rp2040-hal` or `rp2350-hal` feature.
///
/// See the [`rp-sync`](https://github.com/piersfinlayson/task-watchdog/blob/main/examples/src/rp-sync.rs)
/// example for how to use this module.
#[cfg(any(feature = "rp2040-hal", feature = "rp2350-hal"))]
pub mod rp_hal {
    use super::{Clock, HardwareWatchdog, ResetReason};
    use hal::fugit::{Duration as RpHalDuration, MicrosDurationU32};
    #[cfg(feature = "rp2350-hal")]
    use hal::timer::CopyableTimer0;
    use hal::timer::{Instant as RpHalInstant, Timer as RpHalTimer};
    use hal::watchdog::Watchdog as RpHalWatchdog;
    #[cfg(feature = "rp2040-hal")]
    use rp2040_hal as hal;
    #[cfg(feature = "rp2350-hal")]
    use rp235x_hal as hal;

    /// A simple clock implementation based on hal::timer::Timer
    #[cfg(feature = "rp2040-hal")]
    pub struct RpHalClock {
        inner: RpHalTimer,
    }
    #[cfg(feature = "rp2040-hal")]
    impl RpHalClock {
        pub fn new(timer: RpHalTimer) -> Self {
            Self { inner: timer }
        }
    }
    #[cfg(feature = "rp2350-hal")]
    pub struct RpHalClock {
        inner: RpHalTimer<CopyableTimer0>,
    }
    #[cfg(feature = "rp2350-hal")]
    impl RpHalClock {
        pub fn new(timer: RpHalTimer<CopyableTimer0>) -> Self {
            Self { inner: timer }
        }
    }

    /// Implement the Clock trait for [`RpHalClock`]
    impl Clock for RpHalClock {
        type Instant = RpHalInstant;
        type Duration = RpHalDuration<u64, 1, 1_000_000>;

        fn now(&self) -> Self::Instant {
            self.inner.get_counter()
        }

        fn elapsed_since(&self, instant: Self::Instant) -> Self::Duration {
            self.now().checked_duration_since(instant).unwrap()
        }

        fn has_elapsed(&self, instant: Self::Instant, duration: &Self::Duration) -> bool {
            (self.now() - instant) >= *duration
        }

        fn duration_from_millis(&self, millis: u64) -> Self::Duration {
            RpHalDuration::<u64, 1, 1_000_000>::millis(millis as u64)
        }
    }

    /// A hardware watchdog implementation using the RP2040/RP2350 HAL.
    pub struct RpHalTaskWatchdog {
        inner: RpHalWatchdog,
    }
    impl RpHalTaskWatchdog {
        pub fn new(watchdog: RpHalWatchdog) -> Self {
            Self { inner: watchdog }
        }
    }

    /// Implement the HardwareWatchdog trait for the HAL watchdog.
    impl HardwareWatchdog<RpHalClock> for RpHalTaskWatchdog {
        fn start(&mut self, timeout: <RpHalClock as Clock>::Duration) {
            let timeout_micros = timeout.to_micros();
            assert!(timeout_micros <= u32::MAX as u64);
            let micros_dur_u32: MicrosDurationU32 =
                MicrosDurationU32::micros(timeout_micros as u32);
            self.inner.start(micros_dur_u32);
        }

        fn feed(&mut self) {
            self.inner.feed();
        }

        fn trigger_reset(&mut self) -> ! {
            // There is no reset() method on the hal watchdog, so we call
            // hal::reset() directly
            hal::reset()
        }

        fn reset_reason(&self) -> Option<ResetReason> {
            // The hal watchdog does support support a way to check the last
            // reset reason
            None
        }
    }
}

/// An async implementation of task-watchdog for use with the RP2040 and RP2350
/// embassy implementations.  There is an stm32 equivalent of this module.
///
/// This module requires either the `rp2040-embassy` or `rp2350-embassy`
/// feature.
///
/// See the [`embassy`](https://github.com/piersfinlayson/task-watchdog/blob/main/examples/src/embassy.rs)
/// example for how to use this module.
///
/// There is an equivalent `embassy_stm32` module for STM32, but due to
/// docs.rs limitations it is not documented here.  See the above example for
/// usage of that module.
#[cfg(any(feature = "rp2040-embassy", feature = "rp2350-embassy"))]
pub mod embassy_rp {
    use super::{
        info, Clock, EmbassyClock, HardwareWatchdog, Id, ResetReason, Watchdog, WatchdogConfig,
    };
    use embassy_rp::peripherals::WATCHDOG as P_RpWatchdog;
    use embassy_rp::watchdog as rp_watchdog;
    use embassy_time::{Instant, Timer};

    /// RP2040/RP2350-specific watchdog implementation.
    pub struct RpWatchdog {
        inner: rp_watchdog::Watchdog,
    }

    impl RpWatchdog {
        /// Create a new RP2040/RP2350 watchdog.
        #[must_use]
        pub fn new(peripheral: P_RpWatchdog) -> Self {
            Self {
                inner: rp_watchdog::Watchdog::new(peripheral),
            }
        }
    }

    /// Implement the HardwareWatchdog trait for the RP2040/RP2350 watchdog.
    impl HardwareWatchdog<EmbassyClock> for RpWatchdog {
        fn start(&mut self, timeout: <EmbassyClock as Clock>::Duration) {
            self.inner.start(timeout);
        }

        fn feed(&mut self) {
            self.inner.feed();
        }

        fn trigger_reset(&mut self) -> ! {
            self.inner.trigger_reset();
            panic!("Triggering reset via watchdog failed");
        }

        fn reset_reason(&self) -> Option<ResetReason> {
            self.inner.reset_reason().map(|reason| match reason {
                embassy_rp::watchdog::ResetReason::Forced => ResetReason::Forced,
                embassy_rp::watchdog::ResetReason::TimedOut => ResetReason::TimedOut,
            })
        }
    }

    /// An Embassy RP2040/RP2350 watchdog runner.
    #[cfg(feature = "alloc")]
    pub struct WatchdogRunner<I>
    where
        I: Id,
    {
        watchdog: embassy_sync::mutex::Mutex<
            embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
            core::cell::RefCell<Watchdog<I, RpWatchdog, EmbassyClock>>,
        >,
    }

    /// An Embassy RP2040/RP2350 watchdog runner.
    ///
    /// There is an equivalent version of this when using the `alloc` feature
    /// which does not include the `const N: usize` type.
    ///
    /// There is also an equivalent STM32 watchdog runner in the
    /// `embassy_stm32` module.
    ///
    /// Create the watchdog runner using the [`WatchdogRunner::new()`] method, and then use the
    /// methods to register tasks and feed the watchdog.  You probably don't
    /// want to access the other methods directly - use [`watchdog_run()`] to
    /// handle running the task-watchdog.
    #[cfg(not(feature = "alloc"))]
    pub struct WatchdogRunner<I, const N: usize>
    where
        I: Id,
    {
        watchdog: embassy_sync::mutex::Mutex<
            embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
            core::cell::RefCell<Watchdog<I, N, RpWatchdog, EmbassyClock>>,
        >,
    }

    #[cfg(feature = "alloc")]
    impl<I> WatchdogRunner<I>
    where
        I: Id + 'static,
    {
        /// Create a new Embassy-compatible watchdog runner.
        pub fn new(hw_watchdog: P_RpWatchdog, config: WatchdogConfig<EmbassyClock>) -> Self {
            let hw_watchdog = RpWatchdog::new(hw_watchdog);
            let watchdog = Watchdog::new(hw_watchdog, config, EmbassyClock);
            Self {
                watchdog: embassy_sync::mutex::Mutex::new(core::cell::RefCell::new(watchdog)),
            }
        }

        /// Register a task with the watchdog.
        pub async fn register_task(&self, id: &I, max_duration: <EmbassyClock as Clock>::Duration) {
            self.watchdog
                .lock()
                .await
                .borrow_mut()
                .register_task(id, max_duration);
        }

        /// De-register a task with the watchdog.
        pub async fn deregister_task(&self, id: &I) {
            self.watchdog.lock().await.borrow_mut().deregister_task(id);
        }

        /// Feed the watchdog for a specific task.
        pub async fn feed(&self, id: &I) {
            self.watchdog.lock().await.borrow_mut().feed(id);
        }

        /// Start the watchdog.
        pub async fn start(&self) {
            self.watchdog.lock().await.borrow_mut().start();
        }

        /// Trigger a system reset.
        pub async fn trigger_reset(&self) -> ! {
            self.watchdog.lock().await.borrow_mut().trigger_reset()
        }

        /// Get the last reset reason.
        pub async fn reset_reason(&self) -> Option<ResetReason> {
            self.watchdog.lock().await.borrow().reset_reason()
        }

        /// Get the check interval
        pub async fn get_check_interval(&self) -> <EmbassyClock as Clock>::Duration {
            self.watchdog.lock().await.borrow().config.check_interval
        }

        /// Check if any tasks have starved
        pub async fn check_tasks(&self) -> bool {
            self.watchdog.lock().await.borrow_mut().check()
        }
    }

    #[cfg(not(feature = "alloc"))]
    impl<I, const N: usize> WatchdogRunner<I, N>
    where
        I: Id,
    {
        /// Create a new Embassy-compatible watchdog runner.
        pub fn new(hw_watchdog: P_RpWatchdog, config: WatchdogConfig<EmbassyClock>) -> Self {
            let hw_watchdog = RpWatchdog::new(hw_watchdog);
            let watchdog = Watchdog::new(hw_watchdog, config, EmbassyClock);
            Self {
                watchdog: embassy_sync::mutex::Mutex::new(core::cell::RefCell::new(watchdog)),
            }
        }

        /// Register a task with the watchdog.
        pub async fn register_task(&self, id: &I, max_duration: <EmbassyClock as Clock>::Duration) {
            self.watchdog
                .lock()
                .await
                .borrow_mut()
                .register_task(id, max_duration)
                .ok();
        }

        /// Deregister a task with the watchdog.
        pub async fn deregister_task(&self, id: &I) {
            self.watchdog.lock().await.borrow_mut().deregister_task(id);
        }

        /// Feed the watchdog for a specific task.
        pub async fn feed(&self, id: &I) {
            self.watchdog.lock().await.borrow_mut().feed(id);
        }

        /// Start the watchdog.
        pub async fn start(&self) {
            self.watchdog.lock().await.borrow_mut().start();
        }

        /// Trigger a system reset.
        pub async fn trigger_reset(&self) -> ! {
            self.watchdog.lock().await.borrow_mut().trigger_reset()
        }

        /// Get the last reset reason.
        pub async fn reset_reason(&self) -> Option<ResetReason> {
            self.watchdog.lock().await.borrow().reset_reason()
        }

        /// Get the check interval
        pub async fn get_check_interval(&self) -> <EmbassyClock as Clock>::Duration {
            self.watchdog.lock().await.borrow().config.check_interval
        }

        /// Check if any tasks have starved
        pub async fn check_tasks(&self) -> bool {
            self.watchdog.lock().await.borrow_mut().check()
        }
    }

    // For alloc feature
    #[cfg(feature = "alloc")]
    pub struct WatchdogTask<I>
    where
        I: 'static + Id,
    {
        runner: &'static WatchdogRunner<I>,
    }

    #[cfg(feature = "alloc")]
    impl<I> WatchdogRunner<I>
    where
        I: 'static + Id,
    {
        pub fn create_task(&'static self) -> WatchdogTask<I> {
            WatchdogTask { runner: self }
        }
    }

    #[cfg(feature = "alloc")]
    pub async fn watchdog_run<I>(task: WatchdogTask<I>) -> !
    where
        I: 'static + Id,
    {
        info!("Watchdog runner started");

        // Start the watchdog
        task.runner.start().await;

        // Get initial check interval
        let interval = task.runner.get_check_interval().await;
        let mut check_time = Instant::now() + interval;

        loop {
            // Check for starved tasks.  We don't do anything
            let _ = task.runner.check_tasks().await;

            // Wait before checking again
            Timer::at(check_time).await;
            check_time += interval;
        }
    }

    /// A version of the Watchdog Task when not using the `alloc`` feature.
    ///
    /// There is an equivalent version of this when using the `alloc` feature
    /// which does not include the `const N: usize` type.
    #[cfg(not(feature = "alloc"))]
    pub struct NoAllocWatchdogTask<I, const N: usize>
    where
        I: 'static + Id,
    {
        runner: &'static WatchdogRunner<I, N>,
    }

    #[cfg(not(feature = "alloc"))]
    impl<I, const N: usize> WatchdogRunner<I, N>
    where
        I: 'static + Id,
    {
        /// Used to create a watchdog task when not using the alloc feature.
        ///
        /// There is an equivalent version of this when using the `alloc` feature
        /// which does not include the `const N: usize` type.
        pub fn create_task(&'static self) -> NoAllocWatchdogTask<I, N> {
            NoAllocWatchdogTask { runner: self }
        }
    }

    /// Watchdog Runner function, which will monitor tasks and reset the
    /// system if any.
    ///
    /// You must call this function from an async task to start and run the
    /// watchdog.  Using `spawner.must_spawn(watchdog_run(watchdog))` would
    /// likely be a good choice.
    #[cfg(not(feature = "alloc"))]
    pub async fn watchdog_run<I, const N: usize>(task: NoAllocWatchdogTask<I, N>) -> !
    where
        I: 'static + Id,
    {
        info!("Watchdog runner started");

        // Start the watchdog
        task.runner.start().await;

        // Get initial check interval
        let interval = task.runner.get_check_interval().await;
        let mut check_time = Instant::now() + interval;

        loop {
            // Check for starved tasks.  We don't do anthing based on the
            // return code as check_tasks() handles feeding/starving the
            // hardware watchdog.
            let _ = task.runner.check_tasks().await;

            // Wait before checking again
            Timer::at(check_time).await;
            check_time += interval;
        }
    }
}

/// An async implementation of task-watchdog for use with the STM32 embassy
/// implementation.
///
/// This module requires the `stm32-embassy` feature.
#[cfg(feature = "stm32-embassy")]
pub mod embassy_stm32 {
    use super::{
        info, Clock, EmbassyClock, HardwareWatchdog, Id, ResetReason, Watchdog, WatchdogConfig,
    };
    use embassy_stm32::peripherals::IWDG;
    use embassy_stm32::wdg::IndependentWatchdog;
    use embassy_time::{Instant, Timer};

    /// STM32 specific watchdog implementation.
    pub struct Stm32Watchdog {
        peripheral: Option<IWDG>,
        inner: Option<IndependentWatchdog<'static, IWDG>>,
    }

    impl Stm32Watchdog {
        /// Create a new STM32 watchdog.
        #[must_use]
        pub fn new(peripheral: IWDG) -> Self {
            Self {
                peripheral: Some(peripheral),
                inner: None,
            }
        }
    }

    impl HardwareWatchdog<EmbassyClock> for Stm32Watchdog {
        fn start(&mut self, timeout: embassy_time::Duration) {
            let timeout = timeout.as_micros();
            if timeout > u32::MAX as u64 {
                panic!("Watchdog timeout too large for STM32");
            }
            let peripheral = self
                .peripheral
                .take()
                .expect("STM32 Watchdog not properly initialized");

            // Create the watchdog
            let mut wdg = IndependentWatchdog::new(peripheral, timeout as u32);

            // Start it
            wdg.unleash();

            // Store it
            self.inner = Some(wdg);
        }

        fn feed(&mut self) {
            self.inner.as_mut().expect("Watchdog not started").pet();
        }

        fn trigger_reset(&mut self) -> ! {
            cortex_m::peripheral::SCB::sys_reset();
        }

        fn reset_reason(&self) -> Option<ResetReason> {
            None
        }
    }

    /// An Embassy STM32 watchdog runner.
    #[cfg(feature = "alloc")]
    pub struct WatchdogRunner<I>
    where
        I: Id,
    {
        watchdog: embassy_sync::mutex::Mutex<
            embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
            core::cell::RefCell<Watchdog<I, Stm32Watchdog, EmbassyClock>>,
        >,
    }

    #[cfg(not(feature = "alloc"))]
    pub struct WatchdogRunner<I, const N: usize>
    where
        I: Id,
    {
        watchdog: embassy_sync::mutex::Mutex<
            embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
            core::cell::RefCell<Watchdog<I, N, Stm32Watchdog, EmbassyClock>>,
        >,
    }

    #[cfg(feature = "alloc")]
    impl<I> WatchdogRunner<I>
    where
        I: Id + 'static,
    {
        /// Create a new Embassy-compatible watchdog runner.
        pub fn new(hw_watchdog: IWDG, config: WatchdogConfig<EmbassyClock>) -> Self {
            let hw_watchdog = Stm32Watchdog::new(hw_watchdog);
            let watchdog = Watchdog::new(hw_watchdog, config, EmbassyClock);
            Self {
                watchdog: embassy_sync::mutex::Mutex::new(core::cell::RefCell::new(watchdog)),
            }
        }

        /// Register a task with the watchdog.
        pub async fn register_task(&self, id: &I, max_duration: <EmbassyClock as Clock>::Duration) {
            self.watchdog
                .lock()
                .await
                .borrow_mut()
                .register_task(id, max_duration);
        }

        /// De-register a task with the watchdog.
        pub async fn deregister_task(&self, id: &I) {
            self.watchdog.lock().await.borrow_mut().deregister_task(id);
        }

        /// Feed the watchdog for a specific task.
        pub async fn feed(&self, id: &I) {
            self.watchdog.lock().await.borrow_mut().feed(id);
        }

        /// Start the watchdog.
        pub async fn start(&self) {
            self.watchdog.lock().await.borrow_mut().start();
        }

        /// Trigger a system reset.
        pub async fn trigger_reset(&self) -> ! {
            self.watchdog.lock().await.borrow_mut().trigger_reset()
        }

        /// Get the last reset reason.
        pub async fn reset_reason(&self) -> Option<ResetReason> {
            self.watchdog.lock().await.borrow().reset_reason()
        }

        /// Get the check interval
        pub async fn get_check_interval(&self) -> <EmbassyClock as Clock>::Duration {
            self.watchdog.lock().await.borrow().config.check_interval
        }

        /// Check if any tasks have starved
        pub async fn check_tasks(&self) -> bool {
            self.watchdog.lock().await.borrow_mut().check()
        }
    }

    #[cfg(not(feature = "alloc"))]
    impl<I, const N: usize> WatchdogRunner<I, N>
    where
        I: Id,
    {
        /// Create a new Embassy-compatible watchdog runner.
        pub fn new(hw_watchdog: IWDG, config: WatchdogConfig<EmbassyClock>) -> Self {
            let hw_watchdog = Stm32Watchdog::new(hw_watchdog);
            let watchdog = Watchdog::new(hw_watchdog, config, EmbassyClock);
            Self {
                watchdog: embassy_sync::mutex::Mutex::new(core::cell::RefCell::new(watchdog)),
            }
        }

        /// Register a task with the watchdog.
        pub async fn register_task(&self, id: &I, max_duration: <EmbassyClock as Clock>::Duration) {
            self.watchdog
                .lock()
                .await
                .borrow_mut()
                .register_task(id, max_duration)
                .ok();
        }

        /// Deregister a task with the watchdog.
        pub async fn deregister_task(&self, id: &I) {
            self.watchdog.lock().await.borrow_mut().deregister_task(id);
        }

        /// Feed the watchdog for a specific task.
        pub async fn feed(&self, id: &I) {
            self.watchdog.lock().await.borrow_mut().feed(id);
        }

        /// Start the watchdog.
        pub async fn start(&self) {
            self.watchdog.lock().await.borrow_mut().start();
        }

        /// Trigger a system reset.
        pub async fn trigger_reset(&self) -> ! {
            self.watchdog.lock().await.borrow_mut().trigger_reset()
        }

        /// Get the last reset reason.
        pub async fn reset_reason(&self) -> Option<ResetReason> {
            self.watchdog.lock().await.borrow().reset_reason()
        }

        /// Get the check interval
        pub async fn get_check_interval(&self) -> <EmbassyClock as Clock>::Duration {
            self.watchdog.lock().await.borrow().config.check_interval
        }

        /// Check if any tasks have starved
        pub async fn check_tasks(&self) -> bool {
            self.watchdog.lock().await.borrow_mut().check()
        }
    }

    // For alloc feature
    #[cfg(feature = "alloc")]
    pub struct WatchdogTask<I>
    where
        I: 'static + Id,
    {
        runner: &'static WatchdogRunner<I>,
    }

    #[cfg(feature = "alloc")]
    impl<I> WatchdogRunner<I>
    where
        I: 'static + Id,
    {
        pub fn create_task(&'static self) -> WatchdogTask<I> {
            WatchdogTask { runner: self }
        }
    }

    /// Watchdog Runner function, which will monitor tasks and reset the
    /// system if any.  The user must call this function from an async task
    /// to start and run the watchdog.
    #[cfg(feature = "alloc")]
    pub async fn watchdog_run<I>(task: WatchdogTask<I>) -> !
    where
        I: 'static + Id,
    {
        info!("Watchdog runner started");

        // Start the watchdog
        task.runner.start().await;

        // Get initial check interval
        let interval = task.runner.get_check_interval().await;
        let mut check_time = Instant::now() + interval;

        loop {
            // Check for starved tasks.  We don't do anthing based on the
            // return code as check_tasks() handles feeding/starving the
            // hardware watchdog.
            let _ = task.runner.check_tasks().await;

            // Wait before checking again
            Timer::at(check_time).await;
            check_time += interval;
        }
    }

    // For no_alloc feature
    #[cfg(not(feature = "alloc"))]
    pub struct NoAllocWatchdogTask<I, const N: usize>
    where
        I: 'static + Id,
    {
        runner: &'static WatchdogRunner<I, N>,
    }

    #[cfg(not(feature = "alloc"))]
    impl<I, const N: usize> WatchdogRunner<I, N>
    where
        I: 'static + Id,
    {
        pub fn create_task(&'static self) -> NoAllocWatchdogTask<I, N> {
            NoAllocWatchdogTask { runner: self }
        }
    }

    /// Watchdog Runner, which will monitor tasks and reset the system if any
    /// registered task fails to feed the watchdog.
    #[cfg(not(feature = "alloc"))]
    pub async fn watchdog_run<I, const N: usize>(task: NoAllocWatchdogTask<I, N>) -> !
    where
        I: 'static + Id,
    {
        info!("Watchdog runner started");

        // Start the watchdog
        task.runner.start().await;

        // Get initial check interval
        let interval = task.runner.get_check_interval().await;
        let mut check_time = Instant::now() + interval;

        loop {
            // Check for starved tasks.  We don't do anthing based on the
            // return code as check_tasks() handles feeding/starving the
            // hardware watchdog.
            let _ = task.runner.check_tasks().await;

            // Wait before checking again
            Timer::at(check_time).await;
            check_time += interval;
        }
    }
}

/// An async implementation of task-watchdog for use with the ESP32 embassy
/// implementation.
///
/// This module requires the `esp32-embassy` feature.
#[cfg(feature = "esp32-embassy")]
pub mod embassy_esp32 {
    use super::{
        info, Clock, EmbassyClock, HardwareWatchdog, Id, ResetReason, Watchdog, WatchdogConfig,
    };
    use embassy_time::{Instant, Timer};
    use esp_hal::peripherals::TIMG0;
    use esp_hal::timer::timg::MwdtStage;
    use esp_hal::timer::timg::TimerGroup;
    use esp_hal::timer::timg::Wdt;

    /// ESP32 specific watchdog implementation.
    pub struct Esp32Watchdog {
        inner: Wdt<TIMG0>,
    }

    impl Esp32Watchdog {
        /// Create a new ESP32 watchdog.
        ///
        /// Arguments:
        /// - `timg0` - The TimerGroup to use for the watchdog.
        #[must_use]
        pub fn new(timg0: TimerGroup<TIMG0>) -> Self {
            let wdt = timg0.wdt;
            Self { inner: wdt }
        }
    }

    impl HardwareWatchdog<EmbassyClock> for Esp32Watchdog {
        fn start(&mut self, timeout: embassy_time::Duration) {
            self.inner.set_timeout(
                MwdtStage::Stage0,
                esp_hal::time::Duration::from_millis(timeout.as_millis()),
            );
            self.inner.enable();
        }

        fn feed(&mut self) {
            self.inner.feed();
        }

        fn trigger_reset(&mut self) -> ! {
            esp_hal::system::software_reset();
        }

        fn reset_reason(&self) -> Option<ResetReason> {
            None
        }
    }

    /// An Embassy ESP32 watchdog runner.
    #[cfg(feature = "alloc")]
    pub struct WatchdogRunner<I>
    where
        I: Id,
    {
        watchdog: embassy_sync::mutex::Mutex<
            embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
            core::cell::RefCell<Watchdog<I, Esp32Watchdog, EmbassyClock>>,
        >,
    }

    #[cfg(not(feature = "alloc"))]
    pub struct WatchdogRunner<I, const N: usize>
    where
        I: Id,
    {
        watchdog: embassy_sync::mutex::Mutex<
            embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
            core::cell::RefCell<Watchdog<I, N, Esp32Watchdog, EmbassyClock>>,
        >,
    }

    #[cfg(feature = "alloc")]
    impl<I> WatchdogRunner<I>
    where
        I: Id + 'static,
    {
        /// Create a new Embassy-compatible watchdog runner.
        pub fn new(timg0: TimerGroup<TIMG0>, config: WatchdogConfig<EmbassyClock>) -> Self {
            let hw_watchdog = Esp32Watchdog::new(timg0);
            let watchdog = Watchdog::new(hw_watchdog, config, EmbassyClock);
            Self {
                watchdog: embassy_sync::mutex::Mutex::new(core::cell::RefCell::new(watchdog)),
            }
        }

        /// Register a task with the watchdog.
        pub async fn register_task(&self, id: &I, max_duration: <EmbassyClock as Clock>::Duration) {
            self.watchdog
                .lock()
                .await
                .borrow_mut()
                .register_task(id, max_duration);
        }

        /// De-register a task with the watchdog.
        pub async fn deregister_task(&self, id: &I) {
            self.watchdog.lock().await.borrow_mut().deregister_task(id);
        }

        /// Feed the watchdog for a specific task.
        pub async fn feed(&self, id: &I) {
            self.watchdog.lock().await.borrow_mut().feed(id);
        }

        /// Start the watchdog.
        pub async fn start(&self) {
            self.watchdog.lock().await.borrow_mut().start();
        }

        /// Trigger a system reset.
        pub async fn trigger_reset(&self) -> ! {
            self.watchdog.lock().await.borrow_mut().trigger_reset()
        }

        /// Get the last reset reason.
        pub async fn reset_reason(&self) -> Option<ResetReason> {
            self.watchdog.lock().await.borrow().reset_reason()
        }

        /// Get the check interval
        pub async fn get_check_interval(&self) -> <EmbassyClock as Clock>::Duration {
            self.watchdog.lock().await.borrow().config.check_interval
        }

        /// Check if any tasks have starved
        pub async fn check_tasks(&self) -> bool {
            self.watchdog.lock().await.borrow_mut().check()
        }
    }

    #[cfg(not(feature = "alloc"))]
    impl<I, const N: usize> WatchdogRunner<I, N>
    where
        I: Id,
    {
        /// Create a new Embassy-compatible watchdog runner.
        pub fn new(timg0: TimerGroup<TIMG0>, config: WatchdogConfig<EmbassyClock>) -> Self {
            let hw_watchdog = Esp32Watchdog::new(timg0);
            let watchdog = Watchdog::new(hw_watchdog, config, EmbassyClock);
            Self {
                watchdog: embassy_sync::mutex::Mutex::new(core::cell::RefCell::new(watchdog)),
            }
        }

        /// Register a task with the watchdog.
        pub async fn register_task(&self, id: &I, max_duration: <EmbassyClock as Clock>::Duration) {
            self.watchdog
                .lock()
                .await
                .borrow_mut()
                .register_task(id, max_duration)
                .ok();
        }

        /// Deregister a task with the watchdog.
        pub async fn deregister_task(&self, id: &I) {
            self.watchdog.lock().await.borrow_mut().deregister_task(id);
        }

        /// Feed the watchdog for a specific task.
        pub async fn feed(&self, id: &I) {
            self.watchdog.lock().await.borrow_mut().feed(id);
        }

        /// Start the watchdog.
        pub async fn start(&self) {
            self.watchdog.lock().await.borrow_mut().start();
        }

        /// Trigger a system reset.
        pub async fn trigger_reset(&self) -> ! {
            self.watchdog.lock().await.borrow_mut().trigger_reset()
        }

        /// Get the last reset reason.
        pub async fn reset_reason(&self) -> Option<ResetReason> {
            self.watchdog.lock().await.borrow().reset_reason()
        }

        /// Get the check interval
        pub async fn get_check_interval(&self) -> <EmbassyClock as Clock>::Duration {
            self.watchdog.lock().await.borrow().config.check_interval
        }

        /// Check if any tasks have starved
        pub async fn check_tasks(&self) -> bool {
            self.watchdog.lock().await.borrow_mut().check()
        }
    }

    // For alloc feature
    #[cfg(feature = "alloc")]
    pub struct WatchdogTask<I>
    where
        I: 'static + Id,
    {
        runner: &'static WatchdogRunner<I>,
    }

    #[cfg(feature = "alloc")]
    impl<I> WatchdogRunner<I>
    where
        I: 'static + Id,
    {
        pub fn create_task(&'static self) -> WatchdogTask<I> {
            WatchdogTask { runner: self }
        }
    }

    /// Watchdog Runner function, which will monitor tasks and reset the
    /// system if any.  The user must call this function from an async task
    /// to start and run the watchdog.
    #[cfg(feature = "alloc")]
    pub async fn watchdog_run<I>(task: WatchdogTask<I>) -> !
    where
        I: 'static + Id,
    {
        info!("Watchdog runner started");

        // Start the watchdog
        task.runner.start().await;

        // Get initial check interval
        let interval = task.runner.get_check_interval().await;
        let mut check_time = Instant::now() + interval;

        loop {
            // Check for starved tasks.  We don't do anthing based on the
            // return code as check_tasks() handles feeding/starving the
            // hardware watchdog.
            let _ = task.runner.check_tasks().await;

            // Wait before checking again
            Timer::at(check_time).await;
            check_time += interval;
        }
    }

    // For no_alloc feature
    #[cfg(not(feature = "alloc"))]
    pub struct NoAllocWatchdogTask<I, const N: usize>
    where
        I: 'static + Id,
    {
        runner: &'static WatchdogRunner<I, N>,
    }

    #[cfg(not(feature = "alloc"))]
    impl<I, const N: usize> WatchdogRunner<I, N>
    where
        I: 'static + Id,
    {
        pub fn create_task(&'static self) -> NoAllocWatchdogTask<I, N> {
            NoAllocWatchdogTask { runner: self }
        }
    }

    /// Watchdog Runner, which will monitor tasks and reset the system if any
    /// registered task fails to feed the watchdog.
    #[cfg(not(feature = "alloc"))]
    pub async fn watchdog_run<I, const N: usize>(task: NoAllocWatchdogTask<I, N>) -> !
    where
        I: 'static + Id,
    {
        info!("Watchdog runner started");

        // Start the watchdog
        task.runner.start().await;

        // Get initial check interval
        let interval = task.runner.get_check_interval().await;
        let mut check_time = Instant::now() + interval;

        loop {
            // Check for starved tasks.  We don't do anthing based on the
            // return code as check_tasks() handles feeding/starving the
            // hardware watchdog.
            let _ = task.runner.check_tasks().await;

            // Wait before checking again
            Timer::at(check_time).await;
            check_time += interval;
        }
    }
}
