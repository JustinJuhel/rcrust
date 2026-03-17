//! This file contains the hardware context of the system: GPIOs, ADC drivers, button driver, and so on.
//! These states are use in all system modes.

use core::any::Any;

use esp_hal::{
    Blocking,
    analog::adc::{Adc, AdcChannel, AdcConfig, AdcPin, Attenuation},
    gpio::{AnyPin, Input, InputConfig, Pull},
    peripherals::{ADC1, GPIO32, GPIO33, GPIO34, GPIO35},
};

use crate::{
    axis::{axis::AutoCalibAxis, calibration::RcCalibration, raw_axes::RawAxes},
    system_state::{FlyingMode, SystemState},
};

pub struct HardwareContext {
    // throttle_axis: AutoCalibAxis<GPIO32<'static>>,
    // yaw_axis: AutoCalibAxis<GPIO33<'static>>,
    // pitch_axis: AutoCalibAxis<GPIO34<'static>>,
    // roll_axis: AutoCalibAxis<GPIO35<'static>>,
    pin_throttle: AdcPin<GPIO32<'static>, ADC1<'static>>,
    pin_yaw: AdcPin<GPIO33<'static>, ADC1<'static>>,
    pin_pitch: AdcPin<GPIO34<'static>, ADC1<'static>>,
    pin_roll: AdcPin<GPIO35<'static>, ADC1<'static>>,
    raw_axes: RawAxes,
    adc1: Adc<'static, ADC1<'static>, Blocking>,
    switch_calib: Input<'static>,
    switch_fly_mode: Input<'static>,
    pub calibration: Option<RcCalibration>,
}

impl HardwareContext {
    /// Hardware context initialisation. Gives access to the four axes and some other hardware interfaces.
    pub fn init() -> Self {
        let peripherals = esp_hal::init(esp_hal::Config::default());

        let io_throttle = peripherals.GPIO32;
        let io_yaw = peripherals.GPIO33;
        let io_pitch = peripherals.GPIO34;
        let io_roll = peripherals.GPIO35;
        let io_calib = peripherals.GPIO23;
        let io_fly_mode = peripherals.GPIO22;

        let mut adc1_config = AdcConfig::new();
        let attenuation = Attenuation::_11dB;

        let mut pin_throttle = adc1_config.enable_pin(io_throttle, attenuation);
        let mut pin_yaw = adc1_config.enable_pin(io_yaw, attenuation);
        let mut pin_pitch = adc1_config.enable_pin(io_pitch, attenuation);
        let mut pin_roll = adc1_config.enable_pin(io_roll, attenuation);

        let window: f32 = 30.0;
        // PT3 Filter parameters
        let cutoff_hz: f32 = 15.0; // Hz
        let sample_rate_hz: f32 = 100.0; // Hz

        let mut adc1 = Adc::new(peripherals.ADC1, adc1_config);

        let raw_axes = RawAxes::new(
            Self::read_pin(&mut pin_throttle, &mut adc1),
            Self::read_pin(&mut pin_yaw, &mut adc1),
            Self::read_pin(&mut pin_pitch, &mut adc1),
            Self::read_pin(&mut pin_roll, &mut adc1),
        );

        Self {
            // throttle_axis: AutoCalibAxis::new(pin_throttle, window, cutoff_hz, sample_rate_hz),
            // yaw_axis: AutoCalibAxis::new(pin_yaw, window, cutoff_hz, sample_rate_hz),
            // pitch_axis: AutoCalibAxis::new(pin_pitch, window, cutoff_hz, sample_rate_hz),
            // roll_axis: AutoCalibAxis::new(pin_roll, window, cutoff_hz, sample_rate_hz),
            pin_throttle,
            pin_yaw,
            pin_pitch,
            pin_roll,
            raw_axes,
            adc1,
            switch_calib: Input::new(io_calib, InputConfig::default().with_pull(Pull::Down)),
            switch_fly_mode: Input::new(io_fly_mode, InputConfig::default().with_pull(Pull::Down)),
            // TODO: try to get calibration here and replace the None
            calibration: None,
        }
    }

    /// TODO: doc
    fn read_pin(
        pin: &mut AdcPin<impl AdcChannel, ADC1<'static>>,
        adc: &mut Adc<'static, ADC1<'static>, Blocking>,
    ) -> u16 {
        match nb::block!(adc.read_oneshot(pin)) {
            Ok(value) => value as u16,
            Err(_) => 2048 as u16,
        }
    }

    /// # Raw Axes
    /// Reads the joystick pins, updates the `raw_axes` attribute and returns a `RawAxes` object.
    pub fn raw_axes(&mut self) -> RawAxes {
        self.raw_axes = RawAxes::new(
            Self::read_pin(&mut self.pin_throttle, &mut self.adc1),
            Self::read_pin(&mut self.pin_yaw, &mut self.adc1),
            Self::read_pin(&mut self.pin_pitch, &mut self.adc1),
            Self::read_pin(&mut self.pin_roll, &mut self.adc1),
        );
        self.raw_axes
    }

    /// TODO: doc
    pub fn raw_throttle(&mut self) -> u16 {
        Self::read_pin(&mut self.pin_throttle, &mut self.adc1)
    }

    // /// TODO: doc
    // pub fn raw_yaw(&mut self) -> u16 {
    //     Self::read_pin(&mut self.pin_yaw, &mut self.adc1)
    // }

    // /// TODO: doc
    // pub fn raw_pitch(&mut self) -> u16 {
    //     Self::read_pin(&mut self.pin_pitch, &mut self.adc1)
    // }

    // /// TODO: doc
    // pub fn raw_roll(&mut self) -> u16 {
    //     Self::read_pin(&mut self.pin_roll, &mut self.adc1)
    // }

    // /// TODO: doc
    // pub fn raw_axes(&mut self) -> RawAxes {
    //     // temporary implementation
    //     // we would need to use a specific algorithm configuration to feed the most natural jitter-free signal
    //     self.axes(FlyingMode::Angle)
    // }

    /// TODO: doc
    pub fn update_system_state(&mut self, system_state: &mut SystemState) -> SystemState {
        match system_state {
            // update ground mode
            SystemState::StandBy | SystemState::Calibration => {
                *system_state = if self.switch_calib.is_high() {
                    // reset calibration when switching to calibration mode
                    self.calibration = None;
                    SystemState::Calibration
                } else {
                    SystemState::StandBy
                };
            }
            // update flying mode
            SystemState::Flying(mode) => {
                *mode = if self.switch_fly_mode.is_high() {
                    FlyingMode::Acro
                } else {
                    FlyingMode::Angle
                };
            }
        };
        *system_state
    }
}
