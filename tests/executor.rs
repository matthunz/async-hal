use async_hal::executor::Executor;

#[test]
fn it_spawns() {
    let executor = Executor::<_, 2>::new();
    executor.spawn(());
    executor.spawn(());
}
