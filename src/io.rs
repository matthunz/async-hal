use core::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{ready, Stream, StreamExt};

/// Read bytes asynchronously.
pub trait AsyncRead {
    type Error;

    /// Attempt to read from the AsyncRead into buf.
    /// On success, returns Poll::Ready(Ok(num_bytes_read)).
    /// If no data is available for reading, this method returns Poll::Pending
    /// and arranges for the current task to be woken.
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>>;
}

/// Reader for a stream of bytes
pub const fn reader<T, E>(stream: T) -> Reader<T>
where
    T: Stream<Item = Result<u8, E>> + Unpin,
{
    Reader::new(stream)
}

pub struct Reader<T> {
    stream: T,
    idx: usize,
}

impl<T> Reader<T> {
    pub const fn new(stream: T) -> Self {
        Self { stream, idx: 0 }
    }
}

impl<T, E> AsyncRead for Reader<T>
where
    T: Stream<Item = Result<u8, E>> + Unpin,
{
    type Error = E;

    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        while self.idx < buf.len() {
            let byte = ready!(self.stream.poll_next_unpin(cx)).unwrap()?;
            buf[self.idx] = byte;
            self.idx += 1;
        }

        let used = self.idx;
        self.idx = 0;
        Poll::Ready(Ok(used))
    }
}
