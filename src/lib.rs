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
