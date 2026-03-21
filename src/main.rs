#![no_std]
#![no_main]

use core::fmt::Write;

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Ticker};
use panic_probe as _;
use read_gpio::axis::Axis;
use read_gpio::init::init_rc;
use read_gpio::serial::BufWriter;

const INTERVAL_US: u64 = 1000;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let (mut adc, mut pin_throttle, mut pin_yaw, mut pin_pitch, mut pin_roll, mut cdc) = init_rc(spawner);

    let window: f32 = 30.0;

    let mut throttle_axis = Axis::new(window);
    let mut yaw_axis = Axis::new(window);
    let mut pitch_axis = Axis::new(window);
    let mut roll_axis = Axis::new(window);

    let mut ticker = Ticker::every(Duration::from_micros(INTERVAL_US));

    // Wait for USB host to connect
    cdc.wait_connection().await;

    let mut buf = [0u8; 64];

    loop {
        ticker.next().await;

        let throttle = throttle_axis.process(&mut adc, &mut pin_throttle);
        let yaw = yaw_axis.process(&mut adc, &mut pin_yaw);
        let pitch = pitch_axis.process(&mut adc, &mut pin_pitch);
        let roll = roll_axis.process(&mut adc, &mut pin_roll);

        // Format into buffer and send over USB CDC
        let mut wrapper = BufWriter::new(&mut buf);
        let _ = write!(wrapper, "{}, {}, {}, {}\r\n", throttle, yaw, pitch, roll);
        let len = wrapper.pos();
        let _ = cdc.write_packet(&buf[..len]).await;
    }
}
