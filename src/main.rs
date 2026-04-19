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
use read_gpio::init::init_rc;

const INTERVAL_US: u64 = 1000;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // RTT is only used for logs over ST-LINK/SWD.
    rtt_init_print!();

    let (
        mut adc,
        mut pin_throttle,
        mut pin_yaw,
        mut pin_pitch,
        mut pin_roll,
        mut cdc,
        mut screen,
        arm_switch,
    ) = init_rc(spawner);

    let mut throttle_axis = Axis::new();
    let mut yaw_axis = Axis::new();
    let mut pitch_axis = Axis::new();
    let mut roll_axis = Axis::new();

    let mut curr_armed: bool = false;

    cdc.wait_connection().await;

    let mut ticker = Ticker::every(Duration::from_micros(INTERVAL_US));

    loop {
        ticker.next().await;

        // Compute the axes
        let throttle = throttle_axis.process(&mut adc, &mut pin_throttle);
        let yaw = yaw_axis.process(&mut adc, &mut pin_yaw);
        let pitch = pitch_axis.process(&mut adc, &mut pin_pitch);
        let roll = roll_axis.process(&mut adc, &mut pin_roll);

        // Get the armed/disarmed state
        let new_armed = arm_switch.is_low();
        // Arm switch was just toggled
        if new_armed != curr_armed {
            // Draw only if the state changed to save computation resource
            screen.draw_disarmed(new_armed);
            curr_armed = new_armed;
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
