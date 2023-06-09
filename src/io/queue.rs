use super::{AsyncRead, AsyncWrite};
use bbqueue::{BBBuffer, Consumer, Producer};
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use futures::task::AtomicWaker;

pub struct Queue<const N: usize> {
    queue: BBBuffer<N>,
    waker: AtomicWaker,
}

impl<const N: usize> Queue<N> {
    pub const fn new() -> Self {
        Self {
            queue: BBBuffer::new(),
            waker: AtomicWaker::new(),
        }
    }

    pub fn try_split(&self) -> bbqueue::Result<(Reader<N>, Writer<N>)> {
        self.queue
            .try_split()
            .map(|(tx, rx)| (Reader { queue: self, rx }, Writer { queue: self, tx }))
    }
}

pub struct Reader<'a, const N: usize> {
    queue: &'a Queue<N>,
    rx: Consumer<'a, N>,
}

impl<const N: usize> AsyncRead for Reader<'_, N> {
    type Error = ();

    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        let bytes = self.rx.read().unwrap();
        if bytes.len() == 0 {
            self.queue.waker.register(cx.waker());
            return Poll::Pending;
        }

        buf[..bytes.len()].copy_from_slice(&*bytes);
        Poll::Ready(Ok(bytes.len()))
    }
}

pub struct Writer<'a, const N: usize> {
    queue: &'a Queue<N>,
    tx: Producer<'a, N>,
}

impl<const N: usize> AsyncWrite for Writer<'_, N> {
    type Error = ();

    fn poll_write(
        mut self: Pin<&mut Self>,
        _cx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Self::Error>> {
        let mut grant = self.tx.grant_max_remaining(buf.len()).unwrap();
        let used = core::cmp::min(buf.len(), grant.len());
        grant[..used].copy_from_slice(&buf[..used]);
        grant.commit(used);

        self.queue.waker.wake();
        Poll::Ready(Ok(used))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}
