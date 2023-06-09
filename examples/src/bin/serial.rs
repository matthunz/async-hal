#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use async_hal::{
    executor::{Executor, NonPending},
    io::AsyncRead,
    serial::{Reader, SerialRead},
};
use async_hal_examples as _;
use core::future::Future;
use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use defmt::println;
use stm32f1xx_hal::{
    pac::{self, interrupt, Peripherals},
    prelude::*,
    serial::{Config, Serial},
};

type App = impl Future<Output = ()>;
static mut EXECUTOR: Executor<NonPending, App> = Executor::non_pending();

#[interrupt]
fn USART3() {
    println!("USART3");
    _ = unsafe { EXECUTOR.poll() };
}

#[entry]
fn main() -> ! {
    println!("Started!");

    let dp = Peripherals::take().unwrap();

    // Setup clocks
    let mut afio = dp.AFIO.constrain();
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // Setup non-blocking serial
    let mut gpiob = dp.GPIOB.split();
    let tx = gpiob.pb10.into_alternate_push_pull(&mut gpiob.crh);
    let rx = gpiob.pb11;

    let serial = Serial::new(
        dp.USART3,
        (tx, rx),
        &mut afio.mapr,
        Config::default().baudrate(9600.bps()),
        &clocks,
    );
    let (_tx, mut rx) = serial.split();
    rx.listen();

    // Create async serial reader
    let mut reader = Reader::new(rx).into_reader();

    // Create an async task to read serial data
    let task = async move {
        loop {
            let mut buf = [0; 1];
            reader.read(&mut buf).await.unwrap();

            println!("Received: {}", &buf);
        }
    };

    // Spawn the task on the executor
    _ = unsafe {
        _ = EXECUTOR.spawn(task);
        EXECUTOR.poll()
    };

    // Enable TIM2 interrupt
    unsafe {
        NVIC::unmask(pac::Interrupt::USART3);
    }

    // Run in low-power mode
    loop {
        // TODO this is disabled for defmt
        // wfe()
    }
}
