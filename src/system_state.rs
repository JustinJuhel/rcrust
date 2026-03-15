use esp_hal::gpio::Input;
use esp_println::println;

use crate::hardware_context::HardwareContext;

/// This is the flying mode of the drone.
/// - StandBy: the drone is not flying yet, it is waiting for the user to pull the throttle all the way down.
/// - Angle: auto-leveling mode.
/// - Acro: no auto-leveling. It's the fun mode.
#[derive(Clone, Copy)]
pub enum FlyingMode {
    Angle,
    Acro,
}

/// This is the RC state.
/// - Flying: flying the drone in Angle or Acro mode.
/// - Calibration: calibrating the RC.
#[derive(Clone, Copy)]
pub enum SystemState {
    StandBy,
    Calibration,
    Flying(FlyingMode),
}

impl SystemState {
    // TODO: implement udpate
    pub fn update(&mut self, pin: &Input<'_>) -> Self {
        let is_high = pin.is_high();

        match (*self, is_high) {
            (SystemState::Flying(_mode), true) => {
                // go to calibration (actually no I don't want to forsaken the drone mid-air)
                *self = SystemState::Calibration
            }
            (SystemState::Calibration, false) => {
                // default is acro mode. Or do I want to watch the flying mode pin here to directly put the right mode?
                *self = SystemState::Flying(FlyingMode::Angle)
            }
            // stay in the same mode in all other configurations
            _ => {}
        }

        *self
    }

    /// TODO: documentation
    pub fn tick_calib(context: &mut HardwareContext) {
        println!("IN CALIB MODE");
    }

    /// TODO: documentation
    pub fn tick_standby(context: &mut HardwareContext) {
        println!("IN STANDBY MODE");
    }

    /// TODO: documentation
    pub fn tick_fly(context: &mut HardwareContext, flying_mode: FlyingMode) {
        // For now we don't care about the flying_mode
        // We just println! the four axes

        let axes = context.axes(flying_mode);

        println!(
            "{:.2}, {:.2}, {:.2}, {:.2}",
            axes.throttle(),
            axes.yaw(),
            axes.pitch(),
            axes.roll()
        );
    }
}
