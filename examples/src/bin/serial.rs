#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use crate::hal::{
    pac::{interrupt, Peripherals},
    prelude::*,
};
use async_hal::{
    executor::{Executor, Interrupt},
    io::{AsyncRead, AsyncWrite},
    serial::{Reader, SerialRead, SerialWrite, Writer},
};
use async_hal_examples as _;
use core::future::Future;
use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use defmt::println;
use hal::serial::{Config, Serial};
use stm32f1xx_hal::{self as hal, pac};

struct Tim2;

impl Interrupt for Tim2 {
    fn pend(&self) {
        NVIC::pend(pac::Interrupt::TIM2);
    }
}

type App = impl Future<Output = ()>;
static mut EXECUTOR: Executor<Tim2, App> = Executor::new(Tim2);

#[interrupt]
fn USART3() {
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
    let (tx, mut rx) = serial.split();
    rx.listen();

    // Create async serial reader and writer
    let mut writer = Writer::new(tx).into_writer();
    let mut reader = Reader::new(rx).into_reader();

    // Create an async task to loopback serial data
    let task = async move {
        loop {
            let mut buf = [0; 32];
            reader.read(&mut buf).await.unwrap();

            println!("Received: {}", &buf);

            writer.write_all(&buf).await.unwrap();
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
