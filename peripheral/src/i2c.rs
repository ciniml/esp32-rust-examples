#![no_std]
#![feature(alloc)] 

use core::convert::Into;
use core::marker::{Sync, PhantomData};
use core::mem::zeroed;
use cty::*;

use idf;
use idf::AsResult;
use idf::IdfError;

use freertos_rs::*;
use crate::gpio::*;


#[derive(Copy, Clone, Debug)]
pub enum I2cError {
    Generic,
    IdfError(IdfError),
    FreeRtosError(FreeRtosError),
}

impl From<IdfError> for I2cError {
    fn from(err: IdfError) -> I2cError {
        I2cError::IdfError(err)
    }
}
impl From<FreeRtosError> for I2cError {
    fn from(err: FreeRtosError) -> I2cError {
        I2cError::FreeRtosError(err)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum I2cPortNumber {
    Port0,
    Port1,
}

impl Into<idf::i2c_port_t> for I2cPortNumber {
    fn into(self) -> idf::i2c_port_t {
        match self {
            I2cPortNumber::Port0 => 0,
            I2cPortNumber::Port1 => 1,
        }
    }
}

pub struct I2cPortImpl {
    port_number: I2cPortNumber,
    config: I2cConfig,
}

unsafe impl Sync for I2cPort {}

#[derive(Copy, Clone, Debug)]
pub enum I2cMode {
    Slave,
    Master,
}

impl Into<idf::i2c_mode_t> for I2cMode {
    fn into(self) -> idf::i2c_mode_t {
        match self {
            I2cMode::Slave => 0,
            I2cMode::Master => 1,
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(i32)]
pub enum I2cAckType {
    Ack = 0,
    Nack = 1,
    LastNack = 2,
}
impl Into<idf::i2c_ack_type_t> for I2cAckType {
    fn into(self) -> idf::i2c_ack_type_t {
        self as idf::i2c_ack_type_t
    }
}


#[derive(Copy, Clone, Debug)]
pub struct I2cConfig {
    pub mode: I2cMode,
    pub sda_io: GpioPin,
    pub scl_io: GpioPin,
    pub sda_pullup_en: GpioPullUp,
    pub scl_pullup_en: GpioPullUp,
    pub clk_speed: u32,
    pub addr_10bit_en: bool,
    pub slave_addr: u16,
}

impl Into<idf::i2c_config_t> for I2cConfig {
    fn into(self) -> idf::i2c_config_t {
        unsafe {
            let mut config = core::mem::zeroed::<idf::i2c_config_t>();
            config.mode = self.mode as idf::i2c_mode_t;
            config.sda_io_num = self.sda_io.number() as idf::gpio_num_t;
            config.scl_io_num = self.scl_io.number() as idf::gpio_num_t;
            config.sda_pullup_en = self.sda_pullup_en as idf::gpio_pullup_t;
            config.scl_pullup_en = self.scl_pullup_en as idf::gpio_pullup_t;

            match self.mode {
                I2cMode::Master => {
                    config.__bindgen_anon_1.master.as_mut().clk_speed = self.clk_speed;
                },
                I2cMode::Slave => {
                    config.__bindgen_anon_1.slave.as_mut().addr_10bit_en = if self.addr_10bit_en {1} else {0};
                    config.__bindgen_anon_1.slave.as_mut().slave_addr = self.slave_addr;
                },
            };
            config
        }
    }
}

pub trait I2cDeviceAddress {
    fn fill_bytes(&self, bytes: &mut [u8]) -> usize;
}
impl I2cDeviceAddress for u8 {
    fn fill_bytes(&self, bytes: &mut [u8]) -> usize {
        bytes[0] = *self << 1;
        1
    }
}
impl I2cDeviceAddress for u16 {
    fn fill_bytes(&self, bytes: &mut [u8]) -> usize {
        bytes[0] = (*self >> 7) as u8;
        bytes[1] = (*self << 1) as u8;
        2
    }
}


// I2C Port with lock
pub struct I2cPort {
    mutex: freertos_rs::Mutex<I2cPortImpl>,
}

impl I2cPort {
    pub fn new_master(port_number: I2cPortNumber) -> Result<I2cPort, I2cError> {
        let port = I2cPortImpl::new_master(port_number)?;
        Ok(I2cPort {
            mutex: freertos_rs::Mutex::new(port)?,
        })
    }
    pub fn lock(&self, wait_ticks: freertos_rs::Duration) -> Result<freertos_rs::MutexGuard<I2cPortImpl, MutexNormal>, I2cError> {
        let guard = self.mutex.lock(wait_ticks)?;
        Ok(guard)
    }
}
impl I2cPortImpl {
    fn new_master(port_number: I2cPortNumber) -> Result<I2cPortImpl, I2cError> {
        unsafe {
            idf::i2c_driver_install(port_number as idf::i2c_port_t, I2cMode::Master as idf::i2c_mode_t, 0, 0, 0).as_result()?;
            Ok( I2cPortImpl { port_number: port_number, config: core::mem::zeroed() } )
        }
    }

    pub fn config(&mut self, i2c_config: I2cConfig) -> Result<(), I2cError> {
        let idf_config = i2c_config.into();
        unsafe {
            idf::i2c_param_config(self.port_number as idf::i2c_port_t, &idf_config).as_result()?;
            self.config = i2c_config;
            Ok(())
        }
    }

    pub fn cmd_begin<'a>(&mut self, cmd_link: I2cCommandLink<'a>, wait_ticks: freertos_rs::Duration) -> Result<(), I2cError> {
        unsafe {
            idf::i2c_master_cmd_begin(self.port_number as idf::i2c_port_t, cmd_link.handle, wait_ticks.to_ticks()).as_result()?;
        }
        Ok(())
    }

    fn wait_ticks_from_len(&self, len: usize) -> freertos_rs::Duration {
        let wait_ms = ((len as u32) + 8 + 2) * 1000 / self.config.clk_speed + 10;
        freertos_rs::Duration::ms(wait_ms)
    }
}
impl Drop for I2cPortImpl {
    fn drop(&mut self) {
        unsafe {
            idf::i2c_driver_delete(self.port_number as idf::i2c_port_t);
        }
    }
}

pub struct I2cCommandLink<'cmdlink> {
    handle: idf::i2c_cmd_handle_t,
    phantom: PhantomData<&'cmdlink u8>,
}
impl<'cmdlink> I2cCommandLink<'cmdlink> {
    pub fn new<'a>() -> I2cCommandLink<'a> {
        unsafe {
            I2cCommandLink {
                handle: idf::i2c_cmd_link_create(),
                phantom: PhantomData,
            }
        }
    }
    pub fn start(&mut self) -> Result<&mut Self, I2cError> {
        unsafe {
            idf::i2c_master_start(self.handle).as_result()?;
        }
        Ok(self)
    }
    pub fn stop(&mut self) -> Result<&mut Self, I2cError> {
        unsafe {
            idf::i2c_master_stop(self.handle).as_result()?;
        }
        Ok(self)
    }
    pub fn write_byte(&mut self, data: u8, ack_en: bool) -> Result<&mut Self, I2cError> {
        unsafe {
            idf::i2c_master_write_byte(self.handle, data, ack_en).as_result()?;
        }
        Ok(self)
    }
    pub fn write(&mut self, data: &'cmdlink [u8], ack_en: bool) -> Result<&mut Self, I2cError> {
        unsafe {
            idf::i2c_master_write(self.handle, data.as_ptr() as *mut u8, data.len() as u32, ack_en).as_result()?;
        }
        Ok(self)
    }
    pub fn read_byte(&mut self, data: &'cmdlink mut [u8], ack_type: I2cAckType) -> Result<&mut Self, I2cError> {
        unsafe {
            idf::i2c_master_read_byte(self.handle, data.as_mut_ptr(), ack_type.into()).as_result()?;
        }
        Ok(self)
    }
    pub fn read(&mut self, data: &'cmdlink mut [u8], ack_type: I2cAckType) -> Result<&mut Self, I2cError> {
        unsafe {
            idf::i2c_master_read(self.handle, data.as_mut_ptr(), data.len() as u32, ack_type.into()).as_result()?;
        }
        Ok(self)
    }
    pub fn write_register<DeviceAddress>(&mut self, device_address: DeviceAddress, register_address: &[u8], data: &'cmdlink [u8]) -> Result<&mut Self, I2cError>
        where DeviceAddress : I2cDeviceAddress 
    {
        self.start()?;
        let mut device_address_bytes: [u8; 2] = [0; 2];
        let device_address_length = device_address.fill_bytes(&mut device_address_bytes);
        // SLA+W
        for i in 0..device_address_length {
            self.write_byte(device_address_bytes[i], true)?;
        }
        for register_address_byte in register_address {
            self.write_byte(*register_address_byte, true)?;
        }
        self.write(data, true)?;
        self.stop()?;
        Ok(self)
    }
    pub fn read_register<DeviceAddress>(&mut self, device_address: DeviceAddress, register_address: &[u8], data: &'cmdlink mut [u8]) -> Result<&mut Self, I2cError>
        where DeviceAddress : I2cDeviceAddress 
    {
        self.start()?;
        let mut device_address_bytes: [u8; 2] = [0; 2];
        let device_address_length = device_address.fill_bytes(&mut device_address_bytes);
        // SLA+W
        for i in 0..device_address_length {
            self.write_byte(device_address_bytes[i], true)?;
        }
        for register_address_byte in register_address {
            self.write_byte(*register_address_byte, true)?;
        }
        self.start()?;  // Repeated start
        // SLA+R
        for i in 0..device_address_length {
            let r_bit: u8 = if  i == device_address_length - 1 { 1 } else { 0 };
            self.write_byte(device_address_bytes[i] | r_bit, true)?;
        }
        self.read(data, I2cAckType::LastNack)?;
        self.stop()?;
        Ok(self)
    }
}
impl<'cmdlink> Drop for I2cCommandLink<'cmdlink> {
    fn drop(&mut self) {
        unsafe {
            idf::i2c_cmd_link_delete(self.handle);
        }
    }
}

// Implement embedded-hal Blocking I2C API
use embedded_hal::blocking::i2c::*;

impl Read for I2cPortImpl {
    type Error = I2cError;

    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        let wait_ticks = self.wait_ticks_from_len(buffer.len());
        let mut command = I2cCommandLink::new();
        command.start()?;
        command.write_byte((address << 1) | 1, true)?;
        command.read(buffer, I2cAckType::LastNack)?;
        command.stop()?;
        self.cmd_begin(command, wait_ticks)?;
        Ok(())
    }
}

impl Write for I2cPortImpl {
    type Error = I2cError;

    fn write(&mut self, address: u8, buffer: &[u8]) -> Result<(), Self::Error> {
        let wait_ticks = self.wait_ticks_from_len(buffer.len());
        let mut command = I2cCommandLink::new();
        command.start()?;
        command.write_byte((address << 1) | 0, true)?;
        command.write(buffer, true)?;
        command.stop()?;
        self.cmd_begin(command, wait_ticks)?;
        Ok(())
    }
}


impl WriteIter for I2cPortImpl {
    type Error = I2cError;

    fn write<B>(&mut self, address: u8, bytes: B) -> Result<(), Self::Error>
        where B: IntoIterator<Item = u8>
    {
        let mut items_count: usize = 0;
        let mut command = I2cCommandLink::new();
        command.start()?;
        command.write_byte((address << 1) | 0, true)?;
        
        for byte in bytes {
            command.write_byte(byte, true)?;
            items_count += 1;
        }

        command.stop()?;
        self.cmd_begin(command, self.wait_ticks_from_len(items_count))?;
        Ok(())
    }
}


impl WriteRead for I2cPortImpl {
    type Error = I2cError;

    fn write_read(&mut self, address: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Self::Error> {
        let wait_ticks = self.wait_ticks_from_len(buffer.len() + bytes.len());
        let mut command = I2cCommandLink::new();
        command.start()?;
        command.write_byte((address << 1) | 0, true)?;
        command.write(bytes, true)?;
        command.start()?;
        command.write_byte((address << 1) | 1, true)?;
        command.read(buffer, I2cAckType::LastNack)?;
        command.stop()?;
        self.cmd_begin(command, wait_ticks)?;
        Ok(())
    }
}


impl WriteIterRead for I2cPortImpl {
    type Error = I2cError;

    fn write_iter_read<B>(&mut self, address: u8, bytes: B, buffer: &mut [u8]) -> Result<(), Self::Error>
        where B: IntoIterator<Item = u8>
    {
        let mut items_count = buffer.len();
        let mut command = I2cCommandLink::new();
        command.start()?;
        command.write_byte((address << 1) | 0, true)?;
        
        for byte in bytes {
            command.write_byte(byte, true)?;
            items_count += 1;
        }

        command.start()?;
        command.write_byte((address << 1) | 1, true)?;
        command.read(buffer, I2cAckType::LastNack)?;

        command.stop()?;
        self.cmd_begin(command, self.wait_ticks_from_len(items_count))?;
        Ok(())
    }
}

