use async_hal::executor::Executor;
use futures::Future;
use std::pin::Pin;

#[test]
fn it_spawns_tasks() {
    let executor = Executor::<_, 2>::new();

    assert!(executor.spawn(()).is_none());
    assert!(executor.spawn(()).is_none());

    assert!(executor.spawn(()).is_some());
}

#[test]
fn it_runs_tasks() {
    let mut executor = Executor::<_, 2>::new();

    let mut a_was_polled = false;
    let a: Pin<Box<dyn Future<Output = ()>>> = Box::pin(async {
        a_was_polled = true;
    });

    let mut b_was_polled = false;
    let b: Pin<Box<dyn Future<Output = ()>>> = Box::pin(async {
        b_was_polled = true;
    });

    executor.spawn(a);
    executor.spawn(b);

    assert!(executor.run().is_some());
    assert!(executor.run().is_some());
    assert!(executor.run().is_none());

    assert!(a_was_polled);
    assert!(b_was_polled);
}
