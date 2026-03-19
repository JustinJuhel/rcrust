//! This file contains the hardware context of the system: GPIOs, ADC drivers, button driver, and so on.
//! These states are use in all system modes.

use core::any::Any;

use esp_hal::{
    Blocking,
    analog::adc::{Adc, AdcChannel, AdcConfig, AdcPin, Attenuation},
    gpio::{AnyPin, Input, InputConfig, Pull},
    peripherals::{ADC1, GPIO32, GPIO33, GPIO34, GPIO35},
};
use esp_println::println;

use crate::{
    axes::Axes, calibration::RcCalibration, filter::Filter, radio::Radio, system_mode::SystemMode,
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
    raw_axes: Axes,
    adc1: Adc<'static, ADC1<'static>, Blocking>,
    arm_switch: Input<'static>,
    flight_mode_switch: Input<'static>,
    // nagivation_button: Input<'static>,
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

        let raw_axes = Axes::new(
            Self::read_pin(&mut pin_throttle, &mut adc1),
            Self::read_pin(&mut pin_yaw, &mut adc1),
            Self::read_pin(&mut pin_pitch, &mut adc1),
            Self::read_pin(&mut pin_roll, &mut adc1),
        );

        /*
        // initialize rotary encoder
        let timer = dp.TIM2;
        let pins = (gpioa.pa0, gpioa.pa1); // Channels 1 and 2
        let encoder = RotaryEncoder::new(timer, pins);
        */

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
            arm_switch: Input::new(io_calib, InputConfig::default().with_pull(Pull::Down)),
            flight_mode_switch: Input::new(
                io_fly_mode,
                InputConfig::default().with_pull(Pull::Down),
            ),
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
    pub fn read_axes(&mut self) -> Axes {
        self.raw_axes = Axes::new(
            Self::read_pin(&mut self.pin_throttle, &mut self.adc1),
            Self::read_pin(&mut self.pin_yaw, &mut self.adc1),
            Self::read_pin(&mut self.pin_pitch, &mut self.adc1),
            Self::read_pin(&mut self.pin_roll, &mut self.adc1),
        );
        self.raw_axes
    }

    /// # Oversampled Raw Axes
    /// Reads the pins multiple times to average the value and overcome jitter.
    pub fn read_axes_oversample(&mut self, basis: i32) -> Axes {
        // Eventually, throw the first value away to avoid leftover charge in the microcontroller
        // let _trash = self.read(adc);

        let sample = 1 << basis; // 2^basis
        let mut cnt = 0;

        let mut summed_throttle: u32 = 0;
        let mut summed_yaw: u32 = 0;
        let mut summed_pitch: u32 = 0;
        let mut summed_roll: u32 = 0;

        while cnt < sample {
            let axes = self.read_axes();
            summed_throttle += axes.throttle() as u32;
            summed_yaw += axes.yaw() as u32;
            summed_pitch += axes.pitch() as u32;
            summed_roll += axes.roll() as u32;
            cnt += 1;
        }
        // bit shift is much faster than division by a power of 2
        Axes::new(
            (summed_throttle >> basis) as u16,
            (summed_yaw >> basis) as u16,
            (summed_pitch >> basis) as u16,
            (summed_roll >> basis) as u16,
        )
    }

    /// TODO: doc
    pub fn raw_throttle(&mut self) -> u16 {
        Self::read_pin(&mut self.pin_throttle, &mut self.adc1)
    }

    /// # Connection to Serial
    /// If the device is connected to a PC via serial, this function returns `true`. Else, it returns `false`.
    fn connected_serial(&mut self) -> bool {
        true
    }

    /// TODO: doc
    pub fn update_system_mode(&mut self, mode: &mut SystemMode) -> SystemMode {
        SystemMode::Serial
        // *mode = match *mode {
        //     any if self.connected_serial() => SystemMode::Serial,
        // };
        // return *mode;

        // // First, check if the remote is connected to a computer
        // if self.connected_serial() {
        //     *mode = SystemMode::Serial;
        //     return *mode;
        // }

        // // Else, the RC is autonomous. Handle a different logic
        // let armed = self.arm_switch.is_high();
        // match mode {
        //     // TODO: handle enter in Calibration mode like so:
        //     // SystemState::Disarm if calib_button_pushed => *system_state = SystemState::Calibration,
        //     SystemState::Arm if !armed => *mode = SystemState::Disarm,
        //     SystemState::Disarm if armed => *mode = SystemState::Arm,
        //     // update flying mode
        //     SystemState::Flying(mode) => {
        //         *mode = if self.flight_mode_switch.is_high() {
        //             FlyingMode::Acro
        //         } else {
        //             FlyingMode::Angle
        //         };
        //     }
        //     _ => {}
        // };
        // *mode
    }

    /// # Calibration mode
    /// In this version the user will move the joysticks at their extremities and the
    /// system calibrates the axes to keep the historical minimum and maximum.
    ///
    /// For now, no center because it needs to give feedback to the user.
    /// A system upgrade will include a screen to give calibration instructions.
    pub fn tick_calib(&mut self) {
        let calib_axes = self.read_axes();
        match self.calibration.as_mut() {
            Some(calibration) => calibration.update(calib_axes),
            None => self.calibration = Some(RcCalibration::new(calib_axes)),
        }
    }

    /// TODO: impl & doc
    fn get_calib_memory() -> Option<RcCalibration> {
        None
    }

    // useless ? in Disarm we do nothing.
    // /// # Stand-by mode
    // /// In this mode, the RC won't send signals to the drone. The user can get ready to fly.
    // ///
    // /// If the throttle axis comes all the way down **AND** there is a calibration,
    // /// the system goes to `Flying` mode and the drone reacts to joystick movements.
    // ///
    // /// In this mode, if there is no calibration, the drone won't fly. The user needs to calibrate the RC before flying.
    // pub fn tick_standby(&mut self, system_state: &mut SystemState) {
    //     if self.calibration.is_none() {
    //         if let Some(calib_memory) = Self::get_calib_memory() {
    //             self.calibration = Some(calib_memory.clone());
    //             if self.raw_throttle() <= calib_memory.throttle_dead_zone() {
    //                 // if throttle is down
    //                 let flying_mode = if self.flight_mode_switch.is_high() {
    //                     FlyingMode::Acro
    //                 } else {
    //                     FlyingMode::Angle
    //                 };
    //                 *system_state = SystemState::Flying(flying_mode);
    //             }
    //         } else {
    //             // do nothing
    //             println!("WARN: you need to calibrate the RC before flying!");
    //             return;
    //         }
    //     }
    // }

    // useless except if the logic becomes more complex.
    // /// TODO: use read_oversample
    // /// impl & doc
    // pub fn tick_serial(&mut self, &mut filter: Filter) {
    //     let axes = self.read_axes();
    //     let smooth = filter.smooth(axes);
    //     Radio::send_serial(smooth);
    // }
}
