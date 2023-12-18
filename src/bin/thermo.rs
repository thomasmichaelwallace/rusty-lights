#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod neopixel;

use defmt::*;
use dht20::Dht20;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::i2c::{Config, I2c};
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_time::{Delay, Timer};
use smart_leds::RGB8;
use {defmt_rtt as _, panic_probe as _}; // global logger

use crate::neopixel::neopixel::Ws2812;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
});

fn gradient(n: f32, dim: f32) -> RGB8 {
    const SAFE: f32 = 0.2;
    if n < 0.0 {
        RGB8::default()
    } else if n < (0.5 - SAFE / 2.0) {
        RGB8 {
            r: 0,
            g: 0,
            b: (255.0 * dim) as u8,
        }
    } else if n < (0.5 + SAFE / 2.0) {
        RGB8 {
            r: 0,
            g: (255.0 * dim) as u8,
            b: 0,
        }
    } else if n <= 1.0 {
        RGB8 {
            r: (255.0 * dim) as u8,
            g: 0,
            b: 0,
        }
    } else {
        RGB8::default()
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // led rings
    let Pio { mut common, sm0, .. } = Pio::new(p.PIO0, Irqs);
    let mut ring_led = Ws2812::new(&mut common, sm0, p.DMA_CH1, p.PIN_2);
    const NUM_LEDS: usize = 12;

    // temperature sensor
    info!("setting up i2c");
    let sda = p.PIN_14;
    let scl = p.PIN_15;
    let i2c = I2c::new_blocking(p.I2C1, scl, sda, Config::default());
    let mut sensor = Dht20::new(i2c, 0x38, Delay);

    const LOW_DEG: f32 = 15.0;
    const HIGH_DEG: f32 = 25.0;

    loop {
        let reading = sensor.read().unwrap();
        let temp = reading.temp;
        info!("Temp: {} °C", temp);

        let r = ((temp - LOW_DEG) / (HIGH_DEG - LOW_DEG)) * (NUM_LEDS as f32);
        let n: usize;
        if r < 0.0 {
            n = 0;
        } else if r > (NUM_LEDS as f32) {
            n = NUM_LEDS;
        } else {
            n = r as usize;
        }

        info!("n: {}", n);

        let mut data = [RGB8::default(); NUM_LEDS];
        for i in 0..n {
            data[i] = gradient((i as f32) / (NUM_LEDS as f32), 0.1);
            // data[i] = RGB8 { r: 25, g: 0, b: 0 };
        }
        ring_led.write(&data).await;

        Timer::after_millis(1000).await;
    }
}
