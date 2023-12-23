#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod neopixel;

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::join::join5;
use embassy_futures::select::{select4, Either4};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pin, Pull};
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_time::Timer;
use smart_leds::RGB8;
use {defmt_rtt as _, panic_probe as _}; // global logger

use crate::neopixel::neopixel::Ws2812;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    const NUM_LEDS: usize = 15;
    let Pio { mut common, sm0, .. } = Pio::new(p.PIO0, Irqs);
    let mut strip_leds = Ws2812::new(&mut common, sm0, p.DMA_CH1, p.PIN_2);
    strip_leds.write(&[RGB8::default(); NUM_LEDS]).await;

    let mut block_led = Output::new(p.PIN_6, Level::Low);
    block_led.set_low();

    let mut key_1 = Input::new(p.PIN_11.degrade(), Pull::Down);
    let mut key_2 = Input::new(p.PIN_10.degrade(), Pull::Down);
    let mut key_3 = Input::new(p.PIN_13.degrade(), Pull::Down);
    let mut key_4 = Input::new(p.PIN_12.degrade(), Pull::Down);

    let colour_options = [
        RGB8 { r: 20, g: 0, b: 0 },
        RGB8 { r: 0, g: 20, b: 0 },
        RGB8 { r: 0, g: 0, b: 20 },
        RGB8 { r: 20, g: 20, b: 20 },
    ];

    let mut index = 0;
    let mut colours = [RGB8::default(); NUM_LEDS];
    loop {
        info!("Waiting for key down");
        let key_down = select4(
            key_1.wait_for_high(),
            key_2.wait_for_high(),
            key_3.wait_for_high(),
            key_4.wait_for_high(),
        )
        .await;

        let colour_index = match key_down {
            Either4::First(_) => 0,
            Either4::Second(_) => 1,
            Either4::Third(_) => 2,
            Either4::Fourth(_) => 3,
        };
        info!("Colour index: {}", colour_index);
        colours[index] = colour_options[colour_index];
        index = (index + 1) % NUM_LEDS;

        strip_leds.write(&colours).await;
        block_led.toggle();

        info!("Waiting for reset!");
        join5(
            key_1.wait_for_low(),
            key_2.wait_for_low(),
            key_3.wait_for_low(),
            key_4.wait_for_low(),
            Timer::after_millis(500),
        )
        .await;
    }
}
