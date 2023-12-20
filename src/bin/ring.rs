#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod neopixel;

use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Channel, Config, InterruptHandler as AdcInterruptHandler};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::Pull;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_time::Timer;
use smart_leds::RGB8;
use {defmt_rtt as _, panic_probe as _}; // global logger

use crate::neopixel::neopixel::Ws2812;

bind_interrupts!(struct Irqs {
    ADC_IRQ_FIFO => AdcInterruptHandler;
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
});

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

fn dim(color: RGB8, scale: f32) -> RGB8 {
    let r = ((color.r as f32) * scale) as u8;
    let g = ((color.g as f32) * scale) as u8;
    let b = ((color.b as f32) * scale) as u8;
    RGB8 { r, g, b }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut adc = Adc::new(p.ADC, Irqs, Config::default());
    let mut slider_pin = Channel::new_pin(p.PIN_28, Pull::None);

    let Pio { mut common, sm0, .. } = Pio::new(p.PIO0, Irqs);

    let mut ring_led = Ws2812::new(&mut common, sm0, p.DMA_CH1, p.PIN_2);

    const NUM_LEDS: usize = 12;
    let mut data = [RGB8::default(); NUM_LEDS];

    // Loop forever making RGB values and pushing them out to the WS2812.
    loop {
        for j in 0..(256 * 5) {
            let slide = adc.read(&mut slider_pin).await.unwrap();
            let dimness = (slide as f32) / 4096.0;
            // info!("Dimness: {} / {}", slide, dimness);

            // debug!("New Colors:");
            for i in 0..NUM_LEDS {
                data[i] = dim(
                    wheel((((i * 256) as u16 / NUM_LEDS as u16 + j as u16) & 255) as u8),
                    dimness,
                );
                // debug!("R: {} G: {} B: {}", data[i].r, data[i].g, data[i].b);
            }
            ring_led.write(&data).await;

            Timer::after_millis(10).await;
        }
    }
}
