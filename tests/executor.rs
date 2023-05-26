use std::pin::Pin;

use async_hal::executor::Executor;
use futures::Future;

#[test]
fn it_spawns() {
    let executor = Executor::<_, 2>::new();
    assert!(executor.spawn(()).is_none());
    assert!(executor.spawn(()).is_none());

    assert!(executor.spawn(()).is_some());
}

#[test]
fn it_runs() {
    let executor = Executor::<_, 2>::new();

    let a: Pin<Box<dyn Future<Output = ()>>> = Box::pin(async {
        dbg!("A");
    });
    let b: Pin<Box<dyn Future<Output = ()>>> = Box::pin(async {
        dbg!("B");
    });

    executor.spawn(a);
    executor.spawn(b);

    assert!(executor.run().is_some());
    assert!(executor.run().is_some());
    assert!(executor.run().is_none());
}
