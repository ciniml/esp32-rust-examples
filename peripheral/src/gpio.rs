#![no_std]
#![feature(alloc)] 

extern crate alloc;

use idf;
use idf::AsResult;
use idf::IdfError;

use embedded_hal::digital::v2::*;
use embedded_hal::digital::v1_compat::OldOutputPin;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GpioMode {
    Disable,
    Input,
    Output,
    OutputOpenDrain,
    InputOutputOpenDrain,
    InputOutput,
}
impl Default for GpioMode {
    fn default() -> Self { GpioMode::Disable }
}

fn to_gpio_mode(mode: GpioMode) -> idf::gpio_mode_t {
    match mode {
        GpioMode::Disable => idf::gpio_mode_t_GPIO_MODE_DISABLE,
        GpioMode::Input => idf::gpio_mode_t_GPIO_MODE_INPUT,
        GpioMode::Output => idf::gpio_mode_t_GPIO_MODE_OUTPUT,
        GpioMode::OutputOpenDrain => idf::gpio_mode_t_GPIO_MODE_OUTPUT_OD,
        GpioMode::InputOutputOpenDrain => idf::gpio_mode_t_GPIO_MODE_INPUT_OUTPUT_OD,
        GpioMode::InputOutput => idf::gpio_mode_t_GPIO_MODE_INPUT_OUTPUT,
    }
}

#[derive(Copy, Clone, Debug)]
pub enum GpioPullUp {
    Disable,
    Enable,
}
impl Default for GpioPullUp {
    fn default() -> Self { GpioPullUp::Disable }
}

fn to_gpio_pullup(mode: GpioPullUp) -> idf::gpio_pullup_t {
    match mode {
        GpioPullUp::Disable => idf::gpio_pullup_t_GPIO_PULLUP_DISABLE,
        GpioPullUp::Enable => idf::gpio_pullup_t_GPIO_PULLUP_ENABLE,
    }
}

#[derive(Copy, Clone, Debug)]
pub enum GpioPullDown {
    Disable,
    Enable,
}
impl Default for GpioPullDown {
    fn default() -> Self { GpioPullDown::Disable }
}

fn to_gpio_pulldown(mode: GpioPullDown) -> idf::gpio_pulldown_t {
    match mode {
        GpioPullDown::Disable => idf::gpio_pulldown_t_GPIO_PULLDOWN_DISABLE,
        GpioPullDown::Enable => idf::gpio_pulldown_t_GPIO_PULLDOWN_ENABLE,
    }
}

#[derive(Copy, Clone, Debug)]
pub enum GpioPullMode {
    PullUpOnly,
    PullDownOnly,
    PullUpPullDown,
    Floating,
}
impl Default for GpioPullMode {
    fn default() -> Self { GpioPullMode::Floating }
}

fn to_gpio_pull_mode(mode: GpioPullMode) -> idf::gpio_pull_mode_t {
    match mode {
        GpioPullMode::PullUpOnly => idf::gpio_pull_mode_t_GPIO_PULLUP_ONLY,
        GpioPullMode::PullDownOnly => idf::gpio_pull_mode_t_GPIO_PULLDOWN_ONLY,
        GpioPullMode::PullUpPullDown => idf::gpio_pull_mode_t_GPIO_PULLUP_PULLDOWN,
        GpioPullMode::Floating => idf::gpio_pull_mode_t_GPIO_FLOATING,
    }
}

#[derive(Copy, Clone, Debug)]
pub enum GpioDriveCap {
    Cap0,
    Cap1,
    Cap2,
    Default,
    Cap3,
}
impl Default for GpioDriveCap {
    fn default() -> Self { GpioDriveCap::Default }
}

fn to_gpio_drive_cap(cap: GpioDriveCap) -> idf::gpio_drive_cap_t {
    match cap {
        GpioDriveCap::Cap0 => idf::gpio_drive_cap_t_GPIO_DRIVE_CAP_0,
        GpioDriveCap::Cap1 => idf::gpio_drive_cap_t_GPIO_DRIVE_CAP_1,
        GpioDriveCap::Cap2 => idf::gpio_drive_cap_t_GPIO_DRIVE_CAP_2,
        GpioDriveCap::Default => idf::gpio_drive_cap_t_GPIO_DRIVE_CAP_DEFAULT,
        GpioDriveCap::Cap3 => idf::gpio_drive_cap_t_GPIO_DRIVE_CAP_3,
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct GpioConfig {
    pub mode: GpioMode,
    pub pullup: GpioPullUp,
    pub pulldown: GpioPullDown,
}
impl GpioConfig {
    pub fn output() -> GpioConfig { GpioConfig {mode: GpioMode::Output, ..Default::default() } }
    pub fn input() -> GpioConfig { GpioConfig {mode: GpioMode::Input, ..Default::default() } }
    pub fn inputPullUp() -> GpioConfig { GpioConfig {mode: GpioMode::Input, pullup: GpioPullUp::Enable, ..Default::default() } }
}

#[derive(Copy, Clone, Debug)]
pub struct GpioPin {
    number: u32,
}

impl GpioPin {
    pub fn number(&self) -> u32 { self.number }
    pub fn normal(&self) -> NormalGpio { NormalGpio::new(self.number) }
}

pub const GpioPin0  : GpioPin = GpioPin { number:  0 };
pub const GpioPin1  : GpioPin = GpioPin { number:  1 };
pub const GpioPin2  : GpioPin = GpioPin { number:  2 };
pub const GpioPin3  : GpioPin = GpioPin { number:  3 };
pub const GpioPin4  : GpioPin = GpioPin { number:  4 };
pub const GpioPin5  : GpioPin = GpioPin { number:  5 };
pub const GpioPin6  : GpioPin = GpioPin { number:  6 };
pub const GpioPin7  : GpioPin = GpioPin { number:  7 };
pub const GpioPin8  : GpioPin = GpioPin { number:  8 };
pub const GpioPin9  : GpioPin = GpioPin { number:  9 };
pub const GpioPin10 : GpioPin = GpioPin { number: 10 };
pub const GpioPin11 : GpioPin = GpioPin { number: 11 };
pub const GpioPin12 : GpioPin = GpioPin { number: 12 };
pub const GpioPin13 : GpioPin = GpioPin { number: 13 };
pub const GpioPin14 : GpioPin = GpioPin { number: 14 };
pub const GpioPin15 : GpioPin = GpioPin { number: 15 };
pub const GpioPin16 : GpioPin = GpioPin { number: 16 };
pub const GpioPin17 : GpioPin = GpioPin { number: 17 };
pub const GpioPin18 : GpioPin = GpioPin { number: 18 };
pub const GpioPin19 : GpioPin = GpioPin { number: 19 };
pub const GpioPin20 : GpioPin = GpioPin { number: 20 };
pub const GpioPin21 : GpioPin = GpioPin { number: 21 };
pub const GpioPin22 : GpioPin = GpioPin { number: 22 };
pub const GpioPin23 : GpioPin = GpioPin { number: 23 };
pub const GpioPin24 : GpioPin = GpioPin { number: 24 };
pub const GpioPin25 : GpioPin = GpioPin { number: 25 };
pub const GpioPin26 : GpioPin = GpioPin { number: 26 };
pub const GpioPin27 : GpioPin = GpioPin { number: 27 };
pub const GpioPin28 : GpioPin = GpioPin { number: 28 };
pub const GpioPin29 : GpioPin = GpioPin { number: 29 };
pub const GpioPin30 : GpioPin = GpioPin { number: 30 };
pub const GpioPin31 : GpioPin = GpioPin { number: 31 };
pub const GpioPin32 : GpioPin = GpioPin { number: 32 };
pub const GpioPin33 : GpioPin = GpioPin { number: 33 };
pub const GpioPin34 : GpioPin = GpioPin { number: 34 };
pub const GpioPin35 : GpioPin = GpioPin { number: 35 };
pub const GpioPin36 : GpioPin = GpioPin { number: 36 };
pub const GpioPin37 : GpioPin = GpioPin { number: 37 };
pub const GpioPin38 : GpioPin = GpioPin { number: 38 };
pub const GpioPin39 : GpioPin = GpioPin { number: 39 };

pub struct NormalGpio {
    number: u32,
}

impl NormalGpio {
    pub fn new(number: u32) -> NormalGpio {
        NormalGpio{number: number,}
    }

    pub fn number(&self) -> u32 { self.number }

    pub fn configure(&mut self, config: GpioConfig) -> Result<(), IdfError> {
        let idf_config = idf::gpio_config_t {
            pin_bit_mask: (1 as u64) << self.number,
            mode: to_gpio_mode(config.mode),
            pull_up_en: to_gpio_pullup(config.pullup),
            pull_down_en: to_gpio_pulldown(config.pulldown),
            intr_type: idf::gpio_int_type_t_GPIO_INTR_DISABLE,
        };
        unsafe{ idf::gpio_config(&idf_config).as_result() }
    }
    pub fn reset(&mut self) -> Result<(), IdfError> {
        unsafe{ idf::gpio_reset_pin(self.number).as_result() }
    }
    pub fn set_level(&mut self, level_high: bool) -> Result<(), IdfError> {
        let idf_level = if level_high { 1 as u32 } else { 0 as u32 };
        unsafe { idf::gpio_set_level(self.number, idf_level).as_result() }
    }
    pub fn get_level(&self) -> Result<bool, IdfError> {
        Ok( unsafe { idf::gpio_get_level(self.number) != 0 } )
    }
    pub fn to_v1(self) -> NormalGpioV1 {
        NormalGpioV1::new(self)
    }
}

pub type NormalGpioV1 = OldOutputPin<NormalGpio>;

impl OutputPin for NormalGpio {
    type Error = IdfError;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.set_level(false)
    }
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.set_level(true)
    }
}

impl InputPin for NormalGpio {
    type Error = IdfError;

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.get_level()
    }
    fn is_low(&self) -> Result<bool, Self::Error> {
        self.get_level().map(|v| { !v })
    }
}
