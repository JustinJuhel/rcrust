use crate::axis::raw_axes::RawAxes;

#[derive(Clone)]
struct AxisCalibration {
    min: u16,
    max: u16,
    // center: u16,
}

impl AxisCalibration {
    /// TODO: doc
    pub fn new(first_value: u16) -> AxisCalibration {
        AxisCalibration {
            min: first_value,
            max: first_value,
        }
    }
}

impl Default for AxisCalibration {
    fn default() -> Self {
        AxisCalibration {
            min: 4096,
            max: 0,
            // center: 2048,
        }
    }
}

#[derive(Clone)]
pub struct RcCalibration {
    throttle: AxisCalibration,
    yaw: AxisCalibration,
    pitch: AxisCalibration,
    roll: AxisCalibration,
}

impl RcCalibration {
    /// TODO: doc
    pub fn new(axes: RawAxes) -> Self {
        RcCalibration {
            throttle: AxisCalibration::new(axes.throttle()),
            yaw: AxisCalibration::new(axes.yaw()),
            pitch: AxisCalibration::new(axes.pitch()),
            roll: AxisCalibration::new(axes.roll()),
        }
    }

    /// TODO: doc
    pub fn update(&mut self, axes: RawAxes) {
        // update throttle min or max
        if axes.throttle() < self.throttle.min {
            self.throttle.min = axes.throttle();
        } else if axes.throttle() > self.throttle.max {
            self.throttle.max = axes.throttle()
        }
        // update yaw min or max
        if axes.yaw() < self.yaw.min {
            self.yaw.min = axes.yaw();
        } else if axes.yaw() > self.yaw.max {
            self.yaw.max = axes.yaw()
        }
        // update pitch min or max
        if axes.pitch() < self.pitch.min {
            self.pitch.min = axes.pitch();
        } else if axes.pitch() > self.pitch.max {
            self.pitch.max = axes.pitch()
        }
        // update roll min or max
        if axes.roll() < self.roll.min {
            self.roll.min = axes.roll();
        } else if axes.roll() > self.roll.max {
            self.roll.max = axes.roll()
        }
    }

    /// TODO: impl & doc
    pub fn throttle_dead_zone(&self) -> u16 {
        0
    }
}
