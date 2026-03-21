use embassy_stm32::adc::{Adc, AdcChannel, Instance};

/// # Axis
/// This struct is used to read values from pins and store data used by the EMA filter.
pub struct Axis {
    last_filtered: Option<f32>,
    alpha: f32,
}

impl Axis {
    /// # New
    /// `Axis` instanciation. The `window` argument affects the EMA filter's sensitivity. The higher `window`, the more "low-pass" the filter.
    pub fn new(window: f32) -> Self {
        Self {
            last_filtered: None,
            alpha: 2.0 / (window + 1.0),
        }
    }

    /// # Read
    /// Reads the provided hardware pin and return its value as an `u16`.
    pub fn read<T: Instance, P: AdcChannel<T>>(adc: &mut Adc<'_, T>, pin: &mut P) -> u16 {
        adc.blocking_read(pin)
    }

    /// # Process
    /// Processes raw data:
    /// - oversample by reading several times the same pin, in order to smooth the signal and suppress hardware jitter,
    /// - apply an exponential moving average filter to smooth the signal.
    pub fn process<T: Instance, P: AdcChannel<T>>(
        &mut self,
        adc: &mut Adc<'_, T>,
        pin: &mut P,
    ) -> u16 {
        let raw = self.read_oversample(adc, pin);
        self.exponential_moving_average(raw)
    }

    /// # Read with Oversampling
    /// This function firstly reads the pin once and doesn't take the value into account. This happens because of an ADC crosstalk problem.
    /// The STM32 has only one Analog-to-Digital converter which is shared between all the pins. This can cause the voltage level of one analog input channel to
    /// influence the reading of another.
    ///
    /// Then, this function reads 64 times the same pin and computes an average.
    fn read_oversample<T: Instance, P: AdcChannel<T>>(
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

    /// # EMA (exponential moving average) filter.
    /// Computationally cheap. Smoothes out sensor jitter.
    fn exponential_moving_average(&mut self, raw: u16) -> u16 {
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
