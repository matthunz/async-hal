use bxcan::{Instance, Rx0, Rx1};
use embedded_hal::can::Frame;

pub trait Receive {
    type Frame: Frame;
    type Error;

    fn receive(&mut self) -> nb::Result<Self::Frame, Self::Error>;
}

impl<I: Instance> Receive for Rx0<I> {
    type Frame = bxcan::Frame;

    type Error = bxcan::OverrunError;

    fn receive(&mut self) -> nb::Result<Self::Frame, Self::Error> {
        self.receive()
    }
}

impl<I: Instance> Receive for Rx1<I> {
    type Frame = bxcan::Frame;

    type Error = bxcan::OverrunError;

    fn receive(&mut self) -> nb::Result<Self::Frame, Self::Error> {
        self.receive()
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
