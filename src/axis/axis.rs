use embassy_stm32::adc::{Adc, AdcChannel, Instance};

use crate::axis::filtering::Pt3Filter;

pub struct AutoCalibAxis {
    min: u16,
    max: u16,
    pub(super) last_filtered: Option<f32>,
    pub(super) alpha: f32,
    pt3_filter: Pt3Filter,
}

impl AutoCalibAxis {
    pub fn new(window: f32, cutoff_hz: f32, sample_rate_hz: f32) -> Self {
        Self {
            min: 4095,
            max: 0,
            last_filtered: None,
            alpha: 2.0 / (window + 1.0),
            pt3_filter: Pt3Filter::new(cutoff_hz, sample_rate_hz),
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
        self.pt3_filter.update(raw)
    }

    /// warning: this normalization is not robust to potentiometer hysteresis
    fn normalize(&mut self, raw: u16) -> f32 {
        if raw < self.min {
            self.min = raw;
        }
        if raw > self.max {
            self.max = raw;
        }

        let range = self.max.saturating_sub(self.min) as f32;

        // prevent division by 0
        if range == 0.0 {
            return 0.5;
        }

        // map
        ((raw - self.min) as f32) / range
    }
}
