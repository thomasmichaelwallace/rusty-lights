#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pin, Pull};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let seg_1 = Output::new(p.PIN_13.degrade(), Level::Low);
    let seg_2 = Output::new(p.PIN_12.degrade(), Level::Low);
    let seg_3 = Output::new(p.PIN_11.degrade(), Level::Low);
    let seg_4 = Output::new(p.PIN_10.degrade(), Level::Low);
    let seg_5 = Output::new(p.PIN_9.degrade(), Level::Low);

    let dip_1 = Input::new(p.PIN_6.degrade(), Pull::Down);
    let dip_2 = Input::new(p.PIN_5.degrade(), Pull::Down);
    let dip_3 = Input::new(p.PIN_4.degrade(), Pull::Down);
    let dip_4 = Input::new(p.PIN_3.degrade(), Pull::Down);
    let dip_5 = Input::new(p.PIN_2.degrade(), Pull::Down);

    let mut lights = [seg_1, seg_2, seg_3, seg_4, seg_5];
    let mut dips = [dip_1, dip_2, dip_3, dip_4, dip_5];

    loop {
        for (i, dip) in dips.iter_mut().enumerate() {
            let level = match dip.is_high() {
                true => Level::High,
                false => Level::Low,
            };
            lights[i].set_level(level);
        }
    }
}
