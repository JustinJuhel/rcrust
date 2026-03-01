use esp_hal::{Blocking, analog::adc::{Adc, AdcChannel}, peripherals::ADC1};

use crate::axis::axis::AutoCalibAxis;


impl<PIN: AdcChannel> AutoCalibAxis<PIN> {
    /// Read the analog pin multiple times and average them.
    /// The goal is to overcome the sensor jitter.
    pub(super) fn read_oversample(&mut self, adc: &mut Adc<'static, ADC1<'static>, Blocking>) -> u16 {
        let sample = 16;
        let mut summed_raw = 0;
        let mut cnt = 0;
        while cnt < sample {
            summed_raw += self.read(adc);
            cnt += 1;
        }
        // bit shift by 4 is much faster than division by 16
        summed_raw >> 4
    }

    /// EMA (exponential moving average) filter. Computationally cheap. Smoothes out sensor jitter.
    pub(super) fn exponential_moving_average(&mut self, raw: u16) -> u16 {
        let current = raw as f32;

        let filtered = match self.last_filtered {
            Some(prev) => prev + self.alpha * (current - prev),
            None => current,
        };

        self.last_filtered = Some(filtered);
        // cast back to integer for output
        filtered as u16
    }
}