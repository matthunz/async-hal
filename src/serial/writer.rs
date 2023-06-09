use core::{
    pin::Pin,
    task::{Context, Poll},
};
use embedded_hal::serial::Write;
use futures::Sink;

/// Write half of a UART serial port.
pub struct Writer<T, W> {
    // Generic non-blocking serial writer
    write: T,

    // Cache of the next word to send
    word: Option<W>,
}

impl<T, W> Writer<T, W> {
    /// Create a new writer from an instance of [`Write`] and [`Scheduler`].
    pub const fn new(transmit: T) -> Self {
        Self {
            write: transmit,
            word: None,
        }
    }
}

impl<T, W> Sink<W> for Writer<T, W>
where
    T: Write<W> + Unpin,
    W: Clone + Unpin,
{
    type Error = T::Error;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        if self.word.is_none() {
            Poll::Ready(Ok(()))
        } else {
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

    fn poll_flush(mut self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        let Self { write, word } = &mut *self;

        if let Some(word) = word.clone() {
            // TODO flush!
            match write.write(word) {
                Ok(()) => {
                    self.word = None;
                    Poll::Ready(Ok(()))
                }
                Err(nb::Error::WouldBlock) => Poll::Pending,
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
