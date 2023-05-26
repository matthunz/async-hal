use embedded_hal::can::Frame;

mod receiver;
pub use receiver::{DualReceiver, Receiver};

pub trait Receive {
    type Frame: Frame;
    type Error;

    fn receive(&mut self) -> nb::Result<Self::Frame, Self::Error>;
}

#[cfg(feature = "bxcan")]
impl<I: bxcan::Instance> Receive for bxcan::Rx0<I> {
    type Frame = bxcan::Frame;
    type Error = bxcan::OverrunError;

    fn receive(&mut self) -> nb::Result<Self::Frame, Self::Error> {
        self.receive()
    }
}

#[cfg(feature = "bxcan")]
impl<I: bxcan::Instance> Receive for bxcan::Rx1<I> {
    type Frame = bxcan::Frame;
    type Error = bxcan::OverrunError;

    fn receive(&mut self) -> nb::Result<Self::Frame, Self::Error> {
        self.receive()
    }
}
