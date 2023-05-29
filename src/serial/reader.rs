/// Read half of a UART serial port.
use crate::Scheduler;
use core::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use embedded_hal::serial::Read;
use futures::Stream;

pub struct Reader<R, W, S> {
    read: R,
    spawn: S,
    _marker: PhantomData<W>,
}

impl<R, W, S> Reader<R, W, S> {
    /// Create a new reader from an instance of [`Read`] and [`Scheduler`].
    pub const fn new(read: R, spawn: S) -> Self {
        Self {
            read,
            spawn,
            _marker: PhantomData,
        }
    }
}

impl<R: Unpin, W, S: Unpin> Unpin for Reader<R, W, S> {}

impl<R, W, S> Stream for Reader<R, W, S>
where
    R: Read<W> + Unpin,
    S: Scheduler + Unpin,
{
    type Item = Result<W, R::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        match self.read.read() {
            Ok(frame) => Poll::Ready(Some(Ok(frame))),
            Err(nb::Error::WouldBlock) => {
                self.spawn.schedule(cx.waker());
                Poll::Pending
            }
            Err(nb::Error::Other(error)) => Poll::Ready(Some(Err(error))),
        }
    }
}
