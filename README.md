# async-hal
Async hardware abstraction layer for embedded devices

[![crate](https://img.shields.io/crates/v/async-hal.svg)](https://crates.io/crates/async-hal)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/async-hal)
[![CI](https://github.com/matthunz/async-hal/actions/workflows/rust.yml/badge.svg)](https://github.com/matthunz/async-hal/actions/workflows/rust.yml)

## Examples
### [Blinky](https://github.com/matthunz/async-hal/tree/main/examples/blinky)
```rust
loop {
    led.toggle();
    timer.delay_ms(1_000).await?;
}
```

### [Traffic light](https://github.com/matthunz/async-hal/tree/main/examples/traffic-light)

```rust
loop {
    green.set_high();
    red.set_low();

    timer.delay_ms(100).await?;

    green.set_low();
    yellow.set_high();

    timer.delay_ms(5_000).await?;

    yellow.set_low();
    red.set_high();

    timer.delay_ms(100).await?;
}
