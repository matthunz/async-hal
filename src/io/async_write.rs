use core::{
    ops::DerefMut,
    pin::Pin,
    task::{Context, Poll},
};
use void::Void;

pub trait AsyncWrite {
    type Error;

    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>>;

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>>;
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
        let used = core::cmp::min(buf.len(), self.len());
        let (a, b) = core::mem::take(&mut *self).split_at_mut(used);
        a.copy_from_slice(&buf[..used]);
        *self = b;
        Poll::Ready(Ok(used))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}
