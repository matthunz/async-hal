use pin_project_lite::pin_project;

use super::DelayMs;
use core::{
    pin::Pin,
    task::{Context, Poll},
};

/// Create a timer that's ready as soon as it's started.
/// ```
/// use async_hal::delay::{self, DelayMs};
///
/// # let fut = async {
/// let mut delay = delay::ready::<u8>();
///
/// // Ready instantly
/// assert!(delay.delay_ms(100).await.is_ok());
/// # };
/// # futures::pin_mut!(fut);
/// # async_hal::block_on(fut, || {});
/// ```
pub fn ready<D>() -> Ready<D> {
    Ready { delay: None }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AlreadyStarted {
    _priv: (),
}

pin_project! {
    pub struct Ready<D> {
        delay: Option<D>,
    }
}

impl<D> DelayMs for Ready<D> {
    type Delay = D;

    type Error = AlreadyStarted;

    fn start(&mut self, ms: Self::Delay) -> Result<(), Self::Error> {
        self.delay = Some(ms);
        Ok(())
    }

    fn poll_delay_ms(mut self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        if self.delay.take().is_some() {
            Poll::Ready(Ok(()))
        } else {
            Poll::Ready(Err(AlreadyStarted { _priv: () }))
        }
    }

    fn cancel(&mut self) -> Result<(), Self::Error> {
        self.delay = None;
        Ok(())
    }
}
