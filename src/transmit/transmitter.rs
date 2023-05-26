use crate::Transmit;

use futures::{task::AtomicWaker, Sink};
use std::{
    pin::Pin,
    task::{Context, Poll},
};

pub struct Transmitter<T, F> {
    pub transmit: T,
    frame: Option<F>,
    waker: &'static AtomicWaker,
}

impl<T, F> Transmitter<T, F> {
    pub const fn new(transmit: T, waker: &'static AtomicWaker) -> Self {
        Self {
            transmit,
            waker,
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

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        if self.transmit.is_ready() {
            Poll::Ready(Ok(()))
        } else {
            self.waker.register(cx.waker());
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

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        let Self {
            transmit,
            frame,
            waker,
        } = &mut *self;

        if let Some(ref frame) = frame {
            match transmit.transmit(frame) {
                Ok(()) => {
                    self.frame = None;
                    Poll::Ready(Ok(()))
                }
                Err(nb::Error::WouldBlock) => {
                    waker.register(cx.waker());
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
