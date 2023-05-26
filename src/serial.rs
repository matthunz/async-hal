use crate::Spawn;
use core::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use embedded_hal::serial::{Read, Write};
use futures::{Sink, Stream};

pub struct Reader<R, W, S> {
    read: R,
    spawn: S,
    _marker: PhantomData<W>,
}

impl<R: Unpin, W, S: Unpin> Unpin for Reader<R, W, S> {}

impl<R, W, S> Stream for Reader<R, W, S>
where
    R: Read<W> + Unpin,
    S: Spawn + Unpin,
{
    type Item = Result<W, R::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        match self.read.read() {
            Ok(frame) => Poll::Ready(Some(Ok(frame))),
            Err(nb::Error::WouldBlock) => {
                self.spawn.spawn(cx.waker());
                Poll::Pending
            }
            Err(nb::Error::Other(error)) => Poll::Ready(Some(Err(error))),
        }
    }
}

pub struct Writer<T, W, S> {
    pub write: T,
    word: Option<W>,
    spawn: S,
}

impl<T, W, S> Writer<T, W, S> {
    pub const fn new(transmit: T, spawn: S) -> Self {
        Self {
            write: transmit,
            spawn,
            word: None,
        }
    }
}

impl<T, W, S> Sink<W> for Writer<T, W, S>
where
    T: Write<W> + Unpin,
    W: Clone + Unpin,
    S: Spawn + Unpin,
{
    type Error = T::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        if self.word.is_none() {
            Poll::Ready(Ok(()))
        } else {
            self.spawn.spawn(cx.waker());
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
        let Self { write, word, spawn } = &mut *self;

        if let Some(word) = word.clone() {
            // TODO flush!
            match write.write(word) {
                Ok(()) => {
                    self.word = None;
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
