/// # System Mode
/// This is the **System Mode**.
/// - `Serial`: the RC is supposed to be connected to a computer and sends its measurements via Serial with few algorithms.
/// - `Radio`: the RC is in the real world mode. It can pair to a drone, calibrate itself, be armed/disarmed, and fly a drone.
pub enum SystemMode {
    Serial,
    Radio(RadioState),
}

/// # Radio State
/// This is the **Radio State**, i.e. the system substate in the `Radio` mode.
/// - `Disarmed`: the default radio state. The drone (if a drone is paired) won't react to joystick movement.
/// - `Pairing`: the remote controller is trying to pair its radio with a nearby drone.
/// - `Calibrating`: the remote controller is being calibrated by the user.
/// - `Armed`: the user is ready to fly but the drone doesn't react to joystick movements yet. The user has to pull throttle all the way down to enter `Flying` mode.
/// - `Flying`: the drone is flying. It reacts to joystick movements. The RC and the drone use smoothing and stabilization algorithms depending in the `FlightMode`.
pub enum RadioState {
    Disarmed,
    Pairing,
    Calibrating,
    Armed,
    Flying(FlightMode),
}

/// # Flight Mode
/// - `Angle`: the drone react smoothly. It uses its IMU to stabilize itself.
/// - `Acro`: the freestyle mode.
pub enum FlightMode {
    Angle,
    Acro,
}
