#![cfg_attr(not(feature = "mock"), no_std)]

//! Async hardware abstraction layer for embedded devices
use core::{
    pin::Pin,
    task::{Context, Poll, Waker},
};
use futures::{task::AtomicWaker, Stream};

/// CAN bus
pub mod can;

/// Task executor
pub mod executor;
pub use executor::Executor;

/// UART serial port
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

pub struct Interrupt<S> {
    scheduler: S,
    is_waiting: bool,
}

impl<S> Interrupt<S> {
    pub const fn new(scheduler: S) -> Self {
        Self {
            scheduler,
            is_waiting: false,
        }
    }
}

impl<S> Stream for Interrupt<S>
where
    S: Scheduler + Unpin,
{
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        if self.is_waiting {
            self.is_waiting = false;
            Poll::Ready(Some(()))
        } else {
            self.is_waiting = true;
            self.scheduler.schedule(cx.waker());
            Poll::Pending
        }
    }
}
