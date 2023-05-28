#![no_main]
#![no_std]

use crate::hal::{
    pac::{interrupt, Interrupt, Peripherals, TIM2},
    prelude::*,
    timer::Event,
};
use async_hal::{Executor, Timer};
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
    let mut gpioc = dp.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    let mut timer = dp.TIM2.counter_ms(&clocks);
    timer.listen(Event::Update);
    let mut timer = timer!(timer);

    let task = async {
        loop {
            led.toggle();
            timer.wait(1.secs()).await;
        }
    };
    pin_mut!(task);

    let mut executor = Executor::<_, 1>::take().unwrap();
    executor.spawn(task);

    unsafe {
        cortex_m::peripheral::NVIC::unmask(Interrupt::TIM2);
    }

    loop {
        while executor.run().is_some() {}

        wfi();
    }
}
