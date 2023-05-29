use core::{
    pin::Pin,
    task::{Context, Poll},
};
use futures::{Future, FutureExt, Stream};

pub use embedded_hal::timer::Periodic;

#[cfg(feature = "nb")]
mod timer;
#[cfg(feature = "nb")]
pub use timer::Timer;

pub trait DelayMs {
    /// The type of duration to delay for.
    type Delay;

    /// The error returned on failure.
    type Error;

    /// Start a new delay.
    fn start(&mut self, ms: Self::Delay) -> Result<(), Self::Error>;

    /// Poll a delay of `ms` milliseconds.
    fn poll_delay_ms(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>>;

    /// Attempt to cancel a delay in progress.
    fn cancel(&mut self) -> Result<(), Self::Error>;

    fn poll_delay_ms_unpin(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>>
    where
        Self: Unpin,
    {
        Pin::new(self).poll_delay_ms(cx)
    }

    /// Delay for `ms` milliseconds.
    /// Starts a new delay and returns a [`Future`] that completes when either the timer expires.
    /// The returned future also implements [`Stream`] if this delay is [`Periodic`].
    fn delay_ms(&mut self, ms: Self::Delay) -> DelayMsFuture<Self, Self::Delay>
    where
        Self: Unpin,
    {
        DelayMsFuture {
            timer: self,
            ms: Some(ms),
            is_started: false,
        }
    }
}

pub struct DelayMsFuture<'a, T: ?Sized, U> {
    timer: &'a mut T,
    ms: Option<U>,
    is_started: bool,
}

impl<T, U> Future for DelayMsFuture<'_, T, U>
where
    T: ?Sized + DelayMs<Delay = U> + Unpin,
    U: Unpin,
{
    type Output = Result<(), T::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        if !self.is_started {
            let ms = self.ms.take().unwrap();
            self.timer.start(ms)?;

            self.is_started = true;
        }

        self.timer.poll_delay_ms_unpin(cx)
    }
}

impl<T, U> Stream for DelayMsFuture<'_, T, U>
where
    T: ?Sized + Periodic + DelayMs<Delay = U> + Unpin,
    U: Unpin,
{
    type Item = Result<(), T::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.poll_unpin(cx).map(Some)
    }
}
