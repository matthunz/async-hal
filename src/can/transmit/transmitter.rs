use super::Transmit;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use futures::Sink;

/// Transmitter sink for frames to a CAN bus.
pub struct Transmitter<T, F> {
    pub transmit: T,
    frame: Option<F>,
}

impl<T, F> Transmitter<T, F> {
    pub const fn new(transmit: T) -> Self {
        Self {
            transmit,
            frame: None,
        }
    }
}

impl<T> Sink<T::Frame> for Transmitter<T, T::Frame>
where
    T: Transmit + Unpin,
    T::Frame: Unpin,
{
    type Error = T::Error;

    fn poll_ready(mut self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        if self.transmit.is_ready() {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: T::Frame) -> Result<(), Self::Error> {
        if self.frame.is_none() {
            self.frame = Some(item);
            Ok(())
        } else {
            todo!()
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        let Self { transmit, frame } = &mut *self;

        if let Some(ref frame) = frame {
            match transmit.transmit(frame) {
                Ok(()) => {
                    self.frame = None;
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
