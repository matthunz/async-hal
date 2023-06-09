use crate::io::AsyncWrite;
use core::{
    marker::PhantomPinned,
    mem,
    pin::Pin,
    task::{Context, Poll},
};
use futures::{ready, Future};
use pin_project_lite::pin_project;

pin_project! {
    #[derive(Debug)]
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct WriteAll<'a, W: ?Sized> {
        writer: &'a mut W,
        buf: &'a [u8],
        // Make this future `!Unpin` for compatibility with async trait methods.
        #[pin]
        _pin: PhantomPinned,
    }
}

pub(crate) fn write_all<'a, W>(writer: &'a mut W, buf: &'a [u8]) -> WriteAll<'a, W>
where
    W: AsyncWrite + Unpin + ?Sized,
{
    WriteAll {
        writer,
        buf,
        _pin: PhantomPinned,
    }
}

impl<W> Future for WriteAll<'_, W>
where
    W: AsyncWrite + Unpin + ?Sized,
{
    type Output = Result<(), W::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let me = self.project();
        while !me.buf.is_empty() {
            let used = ready!(Pin::new(&mut *me.writer).poll_write(cx, me.buf))?;
            {
                let (_, remaining) = mem::take(&mut *me.buf).split_at(used);
                *me.buf = remaining;
            }
            if used == 0 {
                todo!()
            }
        }

        Poll::Ready(Ok(()))
    }
}
