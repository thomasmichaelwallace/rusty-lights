#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod neopixel;

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Async, Channel, Config, InterruptHandler as AdcInterruptHandler};
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

fn dim(color: RGB8, scale: f32) -> RGB8 {
    let r = ((color.r as f32) * scale) as u8;
    let g = ((color.g as f32) * scale) as u8;
    let b = ((color.b as f32) * scale) as u8;
    RGB8 { r, g, b }
}

async fn listen_wait(steps: usize, adc: &mut Adc<'_, Async>, slider_pin: &mut Channel<'static>) {
    for _ in 0..steps {
        let slide = adc.read(slider_pin).await.unwrap();
        let slide_reading = ((4096 - slide) as f32) / 4096.0;
        const LOWEST_STEP: f32 = 1.0;
        const HIGHEST_STEP: f32 = 40.0;
        let step_size = (LOWEST_STEP + slide_reading * (HIGHEST_STEP - LOWEST_STEP)) as u64;
        Timer::after_millis(step_size).await;
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello, world!");
    let p = embassy_rp::init(Default::default());

    let mut adc = Adc::new(p.ADC, Irqs, Config::default());
    let mut slider_pin = Channel::new_pin(p.PIN_28, Pull::None);

    let Pio { mut common, sm0, .. } = Pio::new(p.PIO0, Irqs);

    let mut leds = Ws2812::new(&mut common, sm0, p.DMA_CH1, p.PIN_2);

    const NUM_LEDS: usize = 15;
    let mut colours = [RGB8::default(); NUM_LEDS];

    // Loop forever making RGB values and pushing them out to the WS2812.
    let mut step: i32 = 0;
    // 0 - fade in red-first
    // 1 - hold red
    // 2 - fade out red-first
    // 3 - hold off
    // 4 - fade in green-first
    // 5 - hold green
    // 6 - fade out green-first
    // 7 - hold off
    loop {
        // let slide = adc.read(&mut slider_pin).await.unwrap();
        info!("step = {}", step);

        for i in 0..NUM_LEDS {
            let n = if step < 3 { i } else { i + 1 };
            if n % 2 == 0 {
                colours[i] = RGB8 { r: 255, g: 0, b: 0 };
            } else {
                colours[i] = RGB8 { r: 0, g: 255, b: 0 };
            }
        }

        if step == 1 || step == 5 {
            leds.write(&colours).await;
            listen_wait(75, &mut adc, &mut slider_pin).await;
        } else if step == 3 || step == 7 {
            leds.write(&[RGB8::default(); NUM_LEDS]).await;
            listen_wait(50, &mut adc, &mut slider_pin).await;
        } else {
            // fade in
            for i in 0..255 {
                let mut dimmed = [RGB8::default(); NUM_LEDS];
                let d = if (step == 0) || (step == 4) {
                    (i as f32) / 255.0
                } else {
                    (255 - i) as f32 / 255.0
                };
                for j in 0..NUM_LEDS {
                    dimmed[j] = dim(colours[j], d);
                }
                leds.write(&dimmed).await;
                listen_wait(1, &mut adc, &mut slider_pin).await;
            }
        }

        step += 1;
        if step == 8 {
            step = 0;
        }
    }
}
