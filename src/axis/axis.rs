use esp_hal::{Blocking, analog::adc::{Adc, AdcChannel, AdcPin}, peripherals::{ADC1}};

use crate::axis::filtering::Pt3Filter;

pub struct AutoCalibAxis<PIN> {
    pin: AdcPin<PIN, ADC1<'static>>,
    min: u16,
    max: u16,
    pub(super) last_filtered: Option<f32>,
    pub(super) alpha: f32,

    pt3_filter: Pt3Filter,
}

impl<PIN: AdcChannel> AutoCalibAxis<PIN> {
    pub fn new(pin: AdcPin<PIN, ADC1<'static>>, window: f32, cutoff_hz: f32, sample_rate_hz: f32) -> Self {
        Self {
            pin,
            min: 4095,
            max: 0 ,
            last_filtered: None,
            alpha: 2.0 / (window + 1.0),
            pt3_filter: Pt3Filter::new(cutoff_hz, sample_rate_hz),
        }
    }

    pub(super) fn read(&mut self, adc: &mut Adc<'static, ADC1<'static>, Blocking>) -> u16 {
        match nb::block!(adc.read_oneshot(&mut self.pin)) {
            Ok(value) => value as u16,
            Err(_) => 2048 as u16,
        }
    }

    /// raw data processing pipeline
    pub fn process(&mut self, adc: &mut Adc<'static, ADC1<'static>, Blocking>) -> u16 {
        let raw = self.read_oversample(adc);
        // self.exponential_moving_average(raw)
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
            return 0.5; // assume perfectly centered at boot
        }

        // map
        ((raw - self.min) as f32) / range
    }

    // /// just returns raw measurement
    // fn raw(&mut self, raw: u16) -> f32 {
    //     raw as f32
    // }
}