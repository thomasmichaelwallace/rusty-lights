#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_rp::gpio::{AnyPin, Input, Level, Output, Pin, Pull};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

async fn set_level_all(lights: &mut [Output<'_, AnyPin>], level: Level) {
    for i in lights.iter_mut() {
        i.set_level(level);
    }
}

async fn winning_flash(lights: &mut [Output<'_, AnyPin>]) {
    for _t in 1..5 {
        set_level_all(lights, Level::High).await;
        Timer::after_millis(100).await;
        set_level_all(lights, Level::Low).await;
        Timer::after_millis(100).await;
    }
}

async fn game_over(lights: &mut [Output<'_, AnyPin>], at_index: usize) {
    set_level_all(lights, Level::Low).await;
    for i in lights.iter_mut().rev().skip(4 - at_index) {
        i.set_high();
        Timer::after_millis(50).await;
        i.set_low();
    }
}

async fn play_game(lights: &mut [Output<'_, AnyPin>], button: &mut Input<'_, AnyPin>, interval_ms: u64) -> bool {
    // blank
    set_level_all(lights, Level::Low).await;

    // move up
    for (i, light) in lights.iter_mut().enumerate() {
        light.set_high();

        let wait_time = Timer::after_millis(interval_ms);
        let wait_button = button.wait_for_high();

        let result = select(wait_button, wait_time).await;
        match result {
            Either::First(_) => {
                if i == 4 {
                    winning_flash(lights).await;
                    return true;
                } else {
                    game_over(lights, i).await;
                    return false;
                }
            }
            Either::Second(_) => {}
        }
        light.set_low();
    }
    // game_over(lights, 4).await;
    return false;
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let seg_1 = Output::new(p.PIN_13.degrade(), Level::Low);
    let seg_2 = Output::new(p.PIN_12.degrade(), Level::Low);
    let seg_3 = Output::new(p.PIN_11.degrade(), Level::Low);
    let seg_4 = Output::new(p.PIN_10.degrade(), Level::Low);
    let seg_5 = Output::new(p.PIN_9.degrade(), Level::Low);

    let mut lights = [seg_5, seg_4, seg_3, seg_2, seg_1];
    let mut green_button = Input::new(p.PIN_2.degrade(), Pull::Down);

    // blank
    set_level_all(&mut lights, Level::Low).await;

    // test
    let mut level = 1;
    let mut interval = 500.0;
    info!("starting level {}", level);
    loop {
        let result = play_game(&mut lights, &mut green_button, interval as u64).await;
        if result {
            interval *= 0.9;
            level += 1;
            info!("starting level {}", level);
        }
        Timer::after_millis(500).await;
    }
}
