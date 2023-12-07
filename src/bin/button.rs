#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::*;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _}; // global logger

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut red_led: Output<'_, PIN_14> = Output::new(p.PIN_14, Level::Low);
    let mut green_led: Output<'_, PIN_25> = Output::new(p.PIN_25, Level::Low);

    let red_button: Input<'_, PIN_3> = Input::new(p.PIN_3, Pull::Down);
    let green_button: Input<'_, PIN_2> = Input::new(p.PIN_2, Pull::Down);

    info!("Button test started");
    loop {
        if red_button.is_high() {
            red_led.set_high();
        } else {
            red_led.set_low();
        }

        if green_button.is_high() {
            green_led.set_high();
        } else {
            green_led.set_low();
        }

        Timer::after_millis(10).await;
    }
}
