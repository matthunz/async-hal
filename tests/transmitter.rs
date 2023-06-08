#[cfg(feature = "mock")]
mod tests {
    use async_hal::{
        block_on,
        can::{MockFrame, MockTransmit, Transmitter},
    };
    use embedded_hal::can::{Id, StandardId};
    use futures::{task::AtomicWaker, SinkExt};

    #[test]
    fn it_works() {
        let waker = AtomicWaker::new();
        let mut tx = Transmitter::new(MockTransmit::default(), &waker);

        let frame = MockFrame {
            id: Id::Standard(StandardId::ZERO),
            data: vec![1, 2, 3],
        };

        block_on(tx.send(frame.clone()), || {}).unwrap();

        assert_eq!(tx.transmit.frames[0], frame);
    }
}
