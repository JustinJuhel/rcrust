use core::f32::consts::PI;


pub(super) struct Pt3Filter {
    alpha: f32,
    s1: f32,
    s2: f32,
    s3: f32,
}

impl Pt3Filter {
    /// - cutoff_hz: the frequency where attenuation starts.
    /// - sample_rate_hz: how often we call the update function (including oversampling)
    pub(super) fn new(cutoff_hz: f32, sample_rate_hz: f32) -> Self {
        let rc = 1.0 / (2.0 * PI * cutoff_hz);
        let dt = 1.0 / sample_rate_hz;

        // smoothing factor
        // alpha = dt / (rc + dt)
        let alpha = dt / (rc + dt);

        Self {
            alpha: alpha.clamp(0.0, 1.0),
            s1: 0.0,
            s2: 0.0,
            s3: 0.0,
        }
    }

    /// process a new sample
    pub(super) fn update(&mut self, input: u16) -> u16 {
        let input_f = input as f32;
        // stage 1
        self.s1 = self.s1 + self.alpha * (input_f - self.s1);
        // stage 2
        self.s2 = self.s2 + self.alpha * (self.s1 - self.s2);
        // stage 3
        self.s3 = self.s3 + self.alpha * (self.s2 - self.s3);

        self.s3 as u16
    }
}

// impl<PIN: AdcChannel> AutoCalibAxis<PIN> {
//     pub(super) fn deadband(&mut self, ) {

//     }
// }