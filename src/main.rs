#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_time::{Duration, Ticker};
use panic_probe as _;

use read_gpio::axis::axis::AutoCalibAxis;

const INTERVAL_US: u64 = 1000;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Config::default());

    let mut adc = Adc::new(p.ADC1);
    adc.set_sample_time(SampleTime::CYCLES480); // slow sample = more accuracy
    adc.set_resolution(embassy_stm32::adc::Resolution::BITS12);

    let mut pin_throttle = p.PA0;
    let mut pin_yaw = p.PA1;
    let mut pin_pitch = p.PA4;
    let mut pin_roll = p.PA5;

    let cutoff_hz: f32 = 15.0;
    let sample_rate_hz: f32 = 1000.0;
    let window: f32 = 30.0;

    let mut throttle_axis = AutoCalibAxis::new(window, cutoff_hz, sample_rate_hz);
    let mut yaw_axis = AutoCalibAxis::new(window, cutoff_hz, sample_rate_hz);
    let mut pitch_axis = AutoCalibAxis::new(window, cutoff_hz, sample_rate_hz);
    let mut roll_axis = AutoCalibAxis::new(window, cutoff_hz, sample_rate_hz);

    let mut ticker = Ticker::every(Duration::from_micros(INTERVAL_US));

    info!("Joystick initialized. Reading values...");

    loop {
        ticker.next().await;

        let throttle = throttle_axis.process(&mut adc, &mut pin_throttle);
        let yaw = yaw_axis.process(&mut adc, &mut pin_yaw);
        let pitch = pitch_axis.process(&mut adc, &mut pin_pitch);
        let roll = roll_axis.process(&mut adc, &mut pin_roll);

        info!("{}, {}, {}, {}", throttle, yaw, pitch, roll);
    }
}
