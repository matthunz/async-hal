use embedded_hal::can::{Frame, Id};

mod transmitter;
pub use transmitter::Transmitter;

pub trait Transmit {
    type Frame: Frame;
    type Error;

    fn is_ready(&mut self) -> bool;

    fn transmit(&mut self, frame: &Self::Frame) -> nb::Result<(), Self::Error>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MockFrame {
    pub id: Id,
    pub data: Vec<u8>,
}

impl Frame for MockFrame {
    fn new(id: impl Into<Id>, data: &[u8]) -> Option<Self> {
        Some(Self {
            id: id.into(),
            data: data.to_owned(),
        })
    }

    fn new_remote(_id: impl Into<Id>, _dlc: usize) -> Option<Self> {
        todo!()
    }

    fn is_extended(&self) -> bool {
        todo!()
    }

    fn is_remote_frame(&self) -> bool {
        todo!()
    }

    fn id(&self) -> Id {
        todo!()
    }

    fn dlc(&self) -> usize {
        todo!()
    }

    fn data(&self) -> &[u8] {
        todo!()
    }
}

#[derive(Default)]
pub struct MockTransmit {
    pub frames: Vec<MockFrame>,
}

impl Transmit for MockTransmit {
    type Frame = MockFrame;

    type Error = ();

    fn is_ready(&mut self) -> bool {
        true
    }

    fn transmit(&mut self, frame: &Self::Frame) -> nb::Result<(), Self::Error> {
        self.frames.push(frame.clone());
        Ok(())
    }
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
