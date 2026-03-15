//! This file contains the hardware context of the system: GPIOs, ADC drivers, button driver, and so on.
//! These states are use in all system modes.

use esp_hal::{
    Blocking,
    analog::adc::{Adc, AdcConfig, Attenuation},
    gpio::{Input, InputConfig, Pull},
    peripherals::{ADC1, GPIO32, GPIO33, GPIO34, GPIO35},
};

use crate::{
    axis::{axis::AutoCalibAxis, raw_axes::RawAxes},
    system_state::{FlyingMode, SystemState},
};

pub struct HardwareContext {
    throttle_axis: AutoCalibAxis<GPIO32<'static>>,
    yaw_axis: AutoCalibAxis<GPIO33<'static>>,
    pitch_axis: AutoCalibAxis<GPIO34<'static>>,
    roll_axis: AutoCalibAxis<GPIO35<'static>>,
    adc1: Adc<'static, ADC1<'static>, Blocking>,
    switch_calib: Input<'static>,
    switch_fly_mode: Input<'static>,
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

        let pin_throttle = adc1_config.enable_pin(io_throttle, attenuation);
        let pin_yaw = adc1_config.enable_pin(io_yaw, attenuation);
        let pin_pitch = adc1_config.enable_pin(io_pitch, attenuation);
        let pin_roll = adc1_config.enable_pin(io_roll, attenuation);

        let window: f32 = 30.0;
        // PT3 Filter parameters
        let cutoff_hz: f32 = 15.0; // Hz
        let sample_rate_hz: f32 = 100.0; // Hz

        Self {
            throttle_axis: AutoCalibAxis::new(pin_throttle, window, cutoff_hz, sample_rate_hz),
            yaw_axis: AutoCalibAxis::new(pin_yaw, window, cutoff_hz, sample_rate_hz),
            pitch_axis: AutoCalibAxis::new(pin_pitch, window, cutoff_hz, sample_rate_hz),
            roll_axis: AutoCalibAxis::new(pin_roll, window, cutoff_hz, sample_rate_hz),
            adc1: Adc::new(peripherals.ADC1, adc1_config),
            switch_calib: Input::new(io_calib, InputConfig::default().with_pull(Pull::Down)),
            switch_fly_mode: Input::new(io_fly_mode, InputConfig::default().with_pull(Pull::Down)),
        }
    }

    /// TODO: doc
    pub fn axes(&mut self, flying_mode: FlyingMode) -> RawAxes {
        RawAxes::new(
            self.throttle_axis.process(&mut self.adc1),
            self.yaw_axis.process(&mut self.adc1),
            self.pitch_axis.process(&mut self.adc1),
            self.roll_axis.process(&mut self.adc1),
        )
    }

    /// TODO: doc
    pub fn update_system_state(&mut self, system_state: &mut SystemState) -> SystemState {
        match system_state {
            // update ground mode
            SystemState::StandBy | SystemState::Calibration => {
                *system_state = if self.switch_calib.is_high() {
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
