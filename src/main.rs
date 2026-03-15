#![no_std]
#![no_main]

use esp_backtrace as _;
esp_bootloader_esp_idf::esp_app_desc!();
use esp_hal::{
    main,
    time::{Duration, Instant},
};
use esp_println::println;

use read_gpio::{hardware_context::HardwareContext, system_state::SystemState};

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
            SystemState::Calibration => SystemState::tick_calib(&mut context),
            SystemState::StandBy => SystemState::tick_standby(&mut context),
            SystemState::Flying(flying_mode) => SystemState::tick_fly(&mut context, flying_mode),
        }
    }
}
