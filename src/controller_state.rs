pub struct ControllerState {
    pub connection: Connection,
    pub mode: FlightMode,
    pub status: SystemStatus,
}

pub enum Connection {
    Disconnected,
    Serial,
    Radio, //(LinkQuality), holding RSSI/LQ data
}

pub enum FlightMode {
    Acro,
    Angle,
    // Horizon, ReturnToHome? Maybe in the future
}

pub enum SystemStatus {
    // setup/maintenance
    Pairing,
    Calibration,

    // active operation
    Disarmed(DisarmReason),
    Armed,
}

pub enum DisarmReason {
    /// The arm switch is in disarm position.
    Intentional,
    /// This triggers immediately after arming if the flight controller's PID loop detects a massive, uncommanded divergence between the gyro data and motor output.
    /// It almost always means the pilot put the props on backward, the motor order is mapped incorrectly, or the board alignment is wrong.
    /// It disarms the quad before it can launch itself into the pilot's face.
    RunawayTakeoff,
    // /// The radio link was lost or the quality dropped below a safety threshold during flight.
    // /// Maybe we would need a rescue GPS return in this case.
    // Failsafe,
    /// Safety reason. If the throttle is at 20%, arming would cause the drone to take off instantly.
    ThrottleNotZero,
}
