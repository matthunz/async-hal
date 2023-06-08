# async-hal
Async hardware abstraction layer for embedded devices

[![crate](https://img.shields.io/crates/v/async-hal.svg)](https://crates.io/crates/async-hal)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/async-hal)
[![CI](https://github.com/matthunz/async-hal/actions/workflows/rust.yml/badge.svg)](https://github.com/matthunz/async-hal/actions/workflows/rust.yml)

## [Examples](https://github.com/matthunz/async-hal/tree/main/examples/)
### [Blinky](https://github.com/matthunz/async-hal/tree/main/examples/bin/blinky)
```rust
use async_hal::delay::DelayMs;

let mut led = _;
let mut timer = _;

loop {
    led.toggle();
    timer.delay_ms(1_000).await?;
}
```

### Serial port loopback
```rust
use async_hal::io;

let mut serial_tx = _;
let mut serial_rx = _;

loop {
    io::copy_buf(&mut serial_tx, &mut serial_rx).await?
}
```
