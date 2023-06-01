# async-hal
Async hardware abstraction layer for embedded devices

[![crate](https://img.shields.io/crates/v/async-hal.svg)](https://crates.io/crates/async-hal)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/async-hal)

## Examples
### Blinky
```rust
loop {
    led.toggle();
    timer.delay_ms(1_000).await.unwrap();
}
```
### Traffic light
```rust
loop {
    green.set_high();
    red.set_low();

    timer.delay_ms(100).await.unwrap();

    green.set_low();
    yellow.set_high();

    timer.delay_ms(5_000).await.unwrap();

    yellow.set_low();
    red.set_high();

    timer.delay_ms(100).await.unwrap();
}
