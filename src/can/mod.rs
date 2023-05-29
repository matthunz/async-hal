use futures::{Sink, Stream};

#[cfg(feature = "nb")]
pub mod transmit;
#[cfg(feature = "mock")]
pub use transmit::MockTransmit;
#[cfg(feature = "nb")]
pub use transmit::Transmitter;

#[cfg(feature = "nb")]
pub mod receive;
#[cfg(feature = "nb")]
pub use receive::{DualReceiver, Receiver};

pub use embedded_hal::can::Frame;

pub trait Can: CanReceive + CanTransmit<Self::Frame> {}

impl<T> Can for T where T: ?Sized + CanReceive + CanTransmit<Self::Frame> {}

pub trait CanReceive:
    Stream<Item = Result<Self::Frame, Self::Error>> + can_receive::Sealed
{
    type Frame: Frame;

    type Error;
}

impl<T, F, E> CanReceive for T
where
    T: ?Sized + Stream<Item = Result<F, E>>,
    F: ?Sized + Frame,
{
    type Frame = F;

    type Error = E;
}

pub trait CanTransmit<F>: Sink<F> + can_transmit::Sealed {}

impl<T, F> CanTransmit<F> for T where T: ?Sized + Sink<F> {}

mod can_receive {
    use super::Stream;

    pub trait Sealed {}

    impl<T, F, E> Sealed for T where T: ?Sized + Stream<Item = Result<F, E>> {}
}

mod can_transmit {

    pub trait Sealed {}

    impl<T> Sealed for T where T: ?Sized {}
}

#[cfg(feature = "mock")]
mod mock {
    use embedded_hal::can::{Frame, Id};

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
}

#[cfg(feature = "mock")]
pub use mock::MockFrame;
