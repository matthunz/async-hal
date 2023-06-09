#![cfg_attr(not(feature = "mock"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! Async hardware abstraction layer for embedded devices.
//! This crate provides zero-cost utilities for async IO with `#![no-std]`.
//!
//! Two execution models are provided:
//! - Interrupt mode: Multiple interrupts can each run a future using an [`Executor`].
//!   Each future is polled on every interrupt and channels can be used to communicate between them.
//!
//! - Thread mode: Interrupts wake a main function where a future is being polled with [`block_on`].
//!
//! ## Installation
//!
//! The easiest way to get started is to enable all features.
//!
//! ```toml
//! [dependencies]
//! async-hal = { version = "...", features = ["full"] }
//! ```
//!
//! Or by using `cargo add`
//! ```sh
//! cargo add async-hal --features full
//! ```
//!
//! ## Feature flags
//!
//! Async-hal uses a set of [feature flags] to reduce the amount of compiled code. It
//! is possible to just enable certain features over others. By default, async-hal
//! does not enable any features but allows one to enable a subset for their use
//! case. Below is a list of the available feature flags. You may also notice
//! above each function, struct and trait there is listed one or more feature flags
//! that are required for that item to be used. If you are new to async-hal it is
//! recommended that you use the `full` feature flag which will enable all public APIs.
//! Beware though that this will pull in many extra dependencies that you may not
//! need.
//!
//! [feature flags]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-features-section
//!
//! - `full`: Enables all features listed below except `mock` and `bxcan`.
//! - `can`: Enables the `async_hal::can` module.
//! - `delay`: Enables the `async_hal::delay` module.
//! - `executor`: Enables the `async_hal::executor` module.
//! - `io`: Enables the `async_hal::io` module.
//! - `serial`: Enables the `async_hal::serial` module.
//! - `nb`: Enables async wrappers for non-blocking interfaces (such as from `embedded_hal`).
//! - `bxcan`: Enables CAN support for stm32 devices with [`bxcan`](https://docs.rs/bxcan/).

use core::task::{Context, Poll};
use futures::{task::noop_waker, Future, FutureExt};

#[cfg_attr(docsrs, doc(cfg(feature = "can")))]
#[cfg(feature = "can")]
/// CAN bus
pub mod can;

#[cfg_attr(docsrs, doc(cfg(feature = "executor")))]
#[cfg(feature = "executor")]
/// Task executor
pub mod executor;
#[cfg(feature = "executor")]
pub use executor::Executor;

/// Interrupt stream
mod interrupt;
pub use interrupt::Interrupt;

#[cfg_attr(docsrs, doc(cfg(feature = "io")))]
#[cfg(feature = "io")]
/// Asynchronous IO
pub mod io;

#[cfg_attr(docsrs, doc(cfg(feature = "serial")))]
#[cfg(feature = "serial")]
/// Serial port
pub mod serial;

#[cfg_attr(docsrs, doc(cfg(feature = "delay")))]
#[cfg(feature = "delay")]
/// Delay timers
pub mod delay;

/// Run `future` to completion and return its output.
/// This will repeatedly poll the future and call `wait()`.
///
/// This is useful for microcontrollers that can be set into a low-power mode while waiting,
/// such as using Cortex-M's `wfi` instruction.
/// ```
/// use futures::pin_mut;
///
/// let task = async { true };
/// pin_mut!(task);
///
/// let output = async_hal::block_on(task, || {
///     dbg!("Waiting!");
/// });
/// assert!(output);
/// ```
pub fn block_on<F, W>(mut future: F, mut wait: W) -> F::Output
where
    F: Future + Unpin,
    W: FnMut(),
{
    let waker = noop_waker();

    loop {
        let mut cx = Context::from_waker(&waker);
        if let Poll::Ready(output) = future.poll_unpin(&mut cx) {
            return output;
        }

        wait()
    }
}
