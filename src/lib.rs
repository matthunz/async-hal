#![cfg_attr(not(feature = "mock"), no_std)]

use core::task::Waker;
use futures::task::AtomicWaker;

pub mod can;

pub trait Spawn {
    fn spawn(&self, waker: &Waker);
}

impl Spawn for AtomicWaker {
    fn spawn(&self, waker: &Waker) {
        self.register(waker)
    }
}

impl<T: Spawn> Spawn for &'_ T {
    fn spawn(&self, waker: &Waker) {
        (*self).spawn(waker)
    }
}
