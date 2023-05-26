use bxcan::{Instance, Rx0, Rx1};
use embedded_hal::can::Frame;
use futures::{task::AtomicWaker, Sink, Stream};
use std::{
    pin::Pin,
    task::{Context, Poll},
};

pub trait Receive {
    type Frame: Frame;
    type Error;

    fn receive(&mut self) -> nb::Result<Self::Frame, Self::Error>;
}

impl<I: Instance> Receive for Rx0<I> {
    type Frame = bxcan::Frame;

    type Error = bxcan::OverrunError;

    fn receive(&mut self) -> nb::Result<Self::Frame, Self::Error> {
        self.receive()
    }
}

impl<I: Instance> Receive for Rx1<I> {
    type Frame = bxcan::Frame;

    type Error = bxcan::OverrunError;

    fn receive(&mut self) -> nb::Result<Self::Frame, Self::Error> {
        self.receive()
    }
}

pub struct Receiver<R> {
    receive: R,
    waker: &'static AtomicWaker,
}

impl<R> Receiver<R> {
    pub const fn new(receive: R, waker: &'static AtomicWaker) -> Self {
        Self { receive, waker }
    }
}

impl<R> Stream for Receiver<R>
where
    R: Receive + Unpin,
{
    type Item = Result<R::Frame, R::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        match self.receive.receive() {
            Ok(frame) => Poll::Ready(Some(Ok(frame))),
            Err(nb::Error::WouldBlock) => {
                self.waker.register(cx.waker());
                Poll::Pending
            }
            Err(nb::Error::Other(error)) => Poll::Ready(Some(Err(error))),
        }
    }
}

pub trait Transmit {
    type Frame: Frame;
    type Error;

    fn is_ready(&mut self) -> bool;

    fn transmit(&mut self, frame: &Self::Frame) -> nb::Result<(), Self::Error>;

    fn abort(&mut self) -> bool;
}

pub struct Transmitter<T, F> {
    transmit: T,
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

    fn poll_close(mut self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.transmit.abort();
        Poll::Ready(Ok(()))
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
