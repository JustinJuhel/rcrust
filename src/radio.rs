use crate::axis::raw_axes::RawAxes;
use esp_println::println;

pub struct Radio {}

impl Radio {
    /// TODO: doc
    pub fn send_serial(axes: RawAxes) {
        println!(
            "{:.2}, {:.2}, {:.2}, {:.2}",
            axes.throttle(),
            axes.yaw(),
            axes.pitch(),
            axes.roll()
        );
    }
}
