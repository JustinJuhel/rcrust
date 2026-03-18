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
    /// Connected to a PC
    Serial,
    /// Calibrating min, max, (center)
    Calibration,
    /// Disarmed. No reaction to joystick movement
    Disarm,
    /// Ready to start flying
    Arm,
    /// Currently flying
    Flying(FlyingMode),
}
