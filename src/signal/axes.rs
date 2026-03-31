pub struct Axes {
    throttle: u16,
    yaw: u16,
    pitch: u16,
    roll: u16,
}

impl Axes {
    pub fn new(throttle: u16, yaw: u16, pitch: u16, roll: u16) -> Self {
        Self {
            throttle,
            yaw,
            pitch,
            roll,
        }
    }

    pub fn throttle(&self) -> u16 {
        self.throttle
    }

    pub fn yaw(&self) -> u16 {
        self.yaw
    }

    pub fn pitch(&self) -> u16 {
        self.pitch
    }

    pub fn roll(&self) -> u16 {
        self.roll
    }
}
