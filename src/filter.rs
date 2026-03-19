use crate::{axes, axis::raw_axes::RawAxes};

#[derive(Clone)]
struct FloatAxes {
    throttle: f32,
    yaw: f32,
    pitch: f32,
    roll: f32,
}

pub struct Filter {
    /// Used in the EMA filter
    last_filtered: Option<FloatAxes>,
    /// Used in the EMA filter. If we want a different sensibility on all axes, we can use a custom type.
    alpha: f32,
}

impl Filter {
    /// TODO: doc
    pub fn new(window: f32) -> Self {
        Self {
            last_filtered: None,
            alpha: 2.0 / (window + 1.0),
        }
    }

    /// TODO: impl & doc
    pub fn process(axes: RawAxes) -> RawAxes {
        axes
    }

    /// # Smooth Signal
    /// This function applies a smoothing filter to the raw data to eliminate jitter and electrical noise.
    pub fn smooth(&mut self, axes: RawAxes) -> RawAxes {
        self.exponential_moving_average(axes)
    }

    /// # Exponential Moving Average filter
    /// Computationally cheap. Smoothes out sensor jitter.
    pub fn exponential_moving_average(&mut self, current_axes: RawAxes) -> RawAxes {
        // get or initialize the last_filtered value
        let prev = self.last_filtered.get_or_insert_with(|| FloatAxes {
            throttle: current_axes.throttle() as f32,
            yaw: current_axes.yaw() as f32,
            pitch: current_axes.pitch() as f32,
            roll: current_axes.roll() as f32,
        });

        // update the values in place
        prev.throttle += self.alpha * (current_axes.throttle() as f32 - prev.throttle);
        prev.yaw += self.alpha * (current_axes.yaw() as f32 - prev.yaw);
        prev.pitch += self.alpha * (current_axes.pitch() as f32 - prev.pitch);
        prev.roll += self.alpha * (current_axes.roll() as f32 - prev.roll);

        RawAxes::new(
            prev.throttle as u16,
            prev.yaw as u16,
            prev.pitch as u16,
            prev.roll as u16,
        )
    }
}
