use crate::Scheduler;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use embedded_hal::serial::Write;
use futures::Sink;

/// Write half of a UART serial port.
pub struct Writer<T, W, S> {
    // Generic non-blocking serial writer
    write: T,

    // Cache of the next word to send
    word: Option<W>,

    // Scheduler to wake the write task
    scheduler: S,
}

impl<T, W, S> Writer<T, W, S> {
    /// Create a new writer from an instance of [`Write`] and [`Scheduler`].
    pub const fn new(transmit: T, scheduler: S) -> Self {
        Self {
            write: transmit,
            scheduler,
            word: None,
        }
    }
}

impl<T, W, S> Sink<W> for Writer<T, W, S>
where
    T: Write<W> + Unpin,
    W: Clone + Unpin,
    S: Scheduler + Unpin,
{
    type Error = T::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        if self.word.is_none() {
            Poll::Ready(Ok(()))
        } else {
            self.scheduler.schedule(cx.waker());
            Poll::Pending
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: W) -> Result<(), Self::Error> {
        if self.word.is_none() {
            self.word = Some(item);
            Ok(())
        } else {
            todo!()
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        let Self {
            write,
            word,
            scheduler: spawn,
        } = &mut *self;

        if let Some(word) = word.clone() {
            // TODO flush!
            match write.write(word) {
                Ok(()) => {
                    self.word = None;
                    Poll::Ready(Ok(()))
                }
                Err(nb::Error::WouldBlock) => {
                    spawn.schedule(cx.waker());
                    Poll::Pending
                }
                Err(nb::Error::Other(error)) => Poll::Ready(Err(error)),
            }
        } else {
            Poll::Ready(Ok(()))
        }
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}
