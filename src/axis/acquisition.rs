use esp_hal::{Blocking, analog::adc::{Adc, AdcChannel}, peripherals::ADC1};

use crate::axis::axis::AutoCalibAxis;


impl<PIN: AdcChannel> AutoCalibAxis<PIN> {
    /// Read the analog pin multiple times and average them.
    /// The goal is to overcome the sensor jitter.
    /// 
    /// This function oversamples by reading 2^basis times the signal.
    pub(super) fn read_oversample(&mut self, adc: &mut Adc<'static, ADC1<'static>, Blocking>, basis: i32) -> u16 {
        // throw the first value away to avoid leftover charge in the microcontroller
        // let _trash = self.read(adc);

        // let sample = 64;
        let sample = 1 << basis; // 2^basis
        let mut summed_raw: u32 = 0;
        let mut cnt = 0;
        while cnt < sample {
            summed_raw += self.read(adc) as u32;
            cnt += 1;
        }
        // bit shift by 6 is much faster than division by 64
        // summed_raw >> 6
        (summed_raw >> basis) as u16
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