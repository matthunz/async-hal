use super::Read;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use void::Void;

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

    /// Tries to read some bytes directly into the given `buf` in asynchronous
    /// manner, returning a future type.
    ///
    /// The returned future will resolve to both the I/O stream and the buffer
    /// as well as the number of bytes read once the read operation is completed.
    /// ```
    /// use async_hal::io::AsyncRead;
    ///
    /// let mut bytes = [1, 2, 3].as_ref();
    /// let mut buf = [0; 3];
    ///
    /// # let fut = async {
    /// bytes.read(&mut buf).await?;   
    /// # Ok::<_, void::Void>(())
    /// # };
    /// # futures::pin_mut!(fut);
    /// # async_hal::block_on(fut, || {}).unwrap();
    ///
    /// assert_eq!([1, 2, 3], buf);
    /// ```
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Read<'a, Self>
    where
        Self: Unpin,
    {
        Read::new(self, buf)
    }
}

macro_rules! deref_async_read {
    () => {
        fn poll_read(
            mut self: Pin<&mut Self>,
            cx: &mut Context,
            buf: &mut [u8],
        ) -> Poll<Result<usize, Self::Error>> {
            Pin::new(&mut **self).poll_read(cx, buf)
        }
    };
}

#[cfg(feature = "alloc")]
impl<T: ?Sized + AsyncRead + Unpin> AsyncRead for Box<T> {
    deref_async_read!();
}

impl<T: ?Sized + AsyncRead + Unpin> AsyncRead for &mut T {
    type Error = T::Error;

    deref_async_read!();
}

impl AsyncRead for &[u8] {
    type Error = Void;

    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Void>> {
        let amt = core::cmp::min(self.len(), buf.len());
        let (a, b) = self.split_at(amt);
        buf[..amt].copy_from_slice(a);
        *self = b;
        Poll::Ready(Ok(amt))
    }
}
