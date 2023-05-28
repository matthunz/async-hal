#![no_main]
#![no_std]

use crate::hal::{
    pac::{interrupt, Interrupt, Peripherals, TIM2},
    prelude::*,
    timer::Event,
};
use async_hal::{block_on, delay::Timer, DelayMs};
use cortex_m::asm::wfi;
use cortex_m_rt::entry;
use futures::task::AtomicWaker;
use panic_halt as _;
use pin_utils::pin_mut;
use stm32f1xx_hal as hal;

macro_rules! timer {
    ($counter:expr) => {{
        static WAKER: AtomicWaker = AtomicWaker::new();

        #[interrupt]
        fn TIM2() {
            WAKER.wake();
            unsafe {
                (*TIM2::ptr())
                    .sr
                    .write(|w| w.bits(0xffff & !Event::Update.bits()));
            };
        }

        Timer::new($counter, &WAKER)
    }};
}

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let clocks = rcc
        .cfgr
        .sysclk(8.MHz())
        .pclk1(8.MHz())
        .freeze(&mut flash.acr);

    // Configure PC13 pin to blink LED
    let mut gpioc = dp.GPIOA.split();
    let mut green = gpioc.pa0.into_push_pull_output(&mut gpioc.crl);
    let mut yellow = gpioc.pa1.into_push_pull_output(&mut gpioc.crl);
    let mut red = gpioc.pa2.into_push_pull_output(&mut gpioc.crl);

    let mut timer = dp.TIM2.counter_ms(&clocks);
    timer.listen(Event::Update);
    let mut timer = timer!(timer);

    let task = async {
        loop {
            green.set_high();
            yellow.set_low();
            red.set_low();

            timer.delay_ms(100).await.unwrap();

            green.set_low();
            yellow.set_high();

            timer.delay_ms(5_000).await.unwrap();

            yellow.set_low();
            red.set_high();

            timer.delay_ms(100).await.unwrap();
        }
    };
    pin_mut!(task);

    unsafe {
        cortex_m::peripheral::NVIC::unmask(Interrupt::TIM2);
    }

    block_on(task, wfi)
}
