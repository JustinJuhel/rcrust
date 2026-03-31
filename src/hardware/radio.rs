use core::fmt::Write;
use defmt_rtt as _;
use embassy_stm32::peripherals::USB_OTG_FS;
use embassy_stm32::usb::Driver;
use embassy_usb::class::cdc_acm::CdcAcmClass;
use heapless::String;
use panic_probe as _;

use crate::signal::axes::Axes;

pub struct Radio {
    pub cdc: CdcAcmClass<'static, Driver<'static, USB_OTG_FS>>,
}

impl Radio {
    pub fn new(cdc: CdcAcmClass<'static, Driver<'static, USB_OTG_FS>>) -> Self {
        Self { cdc }
    }

    /// Format into buffer and send over USB CDC
    pub async fn send_serial(&mut self, axes: Axes) {
        let mut buf: String<64> = String::new();
        let _ = write!(
            buf,
            "{}, {}, {}, {}\r\n",
            axes.throttle(),
            axes.yaw(),
            axes.pitch(),
            axes.roll()
        );
        let _ = self.cdc.write_packet(buf.as_bytes()).await;
    }
}
