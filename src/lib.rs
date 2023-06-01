#![cfg_attr(not(feature = "mock"), no_std)]

//! Async hardware abstraction layer for embedded devices
use core::task::{Context, Poll, Waker};
use futures::{
    task::{noop_waker, AtomicWaker},
    Future, FutureExt,
};

/// CAN bus
pub mod can;

/// Task executor
pub mod executor;
pub use executor::Executor;

/// Interrupt stream
mod interrupt;
pub use interrupt::Interrupt;

/// Asynchronous IO
pub mod io;

/// Serial port
pub mod serial;

/// Timers
pub mod delay;
pub use delay::DelayMs;

pub trait Scheduler {
    fn schedule(&self, waker: &Waker);
}

impl Scheduler for AtomicWaker {
    fn schedule(&self, waker: &Waker) {
        self.register(waker)
    }
}

impl<T: Scheduler> Scheduler for &'_ T {
    fn schedule(&self, waker: &Waker) {
        (*self).schedule(waker)
    }
}

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
