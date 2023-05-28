use crate::{DelayMs, Scheduler};
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use embedded_hal::timer::CountDown;
use fugit::MillisDurationU32;

use void::Void;

pub struct Timer<T, S> {
    timer: T,
    is_started: bool,
    scheduler: S,
}

impl<T, S> Timer<T, S> {
    pub const fn new(timer: T, scheduler: S) -> Self {
        Self {
            timer,
            scheduler,
            is_started: false,
        }
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
            Err(nb::Error::WouldBlock) => {
                self.scheduler.schedule(cx.waker());
                Poll::Pending
            }
        }
    }
}
