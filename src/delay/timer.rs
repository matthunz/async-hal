use super::DelayMs;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use embedded_hal::timer::{Cancel, CountDown};
use fugit::MillisDurationU32;

/// Async wrapper for a non-blocking countdown timer.
pub struct Timer<T> {
    counter: T,
}

impl<T> Timer<T> {
    /// Create a new timer from `counter`.
    pub const fn new(counter: T) -> Self {
        Self { counter }
    }
}

impl<T> DelayMs for Timer<T>
where
    T: CountDown + Cancel + Unpin,
    T::Time: From<MillisDurationU32>,
{
    type Delay = u32;
    type Error = T::Error;

    fn start(&mut self, ms: Self::Delay) -> Result<(), Self::Error> {
        self.counter.start(MillisDurationU32::millis(ms));
        Ok(())
    }

    fn cancel(&mut self) -> Result<(), Self::Error> {
        self.counter.cancel()
    }

    fn poll_delay_ms(mut self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        match self.counter.wait() {
            Ok(()) => Poll::Ready(Ok(())),
            Err(nb::Error::Other(_void)) => unreachable!(),
            Err(nb::Error::WouldBlock) => Poll::Pending,
        }
    }
}
