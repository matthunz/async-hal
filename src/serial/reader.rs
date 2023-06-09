/// Read half of a UART serial port.
use core::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use embedded_hal::serial::Read;
use futures::Stream;

pub struct Reader<R, W> {
    read: R,
    _marker: PhantomData<W>,
}

impl<R, W> Reader<R, W> {
    /// Create a new reader from an instance of [`Read`].
    pub const fn new(read: R) -> Self {
        Self {
            read,
            _marker: PhantomData,
        }
    }
}

impl<R, W> Unpin for Reader<R, W> {}

impl<R, W> Stream for Reader<R, W>
where
    R: Read<W> + Unpin,
{
    type Item = Result<W, R::Error>;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Option<Self::Item>> {
        match self.read.read() {
            Ok(frame) => Poll::Ready(Some(Ok(frame))),
            Err(nb::Error::WouldBlock) => Poll::Pending,
            Err(nb::Error::Other(error)) => Poll::Ready(Some(Err(error))),
        }
    }
}
