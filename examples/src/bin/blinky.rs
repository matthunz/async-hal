#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use crate::hal::{
    pac::{interrupt, Peripherals},
    prelude::*,
    timer::Event,
};
use async_hal::{
    delay::{DelayMs, Timer},
    executor::{Executor, NonPending},
};
use async_hal_examples as _;
use core::future::Future;
use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use defmt::println;
use stm32f1xx_hal::{self as hal, pac};

type App = impl Future<Output = ()>;
static mut EXECUTOR: Executor<NonPending, App> = Executor::non_pending();

#[interrupt]
fn TIM2() {
    _ = unsafe { EXECUTOR.poll() };
}

#[entry]
fn main() -> ! {
    println!("Started!");

    let dp = Peripherals::take().unwrap();

    // Configure PC13 pin to blink LED
    let mut gpioc = dp.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    // Init clocks
    let rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let clocks = rcc
        .cfgr
        .sysclk(8.MHz())
        .pclk1(8.MHz())
        .freeze(&mut flash.acr);

    // Create a counter using TIM2
    let mut counter = dp.TIM2.counter_ms(&clocks);
    counter.listen(Event::Update);
    let mut timer = Timer::new(counter);

    // Create an async task to blink the LED
    let task = async move {
        loop {
            println!("Blink!");

            led.toggle();

            timer.delay_ms(1_000).await.unwrap();
        }
    };

    // Spawn the task on the executor
    _ = unsafe {
        _ = EXECUTOR.spawn(task);
        EXECUTOR.poll()
    };

    // Enable TIM2 interrupt
    unsafe {
        NVIC::unmask(pac::Interrupt::TIM2);
    }

    // Run in low-power mode
    loop {
        // TODO this is disabled for defmt
        // wfe()
    }
}
