use core::{
    array,
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicUsize, Ordering},
};

pub struct Executor<T, const N: usize> {
    tasks: [UnsafeCell<MaybeUninit<T>>; N],
    set: AtomicUsize,
}

impl<T, const N: usize> Executor<T, N> {
    pub fn new() -> Self {
        let tasks = array::from_fn(|_| UnsafeCell::new(MaybeUninit::uninit()));
        Self {
            tasks,
            set: AtomicUsize::new((1 << N) - 1),
        }
    }

    pub fn spawn(&self, task: T) -> Option<T> {
        let set = self.set.load(Ordering::SeqCst);
        let idx = set.trailing_zeros() as usize;

        if idx >= N {
            return Some(task);
        }

        let cell = unsafe { &mut *self.tasks[idx].get() };
        *cell = MaybeUninit::new(task);

        let mask = !(1 << idx);
        self.set.fetch_and(mask, Ordering::SeqCst);

        None
    }
}
