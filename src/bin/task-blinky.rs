#![no_std]
#![no_main]
// required by embassy
#![feature(type_alias_impl_trait)]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::PIN_25;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _}; // global logger

#[embassy_executor::task]
async fn blinker(mut led: Output<'static, PIN_25>) {
    loop {
        info!("turning led on!");
        led.set_high();
        Timer::after_secs(1).await;
        info!("turning led off!");
        led.set_low();
        Timer::after_secs(1).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let led = Output::new(p.PIN_25, Level::Low);
    unwrap!(spawner.spawn(blinker(led)))
}
