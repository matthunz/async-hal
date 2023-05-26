#![cfg_attr(not(feature = "mock"), no_std)]

use core::task::Waker;
use futures::task::AtomicWaker;

pub mod can;

pub mod executor;
pub use executor::Executor;

pub mod serial;

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
