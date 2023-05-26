use can::{MockFrame, MockTransmit, Transmit, Transmitter};
use embedded_hal::can::{Id, StandardId};
use futures::{task::AtomicWaker, SinkExt};

#[tokio::test]
async fn it_works() {
    static WAKER: AtomicWaker = AtomicWaker::new();
    let frame = MockFrame {
        id: Id::Standard(StandardId::ZERO),
        data: vec![1, 2, 3],
    };

    let mut tx = Transmitter::new(MockTransmit::default(), &WAKER);
    tx.send(frame.clone()).await.unwrap();

    assert_eq!(tx.transmit.frames[0], frame);
}
