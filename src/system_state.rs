use esp_hal::gpio::Input;
use esp_println::println;

use crate::{axis::calibration::RcCalibration, hardware_context::HardwareContext};

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
    // // TODO: implement udpate
    // pub fn update(&mut self, pin: &Input<'_>) -> Self {
    //     let is_high = pin.is_high();

    //     match (*self, is_high) {
    //         // go to calibration (actually no I don't want to forsaken the drone mid-air)
    //         (SystemState::Flying(_mode), true) => *self = SystemState::Calibration,
    //         // default is acro mode. Or do I want to watch the flying mode pin here to directly put the right mode?
    //         (SystemState::Calibration, false) => *self = SystemState::Flying(FlyingMode::Angle),
    //         // stay in the same mode in all other configurations
    //         _ => {}
    //     }

    //     *self
    // }

    /// TODO: impl & doc
    fn get_calib_memory() -> Option<RcCalibration> {
        None
    }

    /// # Calibration mode
    /// In this version the user will move the joysticks at their extremities and the
    /// system calibrates the axes to keep the historical minimum and maximum.
    ///
    /// For now, no center because it needs to give feedback to the user.
    /// A system upgrade will include a screen to give calibration instructions.
    pub fn tick_calib(context: &mut HardwareContext) {
        let calib_axes = context.raw_axes();
        match context.calibration.as_mut() {
            Some(calibration) => calibration.update(calib_axes),
            None => context.calibration = Some(RcCalibration::new(calib_axes)),
        }
    }

    /// # Stand-by mode
    /// In this mode, the RC won't send signals to the drone. The user can get ready to fly.
    ///
    /// If the throttle axis comes all the way down **AND** there is a calibration,
    /// the system goes to `Flying` mode and the drone reacts to joystick movements.
    ///
    /// In this mode, if there is no calibration, the drone won't fly. The user needs to calibrate the RC before flying.
    pub fn tick_standby(context: &mut HardwareContext) {
        println!("IN STANDBY MODE");
        // the system starts in this mode. The user can start flying without doing a calibration.
        // But the system needs to remember the conclusion of the last calibration.
        // So we need to store the calibration results from the last calibration.
        //
        // If there is no calibration, must not be able to fly.
        //
        // Use the context.calibration

        if context.calibration.is_none() {
            if let Some(calib_memory) = Self::get_calib_memory() {
                context.calibration = Some(calib_memory.clone());
                // if throttle is down
                // this is sub-optimal because we read all axes just to get the throttle.
                // TODO: implement a read function for each axis.
                if context.raw_axes().throttle() <= calib_memory.throttle_dead_zone() {
                    // TODO: transition to Flying mode
                }
            } else {
                // do nothing
                println!("WARN: you need to calibrate the RC first!");
                return;
            }
        }
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
