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
use embassy_time::Delay;
use hd44780_driver::HD44780;
use smart_leds::RGB8;
use {defmt_rtt as _, panic_probe as _};

use crate::neopixel::neopixel::Ws2812;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("connecting light strip");
    const NUM_LEDS: usize = 15;
    let Pio { mut common, sm0, .. } = Pio::new(p.PIO0, Irqs);
    let mut strip_leds = Ws2812::new(&mut common, sm0, p.DMA_CH1, p.PIN_2);
    let colour_options = [
        RGB8 { r: 20, g: 0, b: 0 },
        RGB8 { r: 0, g: 20, b: 0 },
        RGB8 { r: 0, g: 0, b: 20 },
        RGB8 { r: 20, g: 20, b: 20 },
    ];

    info!("setting light strip");
    let mut colours = [RGB8::default(); NUM_LEDS];
    for i in 0..NUM_LEDS {
        colours[i] = colour_options[i % 4];
    }
    strip_leds.write(&colours).await;

    info!("connecting up lcd");
    let sda = p.PIN_14;
    let scl = p.PIN_15;
    let i2c = I2c::new_blocking(p.I2C1, scl, sda, Config::default());
    let mut lcd = HD44780::new_i2c(i2c, 0x27, &mut Delay).unwrap();

    info!("setting lcd");
    lcd.reset(&mut Delay).unwrap();
    lcd.clear(&mut Delay).unwrap();
    lcd.write_str("oh hai ...", &mut Delay).unwrap();
    lcd.set_cursor_pos(40, &mut Delay).unwrap();
    lcd.write_str("Martin!", &mut Delay).unwrap();

    info!("setting up temperature");
    let sda_t = p.PIN_12;
    let scl_t = p.PIN_13;
    let i2c_t = I2c::new_blocking(p.I2C0, scl_t, sda_t, Config::default());
    let mut sensor = Dht20::new(i2c_t, 0x38, Delay);

    let reading = sensor.read().unwrap();
    let temp = reading.temp;
    info!("Temp: {} °C", temp);
}
