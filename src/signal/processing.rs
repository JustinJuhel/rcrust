use crate::signal::axes::Axes;

pub struct Processing {
    last_filtered_throttle: Option<f32>,
    last_filtered_yaw: Option<f32>,
    last_filtered_pitch: Option<f32>,
    last_filtered_roll: Option<f32>,
    alpha: f32,
}

impl Processing {
    pub fn new(window: f32) -> Self {
        Self {
            last_filtered_throttle: None,
            last_filtered_yaw: None,
            last_filtered_pitch: None,
            last_filtered_roll: None,
            alpha: 2.0 / (window + 1.0),
        }
    }
}

impl Processing {
    /// # Process
    /// Processes raw data:
    /// - apply an exponential moving average filter to smooth the signal.
    pub fn process(&mut self, axes: Axes) -> Axes {
        self.exponential_moving_average(axes)
    }

    /// # EMA (exponential moving average) filter.
    /// Computationally cheap. Smoothes out sensor jitter.
    pub fn exponential_moving_average(&mut self, axes: Axes) -> Axes {
        let filtered_throttle = match self.last_filtered_throttle {
            Some(prev) => prev + self.alpha * (axes.throttle() as f32 - prev),
            None => axes.throttle() as f32,
        };
        self.last_filtered_throttle = Some(filtered_throttle);

        let filtered_yaw = match self.last_filtered_yaw {
            Some(prev) => prev + self.alpha * (axes.yaw() as f32 - prev),
            None => axes.yaw() as f32,
        };
        self.last_filtered_yaw = Some(filtered_yaw);

        let filtered_pitch = match self.last_filtered_pitch {
            Some(prev) => prev + self.alpha * (axes.pitch() as f32 - prev),
            None => axes.pitch() as f32,
        };
        self.last_filtered_pitch = Some(filtered_pitch);

        let filtered_roll = match self.last_filtered_roll {
            Some(prev) => prev + self.alpha * (axes.roll() as f32 - prev),
            None => axes.roll() as f32,
        };
        self.last_filtered_roll = Some(filtered_roll);

        // cast back to integer for output
        Axes::new(
            filtered_throttle as u16,
            filtered_yaw as u16,
            filtered_pitch as u16,
            filtered_roll as u16,
        )
    }
}
