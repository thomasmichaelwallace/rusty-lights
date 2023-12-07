#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select::{select3, Either3};
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::*;
use embassy_time::Timer;
use rand::RngCore;
use {defmt_rtt as _, panic_probe as _}; // global logger

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut red_led: Output<'_, PIN_14> = Output::new(p.PIN_14, Level::Low);
    let mut green_led: Output<'_, PIN_25> = Output::new(p.PIN_25, Level::Low);

    let mut red_button: Input<'_, PIN_3> = Input::new(p.PIN_3, Pull::Down);
    let mut green_button: Input<'_, PIN_2> = Input::new(p.PIN_2, Pull::Down);

    let mut rng = RoscRng;
    let mut score = 0;

    info!("light-game started");
    for _ in 0..10 {
        red_led.set_low();
        green_led.set_low();
        Timer::after_millis(500).await;

        let seed = rng.next_u64();
        let is_red = seed % 2 == 0;
        info!("is_red: {}", is_red);
        if is_red {
            red_led.set_high();
        } else {
            green_led.set_high();
        }

        let wait_red_button = red_button.wait_for_rising_edge();
        let wait_green_button = green_button.wait_for_rising_edge();
        let timer = Timer::after_millis(1000);
        let result = select3(wait_red_button, wait_green_button, timer).await;

        match result {
            Either3::First(_) => {
                if is_red {
                    info!("[yes] red light caught");
                    score += 1;
                } else {
                    info!("[no] red light lost");
                    score -= 1;
                }
            }
            Either3::Second(_) => {
                info!("green button pressed");
                if !is_red {
                    info!("[yes] green light caught");
                    score += 1;
                } else {
                    info!("[no] green light lost");
                    score -= 1;
                }
            }
            Either3::Third(_) => {
                info!("[no] light missed");
                score -= 2;
            }
        }
    }
    info!("light-game ended");
    info!("score: {}", score);
}
