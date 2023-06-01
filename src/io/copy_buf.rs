use super::{AsyncBufRead, AsyncWrite};
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use futures::{ready, Future};

/// A future that asynchronously copies the entire contents of a reader into a
/// writer.
///
/// This struct is generally created by calling [`copy_buf`][copy_buf]. Please
/// see the documentation of `copy_buf()` for more details.
///
/// [copy_buf]: copy_buf()
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
struct CopyBuf<'a, R: ?Sized, W: ?Sized> {
    reader: &'a mut R,
    writer: &'a mut W,
    amt: u64,
}

/// Asynchronously copies the entire contents of a reader into a writer.
///
/// This function returns a future that will continuously read data from
/// `reader` and then write it into `writer` in a streaming fashion until
/// `reader` returns EOF or fails.
///
/// On success, the total number of bytes that were copied from `reader` to
/// `writer` is returned.
///
/// This is a [`async_hal::io::copy`] alternative for [`AsyncBufRead`] readers
/// with no extra buffer allocation, since [`AsyncBufRead`] allow access
/// to the reader's inner buffer.
///
/// [`async_hal::io::copy`]: crate::io::copy
/// [`AsyncBufRead`]: crate::io::AsyncBufRead
///
/// # Errors
///
/// The returned future will finish with an error will return an error
/// immediately if any call to `poll_fill_buf` or `poll_write` returns an
/// error.
///
/// # Examples
///
/// ```
/// use async_hal::io;
///
/// # let task = async {
/// let mut reader: &[u8] = b"hello";
/// let mut writer = [0; 5];
///
/// io::copy_buf(&mut reader, &mut writer.as_mut()).await?;
///
/// assert_eq!(b"hello", &writer[..]);
/// # Ok::<_, void::Void>(())
/// # };
/// # futures::pin_mut!(task);
/// # async_hal::block_on(task, || {}).unwrap();
/// ```
pub async fn copy_buf<'a, R, W>(reader: &'a mut R, writer: &'a mut W) -> Result<u64, R::Error>
where
    R: AsyncBufRead + Unpin + ?Sized,
    W: AsyncWrite + Unpin + ?Sized,
    R::Error: From<W::Error>,
{
    CopyBuf {
        reader,
        writer,
        amt: 0,
    }
    .await
}

impl<R, W> Future for CopyBuf<'_, R, W>
where
    R: AsyncBufRead + Unpin + ?Sized,
    W: AsyncWrite + Unpin + ?Sized,
    R::Error: From<W::Error>,
{
    type Output = Result<u64, R::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let me = &mut *self;
            let buffer = ready!(Pin::new(&mut *me.reader).poll_fill_buf(cx))?;
            if buffer.is_empty() {
                ready!(Pin::new(&mut *me.writer).poll_flush(cx))?;
                return Poll::Ready(Ok(self.amt));
            }

            let i = ready!(Pin::new(&mut *me.writer).poll_write(cx, buffer))?;
            if i == 0 {
                todo!()
                //return Poll::Ready(Err(std::io::ErrorKind::WriteZero.into()));
            }
            self.amt += i as u64;
            Pin::new(&mut *self.reader).consume(i);
        }
    }
}
