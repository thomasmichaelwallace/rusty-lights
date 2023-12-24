#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod neopixel;

use ag_lcd::{Blink, Cursor, LcdDisplay};
use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::i2c::{Config, I2c};
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
use embassy_time::{Delay, Timer};
use port_expander::Pcf8574;
use shared_bus::BusManagerSimple;
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
    let colours = [RGB8::default(); NUM_LEDS];
    strip_leds.write(&colours).await;

    info!("connecting up lcd");
    let sda = p.PIN_14;
    let scl = p.PIN_15;
    let i2c = I2c::new_blocking(p.I2C1, scl, sda, Config::default());
    let manager = BusManagerSimple::new(i2c);

    let mut i2c_expander = Pcf8574::new(manager.acquire_i2c(), true, true, true);
    let mut lcd: LcdDisplay<_, _> = LcdDisplay::new_pcf8574(&mut i2c_expander, Delay)
        .with_blink(Blink::Off)
        .with_cursor(Cursor::Off)
        .with_cols(16)
        .with_layout(ag_lcd::Layout::LeftToRight)
        .with_lines(ag_lcd::Lines::TwoLines)
        .build();

    lcd.clear();
    Timer::after_secs(2).await;

    // 0123456789012345
    // SHOCKING BASH IN
    //      3 ...
    lcd.print("SHOCKING BASH IN");
    let nos = ['3', '2', '1'];
    let dot = '.' as u8;
    let interval = 1000;
    for i in 0..nos.len() {
        let mut buffer = [nos[i], ' ', ' ', ' '].map(|c| c as u8);
        lcd.set_position(5, 1);
        lcd.print(core::str::from_utf8(&buffer).unwrap());

        for i in 1..buffer.len() {
            Timer::after_millis(interval).await;
            buffer[i] = dot;

            lcd.set_position(5, 1);
            lcd.print(core::str::from_utf8(&buffer).unwrap());
        }
        Timer::after_millis(interval).await;
    }

    let buffer = ['#', '$', '&', '!'].map(|c| c as u8);
    lcd.set_position(5, 1);
    lcd.print(core::str::from_utf8(&buffer).unwrap());

    for i in 0..11 {
        let color = if i % 2 == 0 {
            RGB8 { r: 20, g: 20, b: 20 }
        } else {
            RGB8 { r: 0, g: 0, b: 0 }
        };
        strip_leds.write(&[color; NUM_LEDS]).await;
        Timer::after_millis(100).await;
    }

    info!("end of lcd test");
}
