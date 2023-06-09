use core::{
    cell::RefCell,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};
use futures::Future;
use once_cell::unsync::OnceCell;

pub trait Interrupt {
    /// Pend this interrupt handler to run.
    fn pend(&self);
}

pub struct NonPending;

impl Interrupt for NonPending {
    fn pend(&self) {}
}

/// Task executor for a single `'static` future.
///
/// This provides a polling interface for a [`Future`] running on an interrupt handler.
/// If the provided [`Waker`] is woken, the executor will pend the provided interrupt with [`Interrupt::pend`].
///
/// Executors must be static for use with the waker and [`Pin`] support.
/// ```
/// use async_hal::Executor;
///
/// static mut EXECUTOR: Executor<(), ()> = Executor::new(());
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "executor")))]
pub struct Executor<I, F> {
    interrupt: I,
    future: OnceCell<RefCell<F>>,
}

impl<I, F> Executor<I, F> {
    /// Create a new empty executor.
    pub const fn new(interrupt: I) -> Self {
        Self {
            future: OnceCell::new(),
            interrupt,
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
        I: Interrupt,
        F: Future,
    {
        static VTABLE: RawWakerVTable = RawWakerVTable::new(
            |ptr| RawWaker::new(ptr, &VTABLE),
            |ptr| {
                let me = unsafe { *ptr.cast::<&dyn Interrupt>() };
                me.pend();
            },
            |ptr| {
                let me = unsafe { *ptr.cast::<&dyn Interrupt>() };
                me.pend();
            },
            |_| {},
        );
        let raw_waker = RawWaker::new(self as *const dyn Interrupt as *const (), &VTABLE);
        let waker = unsafe { Waker::from_raw(raw_waker) };
        let mut cx = Context::from_waker(&waker);

        let mut future = self.future.get().unwrap().borrow_mut();

        // Safety: `future` is guranteed to be static
        let pinned = unsafe { Pin::new_unchecked(&mut *future) };
        pinned.poll(&mut cx)
    }
}

impl<F> Executor<NonPending, F> {
    pub const fn non_pending() -> Self {
        Self::new(NonPending)
    }
}

impl<I: Interrupt, F> Interrupt for Executor<I, F> {
    fn pend(&self) {
        self.interrupt.pend()
    }
}
