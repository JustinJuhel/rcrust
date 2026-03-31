use defmt_rtt as _;
use embassy_stm32::adc::Adc;
use embassy_stm32::peripherals::{ADC1, PA1, PA2, PB0, PB1};
use embassy_stm32::usb::Driver;
use embassy_stm32::{bind_interrupts, peripherals};
use embassy_usb::UsbDevice;
use panic_probe as _;

use crate::signal::axes::Axes;

bind_interrupts!(pub struct Irqs {
    OTG_FS => embassy_stm32::usb::InterruptHandler<peripherals::USB_OTG_FS>;
});

#[embassy_executor::task]
pub async fn usb_task(mut device: UsbDevice<'static, Driver<'static, peripherals::USB_OTG_FS>>) {
    device.run().await;
}

pub struct HardwareContext {
    adc: Adc<'static, ADC1>,
    throttle_pin: PA1,
    yaw_pin: PA2,
    pitch_pin: PB0,
    roll_pin: PB1,
}

impl HardwareContext {
    pub fn new(
        adc: Adc<'static, ADC1>,
        throttle_pin: PA1,
        yaw_pin: PA2,
        pitch_pin: PB0,
        roll_pin: PB1,
    ) -> Self {
        Self {
            adc,
            throttle_pin,
            yaw_pin,
            pitch_pin,
            roll_pin,
        }
    }

    /// # Read
    /// Reads the provided hardware pin and return its value as an `Axes`.
    pub fn read(&mut self) -> Axes {
        Axes::new(
            self.adc.blocking_read(&mut self.throttle_pin),
            self.adc.blocking_read(&mut self.yaw_pin),
            self.adc.blocking_read(&mut self.pitch_pin),
            self.adc.blocking_read(&mut self.roll_pin),
        )
    }

    /// # Read with Oversampling
    /// This function firstly reads the pin once and doesn't take the value into account. This happens because of an ADC crosstalk problem.
    /// The STM32 has only one Analog-to-Digital converter which is shared between all the pins. This can cause the voltage level of one analog input channel to
    /// influence the reading of another.
    ///
    /// Then, this function reads 64 times the same pin and computes an average.
    pub fn read_oversample(&mut self) -> Axes {
        let _trash = self.read();

        let basis = 6;
        let sample = 1 << basis;
        let mut summed_throttle: u32 = 0;
        let mut summed_yaw: u32 = 0;
        let mut summed_pitch: u32 = 0;
        let mut summed_roll: u32 = 0;
        let mut cnt = 0;
        while cnt < sample {
            let new_axes = self.read();
            summed_throttle += new_axes.throttle() as u32;
            summed_yaw += new_axes.yaw() as u32;
            summed_pitch += new_axes.pitch() as u32;
            summed_roll += new_axes.roll() as u32;

            cnt += 1;
        }
        // bit shift by 6 is much faster than division by 64
        // summed_raw >> 6
        Axes::new(
            (summed_throttle >> basis) as u16,
            (summed_yaw >> basis) as u16,
            (summed_pitch >> basis) as u16,
            (summed_roll >> basis) as u16,
        )
    }
}
