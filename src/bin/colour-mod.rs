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

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Start");
    let p = embassy_rp::init(Default::default());

    let Pio {
        mut common, sm0, sm1, ..
    } = Pio::new(p.PIO0, Irqs);

    // green, red, blue
    let colours: [RGB8; 12] = [
        (240, 140, 255).into(),
        (0, 255, 0).into(),
        (255, 0, 0).into(),
        (0, 0, 255).into(),
        (255, 175, 150).into(),
        (238, 223, 105).into(),
        (150, 150, 200).into(),
        (40, 100, 255).into(),
        (150, 25, 200).into(),
        (175, 150, 255).into(),
        (215, 100, 0).into(),
        (0, 0, 0).into(),
    ];
    let black: RGB8 = (0, 0, 0).into();

    let mut left_led = Ws2812::new(&mut common, sm0, p.DMA_CH0, p.PIN_5);
    let mut right_led = Ws2812::new(&mut common, sm1, p.DMA_CH1, p.PIN_2);

    left_led.write(&[black]).await;
    right_led.write(&[black]).await;

    // Loop forever making RGB values and pushing them out to the WS2812.
    loop {
        info!("writing colours");
        for i in 0..colours.len() {
            let j = if (i + 1) < colours.len() { i + 1 } else { 0 };
            info!("colour {}:{} / {}", i, j, colours.len());

            let left_colour = colours[i];
            let right_colour = colours[j];

            left_led.write(&[left_colour]).await;
            right_led.write(&[right_colour]).await;
            Timer::after_secs(1).await;
        }
    }
}
