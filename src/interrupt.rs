use core::{
    pin::Pin,
    task::{Context, Poll},
};
use futures::Stream;

pub trait Interrupt {
    type Error;

    fn enable(&mut self) -> Result<(), Self::Error>;

    fn disable(&mut self) -> Result<(), Self::Error>;

    fn poll_interrupt(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>>;

    fn poll_interrupt_unpin(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>>
    where
        Self: Unpin,
    {
        Pin::new(self).poll_interrupt(cx)
    }

    /// Enable the interrupt and return a [`Stream`] of events.
    /// This will disable the interrupt on drop.
    fn interrupts(&mut self) -> Interrupts<Self>
    where
        Self: Unpin,
    {
        Interrupts {
            interrupt: self,
            is_enabled: false,
        }
    }
}

pub struct Interrupts<'a, T: Interrupt + ?Sized> {
    interrupt: &'a mut T,
    is_enabled: bool,
}

impl<T> Stream for Interrupts<'_, T>
where
    T: Interrupt + Unpin + ?Sized,
{
    type Item = Result<(), T::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if !self.is_enabled {
            self.interrupt.enable()?;
            self.is_enabled = true;
        }

        self.interrupt.poll_interrupt_unpin(cx).map(Some)
    }
}

impl<T> Drop for Interrupts<'_, T>
where
    T: Interrupt + ?Sized,
{
    fn drop(&mut self) {
        self.interrupt.disable().ok();
    }
}
