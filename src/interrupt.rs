use crate::Scheduler;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use futures::Stream;

pub struct Interrupt<S> {
    scheduler: S,
    is_waiting: bool,
}

impl<S> Interrupt<S> {
    pub const fn new(scheduler: S) -> Self {
        Self {
            scheduler,
            is_waiting: false,
        }
    }
}

impl<S> Stream for Interrupt<S>
where
    S: Scheduler + Unpin,
{
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        if self.is_waiting {
            self.is_waiting = false;
            Poll::Ready(Some(()))
        } else {
            self.is_waiting = true;
            self.scheduler.schedule(cx.waker());

            Poll::Pending
        }
    }
}
