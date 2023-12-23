#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod neopixel;

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pin, Pull};
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use smart_leds::RGB8;
use {defmt_rtt as _, panic_probe as _}; // global logger

use crate::neopixel::neopixel::Ws2812;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let Pio { mut common, sm0, .. } = Pio::new(p.PIO0, Irqs);
    let mut strip_leds = Ws2812::new(&mut common, sm0, p.DMA_CH1, p.PIN_2);

    let mut block_led = Output::new(p.PIN_6, Level::Low);

    let key_1 = Input::new(p.PIN_11.degrade(), Pull::Down);
    let key_2 = Input::new(p.PIN_10.degrade(), Pull::Down);
    let key_3 = Input::new(p.PIN_13.degrade(), Pull::Down);
    let key_4 = Input::new(p.PIN_12.degrade(), Pull::Down);
    let keys = [key_1, key_2, key_3, key_4];

    const NUM_LEDS: usize = 15;

    let colour_options = [
        RGB8 { r: 0, g: 0, b: 20 },
        RGB8 { r: 0, g: 20, b: 0 },
        RGB8 { r: 20, g: 0, b: 0 },
        RGB8 { r: 20, g: 20, b: 20 },
    ];

    let mut colour = RGB8 { r: 0, g: 0, b: 0 };
    loop {
        block_led.set_low();
        for i in 0..keys.len() {
            if keys[i].is_high() {
                block_led.set_high();

                info!("Key {} is pressed", i + 1);
                colour = colour_options[i];
            }
        }

        let mut colours = [RGB8::default(); NUM_LEDS];

        for i in 0..NUM_LEDS {
            colours[i] = colour;
        }
        strip_leds.write(&colours).await;
    }
}
