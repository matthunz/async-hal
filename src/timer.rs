use crate::{Interrupt, Scheduler};
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use embedded_hal::timer::CountDown;
use futures::{future, Stream, StreamExt};
use void::Void;

pub trait Delay<C> {
    type Error;

    fn poll_delay(
        self: Pin<&mut Self>,
        cx: &mut Context,
        count: impl Into<C>,
    ) -> Poll<Result<(), Self::Error>>;

    fn poll_delay_unpin(
        &mut self,
        cx: &mut Context,
        count: impl Into<C>,
    ) -> Poll<Result<(), Self::Error>>
    where
        Self: Unpin,
    {
        Pin::new(self).poll_delay(cx, count)
    }

    fn interval(&mut self, count: impl Into<C>) -> Interval<Self, C>
    where
        Self: Sized + Unpin,
        C: Clone + Unpin,
    {
        Interval {
            timer: self,
            time: count.into(),
        }
    }
}

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

    pub fn interval<U, C>(&mut self, count: C) -> Interval<Self, U>
    where
        C: Into<U>,
    {
        Interval {
            timer: self,
            time: count.into(),
        }
    }
}

impl<T, S> Delay<T::Time> for Timer<T, S>
where
    T: CountDown + Unpin,
    S: Scheduler + Unpin,
{
    type Error = Void;

    fn poll_delay(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        count: impl Into<T::Time>,
    ) -> Poll<Result<(), Self::Error>> {
        if !self.is_started {
            self.is_started = true;
            self.timer.start(count);
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

pub struct Interval<'a, T, U> {
    timer: &'a mut T,
    time: U,
}

impl<T, U> Stream for Interval<'_, T, U>
where
    T: Delay<U> + Unpin,
    U: Clone + Unpin,
{
    type Item = Result<(), T::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let time = self.time.clone();
        self.timer.poll_delay_unpin(cx, time).map(Some)
    }
}
