#![no_std]
#![no_main]

use esp_backtrace as _;
esp_bootloader_esp_idf::esp_app_desc!();
use esp_hal::{
    main,
    time::{Duration, Instant},
};
use esp_println::println;

use read_gpio::{
    filter::Filter, hardware_context::HardwareContext, radio::Radio, system_state::SystemState,
};

const LOOP_INTERVAL_MS: u64 = 10; // 100 Hz

#[main]
fn main() -> ! {
    // TODO: use the "Interrupt" approach
    let tick_rate = Duration::from_millis(LOOP_INTERVAL_MS);

    let mut context = HardwareContext::init();
    let mut current_state = SystemState::StandBy;

    let mut next_tick = Instant::now();

    println!("Joystick initialized. Reading values...");

    loop {
        // wait until the window is up
        while Instant::now() < next_tick {
            core::hint::spin_loop();
        }
        next_tick += tick_rate;

        // update system state and tick in the right mode
        match context.update_system_state(&mut current_state) {
            SystemState::Serial => context.tick_serial(),
            // TODO: make a better calibration
            SystemState::Calibration => SystemState::tick_calib(&mut context),
            SystemState::Arm => current_state.tick_standby(&mut context),
            // TODO: implement the flying mode
            SystemState::Flying(_) => Radio::send_serial(Filter::smooth(context.read_axes())),
            // TODO: check this is the behaviour we want in Disarm mode
            SystemState::Disarm => {}
        }
    }
}
