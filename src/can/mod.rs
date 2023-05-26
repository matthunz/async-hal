mod transmit;
pub use transmit::{Transmit, Transmitter};

#[cfg(feature = "mock")]
pub use transmit::MockTransmit;

mod receive;
pub use receive::{DualReceiver, Receive, Receiver};

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
