use core::task::Waker;
use futures::task::AtomicWaker;

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

pub struct Ready;

impl Scheduler for Ready {
    fn schedule(&self, _waker: &Waker) {}
}
