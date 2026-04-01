#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Ticker};
use panic_probe as _;
use read_gpio::{hardware::init::init_rc, signal::processing::Processing};

const INTERVAL_US: u64 = 1000;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let (mut context, mut radio) = init_rc(spawner);
    radio.cdc.wait_connection().await; // Wait for USB host to connect

    let window: f32 = 30.0;
    let mut processing = Processing::new(window);

    let mut ticker = Ticker::every(Duration::from_micros(INTERVAL_US));

    // useless comment

    loop {
        ticker.next().await;

        let axes = context.read_oversample();
        let axes = processing.process(axes);
        radio.send_serial(axes).await;
    }
}
