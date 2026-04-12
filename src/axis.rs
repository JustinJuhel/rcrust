use embassy_stm32::adc::{Adc, AdcChannel, Instance};

/// # Axis
/// This struct is used to read values from pins.
pub struct Axis {}

impl Axis {
    /// # New
    pub fn new() -> Self {
        Self {}
    }

    /// # Read
    /// Reads the provided hardware pin and return its value as an `u16`.
    pub fn read<T: Instance, P: AdcChannel<T>>(adc: &mut Adc<'_, T>, pin: &mut P) -> u16 {
        adc.blocking_read(pin)
    }

    /// # Process
    /// Processes raw data:
    /// * oversample by reading several times the same pin, in order to smooth the signal and suppress hardware jitter,
    pub fn process<T: Instance, P: AdcChannel<T>>(
        &mut self,
        adc: &mut Adc<'_, T>,
        pin: &mut P,
    ) -> u16 {
        let raw = self.read_oversample(adc, pin);
        raw
    }

    /// # Read with Oversampling
    /// **not anymore**: This function firstly reads the pin once and doesn't take the value into account. This happens because of an ADC crosstalk problem.
    /// The STM32 has only one Analog-to-Digital converter which is shared between all the pins. This can cause the voltage level of one analog input channel to
    /// influence the reading of another.
    ///
    /// Then, this function reads 64 times the same pin and computes an average.
    fn read_oversample<T: Instance, P: AdcChannel<T>>(
        &mut self,
        adc: &mut Adc<'_, T>,
        pin: &mut P,
    ) -> u16 {
        let basis = 6;
        let sample = 1 << basis;
        let mut summed_raw: u32 = 0;
        let mut cnt = 0;
        while cnt < sample {
            summed_raw += Self::read(adc, pin) as u32;
            cnt += 1;
        }
        // bit shift by 6 is much faster than division by 64
        (summed_raw >> basis) as u16
    }
}
