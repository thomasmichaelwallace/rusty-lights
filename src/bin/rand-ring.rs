#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod neopixel;

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Channel, Config, InterruptHandler as AdcInterruptHandler};
use embassy_rp::bind_interrupts;
use embassy_rp::clocks::RoscRng;
use embassy_rp::gpio::Pull;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_time::Timer;
use rand::RngCore;
use smart_leds::RGB8;
use {defmt_rtt as _, panic_probe as _}; // global logger

use crate::neopixel::neopixel::Ws2812;

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => AdcInterruptHandler;
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
});

fn dim(color: RGB8, scale: f32) -> RGB8 {
    let r = ((color.r as f32) * scale) as u8;
    let g = ((color.g as f32) * scale) as u8;
    let b = ((color.b as f32) * scale) as u8;
    RGB8 { r, g, b }
}

fn rnd_colour(rng: &mut RoscRng) -> RGB8 {
    let r = rng.next_u32() as u8;
    let g = rng.next_u32() as u8;
    let b = rng.next_u32() as u8;
    RGB8 { r, g, b }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut adc = Adc::new(p.ADC, Irqs, Config::default());
    let mut slider_pin = Channel::new_pin(p.PIN_28, Pull::None);

    let Pio { mut common, sm0, .. } = Pio::new(p.PIO0, Irqs);

    let mut ring_led = Ws2812::new(&mut common, sm0, p.DMA_CH1, p.PIN_2);

    let mut rng = RoscRng;

    const NUM_LEDS: usize = 12;
    let mut colours = [RGB8::default(); NUM_LEDS];

    // Loop forever making RGB values and pushing them out to the WS2812.
    loop {
        let slide = adc.read(&mut slider_pin).await.unwrap();
        let dimness = (slide as f32) / 4096.0;

        let i = rng.next_u32() as usize % NUM_LEDS;
        colours[i] = rnd_colour(&mut rng);

        let mut data = [RGB8::default(); NUM_LEDS];
        for j in 0..NUM_LEDS {
            let c = dim(colours[j], dimness);
            data[j] = c;
        }

        ring_led.write(&data).await;

        Timer::after_millis(50).await;
    }
}
