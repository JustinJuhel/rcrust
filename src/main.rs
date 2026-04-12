#![no_std]
#![no_main]

use core::fmt::Write;

use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_time::{Duration, Ticker, Timer};
use heapless::String;
use panic_rtt_target as _;
use rtt_target::rtt_init_print;

use read_gpio::axis::Axis;
use read_gpio::display::draw_axes;
use read_gpio::init::init_rc;

const INTERVAL_US: u64 = 1000;
const DISPLAY_INTERVAL: u16 = 100;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // RTT is only used for logs over ST-LINK/SWD.
    rtt_init_print!();

    let (mut adc, mut pin_throttle, mut pin_yaw, mut pin_pitch, mut pin_roll, mut cdc, mut display) =
        init_rc(spawner);

    let mut throttle_axis = Axis::new();
    let mut yaw_axis = Axis::new();
    let mut pitch_axis = Axis::new();
    let mut roll_axis = Axis::new();

    cdc.wait_connection().await;

    let mut ticker = Ticker::every(Duration::from_micros(INTERVAL_US));
    let mut display_counter: u16 = 0;

    loop {
        ticker.next().await;

        let throttle = throttle_axis.process(&mut adc, &mut pin_throttle);
        let yaw = yaw_axis.process(&mut adc, &mut pin_yaw);
        let pitch = pitch_axis.process(&mut adc, &mut pin_pitch);
        let roll = roll_axis.process(&mut adc, &mut pin_roll);

        display_counter += 1;
        if display_counter >= DISPLAY_INTERVAL {
            display_counter = 0;
            draw_axes(&mut display, throttle, yaw, pitch, roll);
        }

        let mut buf: String<64> = String::new();
        let _ = write!(buf, "{}, {}, {}, {}\r\n", throttle, yaw, pitch, roll);
        // Timeout prevents the loop from stalling if the USB host disconnects.
        let _ = select(
            cdc.write_packet(buf.as_bytes()),
            Timer::after(Duration::from_millis(2)),
        )
        .await;
    }
}
