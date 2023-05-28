use crate::{DelayMs, Interrupt, Scheduler};
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use embedded_hal::timer::CountDown;
use fugit::MillisDurationU32;
use futures::{future, StreamExt};
use void::Void;

pub struct Timer<T, S> {
    timer: T,
    is_started: bool,
    interrupt: Interrupt<S>,
}

impl<T, S> Timer<T, S> {
    pub const fn new(timer: T, scheduler: S) -> Self {
        let interrupt = Interrupt::new(scheduler);
        Self {
            timer,
            interrupt,
            is_started: false,
        }
    }

    pub fn poll_wait<C>(&mut self, cx: &mut Context, count: C) -> Poll<()>
    where
        C: Into<T::Time>,
        T: CountDown,
        S: Scheduler + Unpin,
    {
        if !self.is_started {
            self.is_started = true;
            self.timer.start(count);
        }

        match self.timer.wait() {
            Ok(()) => {
                self.is_started = false;
                Poll::Ready(())
            }
            Err(nb::Error::Other(_void)) => unreachable!(),
            Err(nb::Error::WouldBlock) => self.interrupt.poll_next_unpin(cx).map(|_| ()),
        }
    }

    pub async fn wait<C>(&mut self, count: C)
    where
        C: Into<T::Time> + Clone,
        T: CountDown,
        S: Scheduler + Unpin,
    {
        future::poll_fn(|cx| self.poll_wait(cx, count.clone())).await;
    }
}

impl<T, S> DelayMs for Timer<T, S>
where
    T: CountDown + Unpin,
    T::Time: From<MillisDurationU32>,
    S: Scheduler + Unpin,
{
    type Error = Void;

    fn poll_delay_ms(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        ms: u32,
    ) -> Poll<Result<(), Self::Error>> {
        if !self.is_started {
            self.is_started = true;
            self.timer.start(MillisDurationU32::millis(ms));
        }

        match self.timer.wait() {
            Ok(()) => {
                self.is_started = false;
                Poll::Ready(Ok(()))
            }
            Err(nb::Error::Other(_void)) => unreachable!(),
            Err(nb::Error::WouldBlock) => self.interrupt.poll_next_unpin(cx).map(|_| Ok(())),
        }
    }
}
