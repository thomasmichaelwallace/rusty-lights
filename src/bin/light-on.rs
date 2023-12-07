#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::PIN_25;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _}; // global logger

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut led: Output<'_, PIN_25> = Output::new(p.PIN_25, Level::Low);

    info!("hello world!");
    led.set_high();
    Timer::after_secs(10).await;
    info!("goodbye world!")
}
