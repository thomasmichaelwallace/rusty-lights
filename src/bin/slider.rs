#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod neopixel;

use defmt::*;
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

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut adc = Adc::new(p.ADC, Irqs, Config::default());
    let mut slider_pin = Channel::new_pin(p.PIN_28, Pull::None);

    let Pio {
        mut common, sm0, sm1, ..
    } = Pio::new(p.PIO0, Irqs);

    let mut left_led = Ws2812::new(&mut common, sm0, p.DMA_CH0, p.PIN_5);
    let mut right_led = Ws2812::new(&mut common, sm1, p.DMA_CH1, p.PIN_2);

    loop {
        let val = adc.read(&mut slider_pin).await;

        let mut wheel_pos: u8;
        match val {
            Ok(v) => {
                wheel_pos = (v / 16) as u8;
            }
            Err(_) => {
                info!("Error reading ADC");
                wheel_pos = 0;
            }
        }

        let left_colour = wheel(wheel_pos);
        left_led.write(&[left_colour]).await;
        wheel_pos = if wheel_pos < 128 {
            wheel_pos + 128
        } else {
            wheel_pos - 128
        };
        let right_colour = wheel(wheel_pos);
        right_led.write(&[right_colour]).await;

        Timer::after_millis(50).await;
    }
}
