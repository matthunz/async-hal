use core::{
    cell::RefCell,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};
use futures::Future;
use once_cell::unsync::OnceCell;

pub trait Interrupt {
    fn pend(&self);
}

/// Task executor for a single `'static` future.
pub struct Executor<F> {
    future: OnceCell<RefCell<F>>,
    interrupt: &'static dyn Interrupt,
}

impl<F> Executor<F> {
    /// Create a new empty executor.
    pub const fn new(interrupt: &'static impl Interrupt) -> Self {
        Self {
            future: OnceCell::new(),
            interrupt: interrupt,
        }
    }

    /// Spawn a single [`Future`] on the executor.
    /// This method returns Ok(()) if the executor was empty and Err(value) if it was full.
    pub fn spawn(&self, future: F) -> Result<(), F> {
        self.future
            .set(RefCell::new(future))
            .map_err(|cell| cell.into_inner())
    }

    /// Poll the current [`Future`] on the executor.
    pub fn poll(&'static self) -> Poll<F::Output>
    where
        F: Future,
    {
        let mut future = self.future.get().unwrap().borrow_mut();

        // Safety: `future` is guranteed to be static
        let pinned = unsafe { Pin::new_unchecked(&mut *future) };

        // TODO pend interrupt on wakeup
        static VTABLE: RawWakerVTable = RawWakerVTable::new(
            |ptr| RawWaker::new(ptr, &VTABLE),
            |ptr| {
                let interrupt = unsafe { *ptr.cast::<&dyn Interrupt>() };
                interrupt.pend();
            },
            |ptr| {
                let interrupt = unsafe { *ptr.cast::<&dyn Interrupt>() };
                interrupt.pend();
            },
            |_| {},
        );

        let raw_waker = RawWaker::new(self.interrupt as *const dyn Interrupt as *const (), &VTABLE);
        let waker = unsafe { Waker::from_raw(raw_waker) };

        let mut cx = Context::from_waker(&waker);
        pinned.poll(&mut cx)
    }
}
