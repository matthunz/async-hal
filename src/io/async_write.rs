use core::{
    ops::DerefMut,
    pin::Pin,
    task::{Context, Poll},
};
use void::Void;

use super::{write_all::write_all, WriteAll};

pub trait AsyncWrite {
    type Error;

    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>>;

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>>;

    /// Attempts to write an entire buffer into this writer.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn write_all(&mut self, buf: &[u8]) -> io::Result<()>;
    /// ```
    ///
    /// This method will continuously call [`write`] until there is no more data
    /// to be written. This method will not return until the entire buffer
    /// has been successfully written or such an error occurs. The first
    /// error generated from this method will be returned.
    ///
    /// # Cancel safety
    ///
    /// This method is not cancellation safe. If it is used as the event
    /// in a [`tokio::select!`](crate::select) statement and some other
    /// branch completes first, then the provided buffer may have been
    /// partially written, but future calls to `write_all` will start over
    /// from the beginning of the buffer.
    ///
    /// # Errors
    ///
    /// This function will return the first error that [`write`] returns.
    /// [`write`]: AsyncWrite::write
    fn write_all<'a>(&'a mut self, buf: &'a [u8]) -> WriteAll<'a, Self>
    where
        Self: Unpin,
    {
        write_all(self, buf)
    }
}

impl<T: ?Sized + AsyncWrite + Unpin> AsyncWrite for &mut T {
    type Error = T::Error;

    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>> {
        Pin::new(&mut **self).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut **self).poll_flush(cx)
    }
}

impl<P> AsyncWrite for Pin<P>
where
    P: DerefMut + Unpin,
    P::Target: AsyncWrite,
{
    type Error = <P::Target as AsyncWrite>::Error;

    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>> {
        self.get_mut().as_mut().poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.get_mut().as_mut().poll_flush(cx)
    }
}

impl AsyncWrite for &'_ mut [u8] {
    type Error = Void;

    fn poll_write(
        mut self: Pin<&mut Self>,
        _cx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>> {
        let amt = core::cmp::min(buf.len(), self.len());
        let (a, b) = core::mem::replace(&mut *self, &mut []).split_at_mut(amt);
        a.copy_from_slice(&buf[..amt]);
        *self = b;
        Poll::Ready(Ok(amt))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}
