use crate::{DelayMs, Scheduler};
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use embedded_hal::timer::{Cancel, CountDown};
use fugit::MillisDurationU32;

pub struct Timer<T, S> {
    timer: T,
    scheduler: S,
}

impl<T, S> Timer<T, S> {
    pub const fn new(timer: T, scheduler: S) -> Self {
        Self { timer, scheduler }
    }
}

impl<T, S> DelayMs for Timer<T, S>
where
    T: CountDown + Cancel + Unpin,
    T::Time: From<MillisDurationU32>,
    S: Scheduler + Unpin,
{
    type Delay = u32;
    type Error = T::Error;

    fn start(&mut self, ms: Self::Delay) -> Result<(), Self::Error> {
        self.timer.start(MillisDurationU32::millis(ms));
        Ok(())
    }

    fn cancel(&mut self) -> Result<(), Self::Error> {
        self.timer.cancel()
    }

    fn poll_delay_ms(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        match self.timer.wait() {
            Ok(()) => Poll::Ready(Ok(())),
            Err(nb::Error::Other(_void)) => unreachable!(),
            Err(nb::Error::WouldBlock) => {
                self.scheduler.schedule(cx.waker());
                Poll::Pending
            }
        }
    }
}
