#![no_std]
#![no_main]

use core::fmt::Write;

use embassy_executor::Spawner;
use embassy_time::{Duration, Ticker};
use heapless::String;
use panic_rtt_target as _;
use rtt_target::rtt_init_print;

use read_gpio::axis::Axis;
use read_gpio::init::init_rc;

const INTERVAL_US: u64 = 1000;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // RTT is only used for logs over ST-LINK/SWD.
    rtt_init_print!();

    let (mut adc, mut pin_throttle, mut pin_yaw, mut pin_pitch, mut pin_roll, mut cdc) =
        init_rc(spawner);

    let window: f32 = 3.0;

    let mut throttle_axis = Axis::new(window);
    let mut yaw_axis = Axis::new(window);
    let mut pitch_axis = Axis::new(window);
    let mut roll_axis = Axis::new(window);

    let mut ticker = Ticker::every(Duration::from_micros(INTERVAL_US));

    // Wait for USB host to open the CDC port before streaming data.
    cdc.wait_connection().await;

    loop {
        ticker.next().await;

        let throttle = throttle_axis.process(&mut adc, &mut pin_throttle);
        let yaw = yaw_axis.process(&mut adc, &mut pin_yaw);
        let pitch = pitch_axis.process(&mut adc, &mut pin_pitch);
        let roll = roll_axis.process(&mut adc, &mut pin_roll);

        let mut buf: String<64> = String::new();
        let _ = write!(buf, "{}, {}, {}, {}\r\n", throttle, yaw, pitch, roll);
        let _ = cdc.write_packet(buf.as_bytes()).await;
    }
}
