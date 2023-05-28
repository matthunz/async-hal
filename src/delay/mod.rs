use core::{
    pin::Pin,
    task::{Context, Poll},
};
use futures::{Future, Stream};

pub use embedded_hal::timer::Periodic;

#[cfg(feature = "nb")]
mod timer;
#[cfg(feature = "nb")]
pub use timer::Timer;

pub trait DelayMs {
    type Error;

    fn poll_delay_ms(
        self: Pin<&mut Self>,
        cx: &mut Context,
        ms: u32,
    ) -> Poll<Result<(), Self::Error>>;

    fn poll_delay_ms_unpin(&mut self, cx: &mut Context, ms: u32) -> Poll<Result<(), Self::Error>>
    where
        Self: Unpin,
    {
        Pin::new(self).poll_delay_ms(cx, ms)
    }

    fn delay_ms(&mut self, ms: u32) -> DelayMsFuture<Self>
    where
        Self: Unpin,
    {
        DelayMsFuture { timer: self, ms }
    }

    fn interval(&mut self, ms: u32) -> Interval<Self>
    where
        Self: Periodic + Unpin,
    {
        Interval { timer: self, ms }
    }
}

pub struct DelayMsFuture<'a, T: ?Sized> {
    timer: &'a mut T,
    ms: u32,
}

impl<T> Future for DelayMsFuture<'_, T>
where
    T: DelayMs + Unpin + ?Sized,
{
    type Output = Result<(), T::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let ms = self.ms;
        self.timer.poll_delay_ms_unpin(cx, ms)
    }
}

pub struct Interval<'a, T: ?Sized> {
    timer: &'a mut T,
    ms: u32,
}

impl<T> Stream for Interval<'_, T>
where
    T: DelayMs + Unpin + ?Sized,
{
    type Item = Result<(), T::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let ms = self.ms;
        self.timer.poll_delay_ms_unpin(cx, ms).map(Some)
    }
}
