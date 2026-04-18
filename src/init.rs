use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::mode::Blocking;
use embassy_stm32::peripherals::{ADC1, PA0, PA1, PB0, PB1, USB_OTG_FS};
use embassy_stm32::spi::{self, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::usb::Driver;
use embassy_stm32::{Config, bind_interrupts, peripherals};
use embassy_usb::UsbDevice;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use static_cell::StaticCell;

use display_interface_spi::SPIInterface;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_hal_bus::spi::ExclusiveDevice;
use mipidsi::models::ILI9341Rgb565;
use mipidsi::options::{Orientation, Rotation};

bind_interrupts!(struct Irqs {
    OTG_FS => embassy_stm32::usb::InterruptHandler<peripherals::USB_OTG_FS>;
});

#[embassy_executor::task]
async fn usb_task(mut device: UsbDevice<'static, Driver<'static, peripherals::USB_OTG_FS>>) {
    device.run().await;
}

/// # LCD
/// A high-level handle for the ILI9341 LCD Display.
///
/// This type represents a display configured with:
/// * **Interface:** SPI using a blocking driver.
/// * **Bus Management:** Exclusive access (the SPI bus is dedicated to this display).
/// * **Color Protocol:** RGB565 (16-bit color).
/// * **Controller:** ILI9341.
pub type Lcd = mipidsi::Display<
    SPIInterface<
        ExclusiveDevice<Spi<'static, Blocking>, Output<'static>, embedded_hal_bus::spi::NoDelay>,
        Output<'static>,
    >,
    ILI9341Rgb565,
    Output<'static>,
>;

/// # Initialize RC
/// This function is called at boot and does the following tasks:
/// * Clock configuration
/// * SPI display setup (ILI9341) — done first to avoid blocking after USB task is spawned
/// * ADC setup
/// * USB CDC-ACM setup — spawned last so enumeration starts with no pending blocking work
///
/// And returns the peripherals to be used in the main loop.
pub fn init_rc(
    spawner: Spawner,
) -> (
    Adc<'static, ADC1>,
    PA0,
    PA1,
    PB0,
    PB1,
    CdcAcmClass<'static, Driver<'static, USB_OTG_FS>>,
    Lcd,
    Input<'static>,
) {
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

    // SPI2 + ILI9341 display setup — done before USB so the blocking SPI
    // transfers don't starve the USB task of executor time.
    let mut spi_config = spi::Config::default();
    spi_config.frequency = Hertz(10_000_000);

    let spi = Spi::new_blocking_txonly(p.SPI2, p.PB10, p.PB15, spi_config);
    let cs = Output::new(p.PB12, Level::High, Speed::VeryHigh);
    let dc = Output::new(p.PB13, Level::Low, Speed::VeryHigh);
    let mut rst = Output::new(p.PB14, Level::Low, Speed::VeryHigh);

    cortex_m::asm::delay(84_000 * 10);
    rst.set_high();
    cortex_m::asm::delay(84_000 * 120);

    let spi_dev = ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let di = SPIInterface::new(spi_dev, dc);

    let mut display = mipidsi::Builder::new(ILI9341Rgb565, di)
        .reset_pin(rst)
        .orientation(Orientation::new().rotate(Rotation::Deg90))
        .init(&mut embassy_time::Delay)
        .unwrap();

    display.clear(Rgb565::BLACK).unwrap();

    // ADC setup
    let mut adc = Adc::new(p.ADC1);
    adc.set_sample_time(SampleTime::CYCLES56);
    adc.set_resolution(embassy_stm32::adc::Resolution::BITS12);

    // USB CDC setup — spawned last so the USB task starts with no blocking
    // work ahead of it in the executor.
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
    let cdc = CdcAcmClass::new(&mut builder, cdc_state, 64);

    let usb_device = builder.build();
    spawner.must_spawn(usb_task(usb_device));

    // ARM/DISARM switch
    let arm_switch = Input::new(p.PA5, Pull::Up);

    (adc, p.PA0, p.PA1, p.PB0, p.PB1, cdc, display, arm_switch)
}
