mod transmit;
use std::task::Waker;

use futures::task::AtomicWaker;
pub use transmit::{MockFrame, MockTransmit, Transmit, Transmitter};

mod receive;
pub use receive::{DualReceiver, Receive, Receiver};

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