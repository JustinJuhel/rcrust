use embassy_stm32::adc::{Adc, AdcChannel, Instance};

pub struct AutoCalibAxis {
    pub(super) last_filtered: Option<f32>,
    pub(super) alpha: f32,
}

impl AutoCalibAxis {
    pub fn new(window: f32) -> Self {
        Self {
            last_filtered: None,
            alpha: 2.0 / (window + 1.0),
        }
    }

    pub(super) fn read<T: Instance, P: AdcChannel<T>>(adc: &mut Adc<'_, T>, pin: &mut P) -> u16 {
        adc.blocking_read(pin)
    }

    pub fn process<T: Instance, P: AdcChannel<T>>(
        &mut self,
        adc: &mut Adc<'_, T>,
        pin: &mut P,
    ) -> u16 {
        let raw = self.read_oversample(adc, pin);
        self.exponential_moving_average(raw)
    }

    pub(super) fn read_oversample<T: Instance, P: AdcChannel<T>>(
        &mut self,
        adc: &mut Adc<'_, T>,
        pin: &mut P,
    ) -> u16 {
        let _trash = Self::read(adc, pin);

        let basis = 6;
        let sample = 1 << basis;
        let mut summed_raw: u32 = 0;
        let mut cnt = 0;
        while cnt < sample {
            summed_raw += Self::read(adc, pin) as u32;
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
