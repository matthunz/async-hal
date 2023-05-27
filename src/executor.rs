use core::{
    array,
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};
use futures::{Future, FutureExt};

static READY: AtomicUsize = AtomicUsize::new(0);
static IS_LOCKED: AtomicBool = AtomicBool::new(false);

/// Task executor singleton
pub struct Executor<T, const N: usize> {
    tasks: [UnsafeCell<MaybeUninit<T>>; N],
    set: AtomicUsize,
    locked: AtomicUsize,
}

impl<T, const N: usize> Executor<T, N> {
    /// ```
    /// use async_hal::Executor;
    ///
    /// let executor = Executor::<(), 1>::take().unwrap();
    ///
    /// // Only one executor can exist at a time
    /// assert!(Executor::<(), 2>::take().is_none());
    ///
    /// // After dropping the executor, we can create a new one
    /// drop(executor);
    /// assert!(Executor::<(), 2>::take().is_some());
    /// ```
    pub fn take() -> Option<Self> {
        if IS_LOCKED.swap(true, Ordering::SeqCst) {
            return None;
        }

        let tasks = array::from_fn(|_| UnsafeCell::new(MaybeUninit::uninit()));
        Some(Self {
            tasks,
            set: AtomicUsize::new((1 << N) - 1),
            locked: AtomicUsize::new(0),
        })
    }

    /// Spawn a task on the executor.
    pub fn spawn(&self, task: T) -> Option<T> {
        let set = self.set.load(Ordering::SeqCst);
        let idx = set.trailing_zeros() as usize;

        if idx >= N {
            return Some(task);
        }

        self.locked.fetch_or(1 << idx, Ordering::SeqCst);

        let cell = unsafe { &mut *self.tasks[idx].get() };
        *cell = MaybeUninit::new(task);

        let mask = !(1 << idx);
        self.set.fetch_and(mask, Ordering::SeqCst);
        self.locked.fetch_and(mask, Ordering::SeqCst);

        READY.fetch_or(1 << idx, Ordering::SeqCst);

        None
    }

    /// Poll each pending task, returning the output of the first that's ready.
    /// If none are ready, this function returns `None`.
    /// Otherwise this method should be called until no tasks are left pending.
    pub fn run(&mut self) -> Option<T::Output>
    where
        T: Future + Unpin,
    {
        // Safety: we have guarenteed unique access with `&mut self`
        unsafe { self.run_unchecked() }
    }

    // Safety: this can only be called by one interrupt function at a time
    unsafe fn run_unchecked(&self) -> Option<T::Output>
    where
        T: Future + Unpin,
    {
        loop {
            // Check if any tasks are ready
            let ready = READY.load(Ordering::SeqCst);
            if ready == 0 {
                break None;
            }

            // Get the index of the first ready task
            let idx = ready.trailing_zeros() as usize;
            if (self.locked.load(Ordering::SeqCst) & (1 << idx)) != 0 {
                continue;
            }

            // Clear the pending bit for this task
            let mask = !(1 << idx);
            READY.fetch_and(mask, Ordering::SeqCst);

            // Create the task waker and context
            static VTABLE: RawWakerVTable =
                RawWakerVTable::new(|data| RawWaker::new(data, &VTABLE), wake, wake, |_| {});
            let raw_waker = RawWaker::new(idx as *const (), &VTABLE);
            let waker = Waker::from_raw(raw_waker);
            let mut cx = Context::from_waker(&waker);

            // Poll the current task
            let cell = &mut *self.tasks[idx].get();
            let task = cell.assume_init_mut();

            if let Poll::Ready(output) = task.poll_unpin(&mut cx) {
                *cell = MaybeUninit::uninit();
                let mask = !(1 << idx);
                self.set.fetch_and(mask, Ordering::SeqCst);

                break Some(output);
            }
        }
    }

    pub fn split(&mut self) -> (Spawner<T, N>, Runner<T, N>) {
        (Spawner { executor: self }, Runner { executor: self })
    }
}

impl<T, const N: usize> Drop for Executor<T, N> {
    fn drop(&mut self) {
        IS_LOCKED.store(false, Ordering::SeqCst);
    }
}

fn wake(data: *const ()) {
    // Set this task's pending bit
    let idx = data as usize;
    READY.fetch_or(1 << idx, Ordering::SeqCst);
}

pub struct Runner<'a, T, const N: usize> {
    executor: &'a Executor<T, N>,
}

impl<T, const N: usize> Runner<'_, T, N> {
    pub fn run(&mut self) -> Option<T::Output>
    where
        T: Future + Unpin,
    {
        // Safety: we have guarenteed unique access with `&mut self`
        unsafe { self.executor.run_unchecked() }
    }
}

pub struct Spawner<'a, T, const N: usize> {
    executor: &'a Executor<T, N>,
}

impl<T, const N: usize> Spawner<'_, T, N> {
    pub fn spawn(&self, task: T) -> Option<T> {
        self.executor.spawn(task)
    }
}
