use crate::{Spawn, Transmit};
use futures::Sink;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

pub struct Transmitter<T, F, S> {
    pub transmit: T,
    frame: Option<F>,
    spawn: S,
}

impl<T, F, S> Transmitter<T, F, S> {
    pub const fn new(transmit: T, spawn: S) -> Self {
        Self {
            transmit,
            spawn,
            frame: None,
        }
    }
}

impl<T, S> Sink<T::Frame> for Transmitter<T, T::Frame, S>
where
    T: Transmit + Unpin,
    T::Frame: Unpin,
    S: Spawn + Unpin,
{
    type Error = T::Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        if self.transmit.is_ready() {
            Poll::Ready(Ok(()))
        } else {
            self.spawn.spawn(cx.waker());
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
            spawn,
        } = &mut *self;

        if let Some(ref frame) = frame {
            match transmit.transmit(frame) {
                Ok(()) => {
                    self.frame = None;
                    Poll::Ready(Ok(()))
                }
                Err(nb::Error::WouldBlock) => {
                    spawn.spawn(cx.waker());
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
