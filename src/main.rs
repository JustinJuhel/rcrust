#![no_std]
#![no_main]

use core::mem::Discriminant;

use esp_backtrace as _;
esp_bootloader_esp_idf::esp_app_desc!();
use esp_hal::{
    main,
    time::{Duration, Instant},
};
use esp_println::println;

use read_gpio::{
    filter::Filter,
    hardware_context::HardwareContext,
    radio::Radio,
    system_mode::{RadioState, SystemMode, SystemState},
};

const LOOP_INTERVAL_MS: u64 = 10; // 100 Hz

#[main]
fn main() -> ! {
    // TODO: use the "Interrupt" approach
    let tick_rate = Duration::from_millis(LOOP_INTERVAL_MS);

    let mut context = HardwareContext::init();
    // Serial by default (later we will switch to have the mode Radio by default)
    let mut current_state = SystemMode::Serial;
    // define the filter and its window (for EMA smoothing)
    let window: f32 = 30.0;
    let mut filter = Filter::new(window);

    let mut next_tick = Instant::now();

    println!("Joystick initialized. Reading values...");

    loop {
        // wait until the window is up
        while Instant::now() < next_tick {
            core::hint::spin_loop();
        }
        next_tick += tick_rate;

        // update system state and tick in the right mode
        match context.update_system_mode(&mut current_state) {
            SystemMode::Serial => {
                let axes = context.read_axes(); // oversample? try without
                let axes = filter.smooth(axes); // double-check it's necessary
                Radio::send_serial(axes);
            }
            SystemMode::Radio(RadioState::Disarmed | RadioState::Armed) => { /* Do nothing */ }
            SystemMode::Radio(RadioState::Pairing) => { /* TODO: pairing workflow */ }
            SystemMode::Radio(RadioState::Calibrating) => { /* TODO: calibration workflow */ }
            SystemMode::Radio(RadioState::Flying(flight_mode)) => {
                let axes = context.read_axes_oversample(5);
                let filtered = Filter::process(axes);
                Radio::send_radio(axes);
            }
        }
    }
}
