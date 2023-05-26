use core::{
    array,
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicUsize, Ordering},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};
use futures::{Future, FutureExt};

static READY: AtomicUsize = AtomicUsize::new(0);

pub struct Executor<T, const N: usize> {
    tasks: [UnsafeCell<MaybeUninit<T>>; N],
    set: AtomicUsize,
    locked: AtomicUsize,
}

impl<T, const N: usize> Executor<T, N> {
    pub fn new() -> Self {
        let tasks = array::from_fn(|_| UnsafeCell::new(MaybeUninit::uninit()));
        Self {
            tasks,
            set: AtomicUsize::new((1 << N) - 1),
            locked: AtomicUsize::new(0),
        }
    }

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

    // TODO this is unsafe if multiple interrupts use `run`.
    pub fn run(&self) -> Option<T::Output>
    where
        T: Future + Unpin,
    {
        loop {
            let ready = READY.load(Ordering::SeqCst);

            if ready == 0 {
                break None;
            }

            let idx = ready.trailing_zeros() as usize;
            if (self.locked.load(Ordering::SeqCst) & (1 << idx)) != 0 {
                continue;
            }

            let mask = !(1 << idx);
            READY.fetch_and(mask, Ordering::SeqCst);

            let cell = unsafe { &mut *self.tasks[idx].get() };
            let task = unsafe { cell.assume_init_mut() };

            static VTABLE: RawWakerVTable =
                RawWakerVTable::new(|_| todo!(), |_| {}, |_| {}, |_| {});
            let raw_waker = RawWaker::new(&(), &VTABLE);
            let waker = unsafe { Waker::from_raw(raw_waker) };
            let mut cx = Context::from_waker(&waker);

            if let Poll::Ready(output) = task.poll_unpin(&mut cx) {
                *cell = MaybeUninit::uninit();
                let mask = !(1 << idx);
                self.set.fetch_and(mask, Ordering::SeqCst);

                break Some(output);
            }
        }
    }
}
