use core::{
    pin::Pin,
    task::{Context, Poll},
};

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
