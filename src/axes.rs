/// Safer than using a `Vec<u16>` because here I know I won't confuse axes.
#[derive(Clone, Copy)]
pub struct Axes {
    throttle: u16,
    yaw: u16,
    pitch: u16,
    roll: u16,
}

impl Axes {
    /// TODO: documentation
    pub fn new(throttle: u16, yaw: u16, pitch: u16, roll: u16) -> Axes {
        Self {
            throttle,
            yaw,
            pitch,
            roll,
        }
    }

    /// TODO: documentation
    pub fn throttle(&self) -> u16 {
        self.throttle
    }

    /// TODO: documentation
    pub fn yaw(&self) -> u16 {
        self.yaw
    }

    /// TODO: documentation
    pub fn pitch(&self) -> u16 {
        self.pitch
    }

    /// TODO: documentation
    pub fn roll(&self) -> u16 {
        self.roll
    }
}
