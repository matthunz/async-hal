use crate::{Interrupt, Scheduler};
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use embedded_hal::timer::CountDown;
use futures::{future, Stream, StreamExt};

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

    pub fn interval<U, C>(&mut self, count: C) -> Interval<T, S, U>
    where
        C: Into<U>,
    {
        Interval {
            timer: self,
            time: count.into(),
        }
    }
}

pub struct Interval<'a, T, S, U> {
    timer: &'a mut Timer<T, S>,
    time: U,
}

impl<T, S> Stream for Interval<'_, T, S, T::Time>
where
    T: CountDown + Unpin,
    T::Time: Clone + Unpin,
    S: Scheduler + Unpin,
{
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let time = self.time.clone();
        self.timer.poll_wait(cx, time).map(|()| Some(()))
    }
}
