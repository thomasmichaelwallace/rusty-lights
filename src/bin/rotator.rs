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

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut adc = Adc::new(p.ADC, Irqs, Config::default());
    let mut slider_pin = Channel::new_pin(p.PIN_28, Pull::None);

    let Pio { mut common, sm0, .. } = Pio::new(p.PIO0, Irqs);

    let mut ring_led = Ws2812::new(&mut common, sm0, p.DMA_CH1, p.PIN_2);

    const NUM_LEDS: usize = 12;

    let mut i: usize = 1;
    loop {
        let slide = adc.read(&mut slider_pin).await.unwrap();
        let wait = (slide as i32) - 2048;
        let time: u64;
        if wait < 0 {
            i -= 1;
            time = (2048 + wait) as u64;
        } else {
            i += 1;
            time = (2048 - wait) as u64;
        }

        // shift i by 1 to avoid unsigned overflow
        if i > NUM_LEDS {
            i = 1;
        }
        if i < 1 {
            i = NUM_LEDS;
        }

        let mut data = [RGB8::default(); NUM_LEDS];
        data[i - 1] = RGB8 { r: 25, g: 0, b: 0 };
        ring_led.write(&data).await;

        Timer::after_millis(time).await;
    }
}
