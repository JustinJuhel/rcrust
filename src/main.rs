#![no_std]
#![no_main]

use core::fmt::Write;

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_stm32::time::Hertz;
use embassy_stm32::usb::Driver;
use embassy_stm32::{Config, bind_interrupts, peripherals};
use embassy_time::{Duration, Ticker};
use embassy_usb::UsbDevice;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use panic_probe as _;
use read_gpio::axis::Axis;
use static_cell::StaticCell;

const INTERVAL_US: u64 = 1000;

bind_interrupts!(struct Irqs {
    OTG_FS => embassy_stm32::usb::InterruptHandler<peripherals::USB_OTG_FS>;
});

#[embassy_executor::task]
async fn usb_task(mut device: UsbDevice<'static, Driver<'static, peripherals::USB_OTG_FS>>) {
    device.run().await;
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Configure clocks: HSE 25 MHz (typical BlackPill) -> PLL -> 84 MHz sys, 48 MHz USB
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(25_000_000),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV25,  // 25 MHz / 25 = 1 MHz
            mul: PllMul::MUL336,       // 1 MHz * 336 = 336 MHz VCO
            divp: Some(PllPDiv::DIV4), // 336 / 4 = 84 MHz system clock
            divq: Some(PllQDiv::DIV7), // 336 / 7 = 48 MHz USB clock
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 42 MHz (max for APB1)
        config.rcc.apb2_pre = APBPrescaler::DIV1; // 84 MHz
    }
    let p = embassy_stm32::init(config);

    // USB CDC setup
    static EP_OUT_BUFFER: StaticCell<[u8; 256]> = StaticCell::new();
    let ep_out_buffer = EP_OUT_BUFFER.init([0u8; 256]);

    let driver = Driver::new_fs(
        p.USB_OTG_FS,
        Irqs,
        p.PA12,
        p.PA11,
        ep_out_buffer,
        embassy_stm32::usb::Config::default(),
    );

    let mut usb_config = embassy_usb::Config::new(0xc0de, 0xcafe);
    usb_config.manufacturer = Some("RCrust");
    usb_config.product = Some("Joystick Controller");
    usb_config.serial_number = Some("001");
    usb_config.max_power = 100;
    usb_config.max_packet_size_0 = 64;

    static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static MSOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

    let mut builder = embassy_usb::Builder::new(
        driver,
        usb_config,
        CONFIG_DESCRIPTOR.init([0; 256]),
        BOS_DESCRIPTOR.init([0; 256]),
        MSOS_DESCRIPTOR.init([0; 256]),
        CONTROL_BUF.init([0; 64]),
    );

    static CDC_STATE: StaticCell<State> = StaticCell::new();
    let cdc_state = CDC_STATE.init(State::new());
    let mut cdc = CdcAcmClass::new(&mut builder, cdc_state, 64);

    let usb_device = builder.build();
    spawner.must_spawn(usb_task(usb_device));

    // ADC setup
    let mut adc = Adc::new(p.ADC1);
    adc.set_sample_time(SampleTime::CYCLES480);
    adc.set_resolution(embassy_stm32::adc::Resolution::BITS12);

    let mut pin_throttle = p.PA1;
    let mut pin_yaw = p.PA2;
    let mut pin_pitch = p.PB0;
    let mut pin_roll = p.PB1;

    let window: f32 = 30.0;

    let mut throttle_axis = Axis::new(window);
    let mut yaw_axis = Axis::new(window);
    let mut pitch_axis = Axis::new(window);
    let mut roll_axis = Axis::new(window);

    let mut ticker = Ticker::every(Duration::from_micros(INTERVAL_US));

    // Wait for USB host to connect
    cdc.wait_connection().await;

    let mut buf = [0u8; 64];

    loop {
        ticker.next().await;

        let throttle = throttle_axis.process(&mut adc, &mut pin_throttle);
        let yaw = yaw_axis.process(&mut adc, &mut pin_yaw);
        let pitch = pitch_axis.process(&mut adc, &mut pin_pitch);
        let roll = roll_axis.process(&mut adc, &mut pin_roll);

        // Format into buffer and send over USB CDC
        let mut wrapper = BufWriter::new(&mut buf);
        let _ = write!(wrapper, "{}, {}, {}, {}\r\n", throttle, yaw, pitch, roll);
        let len = wrapper.pos;
        let _ = cdc.write_packet(&buf[..len]).await;
    }
}

/// Minimal wrapper to use core::fmt::Write into a byte buffer
struct BufWriter<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> BufWriter<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, pos: 0 }
    }
}

impl core::fmt::Write for BufWriter<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remaining = self.buf.len() - self.pos;
        let len = bytes.len().min(remaining);
        self.buf[self.pos..self.pos + len].copy_from_slice(&bytes[..len]);
        self.pos += len;
        Ok(())
    }
}
