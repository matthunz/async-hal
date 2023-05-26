mod transmitter;
use embedded_hal::can::Frame;
pub use transmitter::Transmitter;

pub trait Transmit {
    type Frame: Frame;
    type Error;

    fn is_ready(&mut self) -> bool;

    fn transmit(&mut self, frame: &Self::Frame) -> nb::Result<(), Self::Error>;
}

#[cfg(feature = "bxcan")]
impl<I: bxcan::Instance> Transmit for bxcan::Tx<I> {
    type Frame = bxcan::Frame;

    type Error = core::convert::Infallible;

    fn is_ready(&mut self) -> bool {
        !self.is_idle()
    }

    fn transmit(&mut self, frame: &Self::Frame) -> nb::Result<(), Self::Error> {
        self.transmit(frame).map(|_| ())
    }
}
