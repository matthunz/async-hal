use core::{
    pin::Pin,
    task::{Context, Poll},
};
use futures::Future;

pub trait Interrupt {
    type Error;

    fn poll_interrupt(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>>;

    fn cancel(&mut self) -> Result<bool, Self::Error>;

    fn poll_interrupt_unpin(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>>
    where
        Self: Unpin,
    {
        Pin::new(self).poll_interrupt(cx)
    }

    /// This will cancel the interrupt on drop.
    fn interrupt(&mut self) -> InterruptFuture<Self>
    where
        Self: Unpin,
    {
        InterruptFuture { interrupt: self }
    }
}

pub struct InterruptFuture<'a, T: Interrupt + ?Sized> {
    interrupt: &'a mut T,
}

impl<T> Future for InterruptFuture<'_, T>
where
    T: Interrupt + Unpin + ?Sized,
{
    type Output = Result<(), T::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        self.interrupt.poll_interrupt_unpin(cx)
    }
}

impl<T> Drop for InterruptFuture<'_, T>
where
    T: Interrupt + ?Sized,
{
    fn drop(&mut self) {
        self.interrupt.cancel().ok();
    }
}
