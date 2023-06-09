#[cfg(feature = "mock")]
mod tests {
    use async_hal::{
        block_on,
        can::{MockFrame, MockTransmit, Transmitter},
    };
    use embedded_hal::can::{Id, StandardId};
    use futures::SinkExt;

    #[test]
    fn it_works() {
        let mut tx = Transmitter::new(MockTransmit::default());

        let frame = MockFrame {
            id: Id::Standard(StandardId::ZERO),
            data: vec![1, 2, 3],
        };

        block_on(tx.send(frame.clone()), || {}).unwrap();

        assert_eq!(tx.transmit.frames[0], frame);
    }
}
