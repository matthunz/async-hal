use super::AsyncRead;
use core::{
    marker::PhantomPinned,
    pin::Pin,
    task::{Context, Poll},
};
use futures::{ready, Future};
use pin_project_lite::pin_project;

pin_project! {
    /// A future which can be used to easily read available number of bytes to fill
    /// a buffer.
    ///
    /// Created by the [`read`] function.
    #[derive(Debug)]
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct Read<'a, R: ?Sized> {
        reader: &'a mut R,
        buf: &'a mut [u8],
        // Make this future `!Unpin` for compatibility with async trait methods.
        #[pin]
        _pin: PhantomPinned,
    }
}

impl<'a, R: ?Sized> Read<'a, R> {
    pub(super) fn new(reader: &'a mut R, buf: &'a mut [u8]) -> Self {
        Self {
            reader,
            buf,
            _pin: PhantomPinned,
        }
    }
}

impl<R> Future for Read<'_, R>
where
    R: AsyncRead + Unpin + ?Sized,
{
    type Output = Result<usize, R::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<usize, R::Error>> {
        let me = self.project();

        ready!(Pin::new(me.reader).poll_read(cx, me.buf))?;
        Poll::Ready(Ok(me.buf.len()))
    }
}
