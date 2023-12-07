#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::*;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _}; // global logger

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut block_led: Output<'_, PIN_14> = Output::new(p.PIN_14, Level::Low);
    let mut led: Output<'_, PIN_25> = Output::new(p.PIN_25, Level::Low);

    for n in 1..5 {
        info!("flash {}", n);
        block_led.set_high();
        led.set_low();
        Timer::after_secs(1).await;
        block_led.set_low();
        led.set_high();
        Timer::after_secs(1).await;
    }
}
