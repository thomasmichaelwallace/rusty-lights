#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{self, Pin};
use embassy_time::Timer;
use gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let seg_1 = Output::new(p.PIN_13.degrade(), Level::Low);
    let seg_2 = Output::new(p.PIN_12.degrade(), Level::Low);
    let seg_3 = Output::new(p.PIN_11.degrade(), Level::Low);
    let seg_4 = Output::new(p.PIN_10.degrade(), Level::Low);
    let seg_5 = Output::new(p.PIN_9.degrade(), Level::Low);

    let mut graph = [seg_1, seg_2, seg_3, seg_4, seg_5];

    // blank
    for i in graph.iter_mut() {
        i.set_low();
    }

    // do the first light
    graph[0].set_high();
    Timer::after_millis(100).await;
    graph[0].set_low();

    // bump back and forth
    loop {
        for i in graph.iter_mut().skip(1) {
            i.set_high();
            Timer::after_millis(100).await;
            i.set_low();
        }
        for i in graph.iter_mut().rev().skip(1) {
            i.set_high();
            Timer::after_millis(100).await;
            i.set_low();
        }
    }
}
