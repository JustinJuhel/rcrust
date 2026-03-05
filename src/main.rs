#![no_std]
#![no_main]

use esp_backtrace as _;
esp_bootloader_esp_idf::esp_app_desc!();
use esp_hal::{
    analog::adc::{Adc, AdcConfig, Attenuation},
    main, time::{Duration, Instant},
};
use esp_println::println;

use read_gpio::axis::axis::AutoCalibAxis;

const INTERVAL_US: u64 = 1000;


#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    // TODO: use the "Interrupt" approach
    let tick_rate = Duration::from_micros(INTERVAL_US);

    let window: f32 = 30.0;
    // PT3 Filter parameters
    let cutoff_hz: f32 = 15.0; // Hz
    let sample_rate_hz: f32 = 1000.0; // Hz

    let mut adc1_config = AdcConfig::new();

    // throttle, D32
    let pin_throttle = adc1_config.enable_pin(peripherals.GPIO13, Attenuation::_11dB);
    // yaw, D34
    let pin_yaw = adc1_config.enable_pin(peripherals.GPIO14, Attenuation::_11dB);
    // pitch, VP
    let pin_pitch = adc1_config.enable_pin(peripherals.GPIO4, Attenuation::_11dB);
    // roll, D33
    let pin_roll = adc1_config.enable_pin(peripherals.GPIO12, Attenuation::_11dB);

    let mut adc1 = Adc::new(peripherals.ADC1, adc1_config);

    let mut throttle_axis = AutoCalibAxis::new(
        pin_throttle,
        window,
        cutoff_hz,
        sample_rate_hz,
    );
    let mut yaw_axis = AutoCalibAxis::new(
        pin_yaw,
        window,
        cutoff_hz,
        sample_rate_hz,
    );
    let mut pitch_axis = AutoCalibAxis::new(
        pin_pitch,
        window,
        cutoff_hz,
        sample_rate_hz,
    );
    let mut roll_axis = AutoCalibAxis::new(
        pin_roll,
        window,
        cutoff_hz,
        sample_rate_hz,
    );

    let mut next_tick = Instant::now();

    println!("Joystick initialized. Reading values...");

    loop {
        // wait until the window is up
        while Instant::now() < next_tick {
            core::hint::spin_loop();
        }

        next_tick += tick_rate;

        let throttle = throttle_axis.process(&mut adc1);
        let yaw = yaw_axis.process(&mut adc1);
        let pitch = pitch_axis.process(&mut adc1);
        let roll = roll_axis.process(&mut adc1);

        println!(
            "{:.2}, {:.2}, {:.2}, {:.2}",
            throttle,
            yaw,
            pitch,
            roll,
        );
    }
}
