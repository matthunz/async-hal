use async_hal::executor::Executor;

#[test]
fn it_spawns() {
    let executor = Executor::<_, 2>::new();
    assert!(executor.spawn(()).is_none());
    assert!(executor.spawn(()).is_none());

    assert!(executor.spawn(()).is_some());
}
