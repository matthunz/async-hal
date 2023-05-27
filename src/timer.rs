use embedded_hal::timer::CountDown;
use futures::StreamExt;
use crate::{Interrupt, Scheduler};

pub struct Timer<T, S> {
    timer: T,
    interrupt: Interrupt<S>,
}

impl<T, S> Timer<T, S> {
    pub const fn new(timer: T, scheduler: S) -> Self {
        let interrupt = Interrupt::new(scheduler);
        Self { timer, interrupt }
    }

    pub async fn wait<C>(&mut self, count: C)
    where
        C: Into<T::Time>,
        T: CountDown,
        S: Scheduler + Unpin,
    {
        self.timer.start(count);

        loop {
            match self.timer.wait() {
                Ok(()) => break,
                Err(nb::Error::Other(_void)) => unreachable!(),
                Err(nb::Error::WouldBlock) => {
                    self.interrupt.next().await;
                }
            }
        }
    }
}
