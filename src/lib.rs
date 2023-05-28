#![cfg_attr(not(feature = "mock"), no_std)]

//! Async hardware abstraction layer for embedded devices
use core::task::Waker;
use futures::task::AtomicWaker;

/// CAN bus
pub mod can;

/// Task executor
pub mod executor;
pub use executor::Executor;

mod interrupt;
pub use interrupt::Interrupt;

/// UART serial port
pub mod serial;

/// Timers
pub mod delay;
pub use delay::{DelayMs, Timer};

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
