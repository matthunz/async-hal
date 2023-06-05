use crate::{io::AsyncRead, Scheduler};
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use usb_device::{class_prelude::UsbBus, endpoint::EndpointAddress, UsbError};

pub struct Peripheral<T, S> {
    usb: T,
    scheduler: S,
}

impl<T, S> Peripheral<T, S>
where
    T: UsbBus,
    S: Scheduler,
{
    pub fn poll_read(
        self: Pin<&Self>,
        cx: &mut Context,
        addr: EndpointAddress,
        buf: &mut [u8],
    ) -> Poll<Result<usize, ()>> {
        match self.usb.read(addr, buf) {
            Ok(used) => Poll::Ready(Ok(used)),
            Err(UsbError::WouldBlock) => {
                self.scheduler.schedule(cx.waker());
                Poll::Pending
            }
            Err(_) => todo!(),
        }
    }
}

pub struct Endpoint<'a, T, S> {
    peripheral: Pin<&'a Peripheral<T, S>>,
    address: EndpointAddress,
}

impl<T, S> AsyncRead for Endpoint<'_, T, S>
where
    T: UsbBus,
    S: Scheduler,
{
    type Error = ();

    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Self::Error>> {
        self.peripheral.poll_read(cx, self.address, buf)
    }
}
