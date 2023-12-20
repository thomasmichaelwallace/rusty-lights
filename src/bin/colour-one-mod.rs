//! This example shows powerful PIO module in the RP2040 chip to communicate with WS2812 LED modules.
//! See (https://www.sparkfun.com/categories/tags/ws2812)

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod neopixel;

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_time::Timer;
use smart_leds::RGB8;
use {defmt_rtt as _, panic_probe as _};

use crate::neopixel::neopixel::Ws2812;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

/// Input a value 0 to 255 to get a color value
/// The colours are a transition r - g - b - back to r.
fn wheel(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return (255 - wheel_pos * 3, 0, wheel_pos * 3).into();
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return (0, wheel_pos * 3, 255 - wheel_pos * 3).into();
    }
    wheel_pos -= 170;
    (wheel_pos * 3, 255 - wheel_pos * 3, 0).into()
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Start");
    let p = embassy_rp::init(Default::default());

    let Pio { mut common, sm0, .. } = Pio::new(p.PIO0, Irqs);

    let mut left_colour = [RGB8::default(); 1];
    let mut right_colour = [RGB8::default(); 1];

    let mut left_led = Ws2812::new(&mut common, sm0, p.DMA_CH0, p.PIN_5);
    // let mut right_led = Ws2812::new(&mut common, sm1, p.DMA_CH1, p.PIN_2);

    // Loop forever making RGB values and pushing them out to the WS2812.
    loop {
        for j in 0..(256 * 5) {
            debug!("New Colors:");
            for i in 0..2 {
                let colour = wheel((((i * 256) as u16 / 2 as u16 + j as u16) & 255) as u8);
                if i == 0 {
                    debug!("left R: {} G: {} B: {}", colour.r, colour.g, colour.b);
                    left_colour[0] = colour;
                } else {
                    debug!("right R: {} G: {} B: {}", colour.r, colour.g, colour.b);
                    right_colour[0] = colour;
                }
            }
            left_led.write(&left_colour).await;
            // right_led.write(&right_colour).await;

            Timer::after_millis(10).await;
        }
    }
}
