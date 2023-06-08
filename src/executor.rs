use core::{
    cell::{OnceCell, RefCell},
    pin::Pin,
    task::{Context, Poll},
};
use futures::Future;
use noop_waker::noop_waker;

pub struct Executor<F> {
    future: OnceCell<RefCell<F>>,
}

impl<F> Executor<F> {
    /// Create a new empty executor.
    pub const fn new() -> Self {
        Self {
            future: OnceCell::new(),
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
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);
        pinned.poll(&mut cx)
    }
}
