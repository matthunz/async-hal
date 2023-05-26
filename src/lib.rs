use bxcan::{Instance, Rx0, Rx1};
use embedded_hal::can::Frame;
use futures::{task::AtomicWaker, Stream};
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
    r: R,
    waker: &'static AtomicWaker,
}

impl<R> Stream for Receiver<R>
where
    R: Receive + Unpin,
{
    type Item = Result<R::Frame, R::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        match self.r.receive() {
            Ok(frame) => Poll::Ready(Some(Ok(frame))),
            Err(nb::Error::WouldBlock) => {
                self.waker.register(cx.waker());
                Poll::Pending
            }
            Err(nb::Error::Other(error)) => Poll::Ready(Some(Err(error))),
        }
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
