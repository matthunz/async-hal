use async_hal::can::{MockFrame, MockTransmit, Transmitter};
use embedded_hal::can::{Id, StandardId};
use futures::{task::AtomicWaker, SinkExt};

#[tokio::test]
async fn it_works() {
    let waker = AtomicWaker::new();
    let mut tx = Transmitter::new(MockTransmit::default(), &waker);

    let frame = MockFrame {
        id: Id::Standard(StandardId::ZERO),
        data: vec![1, 2, 3],
    };
    tx.send(frame.clone()).await.unwrap();

    assert_eq!(tx.transmit.frames[0], frame);
}
