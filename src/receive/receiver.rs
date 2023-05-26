use futures::{task::AtomicWaker, Stream};
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crate::Receive;

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

pub struct DualReceiver<T, U> {
    rx0: T,
    rx1: U,
    waker: &'static AtomicWaker,
}

impl<T, U> DualReceiver<T, U> {
    pub const fn new(rx0: T, rx1: U, waker: &'static AtomicWaker) -> Self {
        Self { rx0, rx1, waker }
    }
}

impl<T, U> Stream for DualReceiver<T, U>
where
    T: Receive + Unpin,
    U: Receive<Frame = T::Frame, Error = T::Error> + Unpin,
{
    type Item = Result<T::Frame, T::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        match self.rx0.receive() {
            Ok(frame) => Poll::Ready(Some(Ok(frame))),
            Err(nb::Error::WouldBlock) => match self.rx1.receive() {
                Ok(frame) => Poll::Ready(Some(Ok(frame))),
                Err(nb::Error::WouldBlock) => {
                    self.waker.register(cx.waker());
                    Poll::Pending
                }
                Err(nb::Error::Other(error)) => Poll::Ready(Some(Err(error))),
            },
            Err(nb::Error::Other(error)) => Poll::Ready(Some(Err(error))),
        }
    }
}
