use core::{
    pin::Pin,
    task::{Context, Poll},
};
use futures::{ready, Sink, SinkExt, Stream, StreamExt};

mod async_buf_read;
pub use async_buf_read::AsyncBufRead;

mod async_read;
pub use async_read::AsyncRead;

mod async_write;
pub use async_write::AsyncWrite;

mod buf_reader;
pub use buf_reader::BufReader;

mod copy_buf;
pub use copy_buf::copy_buf;

mod cursor;
pub use cursor::Cursor;

mod read;
pub use read::Read;

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

/// Writer for a sink of bytes
pub const fn writer<T>(stream: T) -> Writer<T>
where
    T: Sink<u8> + Unpin,
{
    Writer::new(stream)
}

pub struct Writer<T> {
    sink: T,
}

impl<T> Writer<T> {
    pub const fn new(sink: T) -> Self {
        Self { sink }
    }
}

impl<T> AsyncWrite for Writer<T>
where
    T: Sink<u8> + Unpin,
{
    type Error = T::Error;

    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>> {
        let mut idx = 0;
        while idx < buf.len() {
            ready!(self.sink.poll_ready_unpin(cx))?;
            self.sink.start_send_unpin(buf[0])?;
            idx += 1;
        }

        Poll::Ready(Ok(idx))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.sink.poll_flush_unpin(cx)
    }
}
