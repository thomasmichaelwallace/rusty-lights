#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{PIN_14, PIN_25};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _}; // global logger

fn toggle(led: &mut Output<'_, PIN_25>) {
    if led.is_set_high() {
        led.set_low();
    } else {
        led.set_high();
    }
}

async fn unit(green_led: &mut Output<'_, PIN_25>, count: usize) {
    for _ in 0..count {
        toggle(green_led);
        Timer::after_millis(132).await;
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut red_led: Output<'_, PIN_14> = Output::new(p.PIN_14, Level::Low);
    let green_led: &mut Output<'_, PIN_25> = &mut Output::new(p.PIN_25, Level::Low);

    let message = "-- . .-. .-. -.-- / -.-. .... .-. .. ... - -- .- ...";

    for c in message.chars() {
        match c {
            '.' => {
                red_led.set_high();
                unit(green_led, 1).await;
                red_led.set_low();
                unit(green_led, 1).await;
            }
            '-' => {
                red_led.set_high();
                unit(green_led, 1).await;
                red_led.set_low();
                unit(green_led, 1).await;
            }
            ' ' => {
                // minus one for tail of - and .
                unit(green_led, 3 - 1).await;
            }
            '/' => {
                // minus spaces either side and tail of -/.
                unit(green_led, 7 - ((3 - 1) + (3 - 1) + 1)).await;
            }
            _ => {}
        }
    }
}
