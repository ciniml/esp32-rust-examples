#![no_std]
#![feature(alloc)] 

use core::ptr;
use core::mem;
use core::ops::{Deref, DerefMut};
use core::cell::{UnsafeCell};
use core::convert::Into;

extern crate alloc;
use alloc::boxed::Box;

use idf;
use idf::AsResult;
use cty::*;
use idf::IdfError;

use freertos_rs::*;
use embedded_hal::blocking::spi::*;
use embedded_hal::spi::FullDuplex;
use embedded_hal::blocking::spi::transfer::Default as TransferDefault;

use nb;

use crate::gpio::*;

pub struct SpiBus {
    host_device: idf::spi_host_device_t,
    config: idf::spi_bus_config_t,
    dma_channel: i32,
    lock: Semaphore,
}

#[derive(Copy, Clone, Debug)]
pub enum SpiHostDevice {
    Spi1,
    Spi2,
    Spi3,
}

impl Into<idf::spi_host_device_t> for SpiHostDevice {
    fn into(self) -> idf::spi_host_device_t {
        match self {
            SpiHostDevice::Spi1 => idf::spi_host_device_t_SPI1_HOST,
            SpiHostDevice::Spi2 => idf::spi_host_device_t_SPI2_HOST,
            SpiHostDevice::Spi3 => idf::spi_host_device_t_SPI3_HOST,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum SpiError {
    Generic,
    IdfError(IdfError),
    FreeRtosError(FreeRtosError),
}

impl From<IdfError> for SpiError {
    fn from(err: IdfError) -> SpiError {
        SpiError::IdfError(err)
    }
}
impl From<FreeRtosError> for SpiError {
    fn from(err: FreeRtosError) -> SpiError {
        SpiError::FreeRtosError(err)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SpiBusConfig {
    pub mosi_pin: GpioPin,
    pub miso_pin: GpioPin,
    pub sclk_pin: GpioPin,
    pub quadwp_pin: Option<GpioPin>,
    pub quadhd_pin: Option<GpioPin>,
    pub max_transfer_size: u32,
}

impl Into<idf::spi_bus_config_t> for SpiBusConfig {
    fn into(self) -> idf::spi_bus_config_t {
        idf::spi_bus_config_t {
            mosi_io_num: self.mosi_pin.number() as i32,
            miso_io_num: self.miso_pin.number() as i32,
            sclk_io_num: self.sclk_pin.number() as i32,
            quadwp_io_num: self.quadwp_pin.map_or(-1, |pin| { pin.number() as i32 }),
            quadhd_io_num: self.quadhd_pin.map_or(-1, |pin| { pin.number() as i32 }),
            max_transfer_sz: self.max_transfer_size as i32,
            flags: 0,
            intr_flags: 0,
        }
    }
}

#[derive(Copy, Clone)]
pub enum SpiMode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
}

impl Default for SpiMode {
    fn default() -> Self { SpiMode::Mode0 }
}
impl Into<u8> for SpiMode {
    fn into(self) -> u8 {
        match self {
            SpiMode::Mode0 => 0,
            SpiMode::Mode1 => 1,
            SpiMode::Mode2 => 2,
            SpiMode::Mode3 => 3,
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct SpiDeviceInterfaceConfig {
    pub command_bits: u8,
    pub address_bits: u8,
    pub dummy_bits: u8,
    pub mode: SpiMode,
    pub duty_cycle_pos: u8,
    pub cs_ena_pretrans: u8,
    pub cs_ena_posttrans: u8,
    pub clock_speed_hz: i32,
    pub input_delay_ns: i32,
    pub cs_pin: Option<GpioPin>,
}

impl Into<idf::spi_device_interface_config_t> for SpiDeviceInterfaceConfig {
    fn into(self) -> idf::spi_device_interface_config_t {
        idf::spi_device_interface_config_t {
            command_bits: self.command_bits,
            address_bits: self.address_bits,
            dummy_bits:   self.dummy_bits,
            mode:         self.mode.into(),
            duty_cycle_pos: self.duty_cycle_pos,
            cs_ena_pretrans:  self.cs_ena_pretrans,
            cs_ena_posttrans: self.cs_ena_posttrans,
            clock_speed_hz: self.clock_speed_hz,
            input_delay_ns: self.input_delay_ns,
            spics_io_num: self.cs_pin.map_or(-1, |pin| { pin.number() as i32 }),
            flags: 0,
            queue_size: 8,
            pre_cb: None,
            post_cb: None,
        }
    }
}

impl SpiBus {
    pub fn new(host_device: SpiHostDevice, config: SpiBusConfig, dma_channel: i32) -> Result<SpiBus, SpiError> {
        unsafe {
            let host_device = host_device as idf::spi_host_device_t;
            let config = config.into();
            idf::spi_bus_initialize(host_device, &config, dma_channel).as_result()?;
            match Semaphore::new_binary() {
                Ok(lock) => Ok(SpiBus{host_device: host_device, config: config, dma_channel: dma_channel, lock: lock}),
                Err(_) => Err(SpiError::Generic)
            }
        }
    }

    pub fn add_device<TTransactionContext, FPre, FPost>(&mut self, config: SpiDeviceInterfaceConfig, pre_callback: FPre, post_callback: FPost ) -> Result<SpiDeviceBusLock<TTransactionContext>, SpiError> 
        where FPre : FnMut(&TTransactionContext) + 'static, FPost : FnMut(&TTransactionContext) + 'static {
        let mut handle: idf::spi_device_handle_t = ptr::null_mut();
        //let guard = self.lock.lock(Duration::infinite()).unwrap();
        unsafe {
            let mut idf_config : idf::spi_device_interface_config_t = config.into();
            idf_config.pre_cb  = Some(SpiDevice::<TTransactionContext>::pre_callback_handler);
            idf_config.post_cb = Some(SpiDevice::<TTransactionContext>::post_callback_handler);
            let result = idf::spi_bus_add_device(self.host_device, &idf_config, &mut handle).as_result()?;
            let device = SpiDevice::new(handle, config.into(), pre_callback, post_callback)?;
            Ok(device)
        }
    }
}

unsafe impl Sync for SpiBus {}
impl Drop for SpiBus {
    fn drop(&mut self) {
        unsafe {
            idf::spi_bus_free(self.host_device);
        }
    }
}


pub struct SpiTransaction<'a, T> {
    flags: u32,
    cmd: u16,
    addr: u64,
    length: usize,
    rxlength: Option<usize>,
    tx_buffer: Option<&'a [u8]>,
    rx_buffer: Option<&'a mut [u8]>,
    user: T,
}

impl<'a, T> SpiTransaction<'a, T> {
    pub fn new_write(tx_buffer: &'a [u8], user: T) -> Self {
        Self {
            flags: 0,
            cmd: 0,
            addr: 0,
            length: tx_buffer.len()*8 as usize,
            rxlength: None,
            tx_buffer: Some(tx_buffer),
            rx_buffer: None,
            user: user,
        }
    }
    pub fn new_read(rx_buffer: &'a mut [u8], user: T) -> Self {
        Self {
            flags: 0,
            cmd: 0,
            addr: 0,
            length: rx_buffer.len()*8 as usize,
            rxlength: Some(0),
            tx_buffer: None,
            rx_buffer: Some(rx_buffer),
            user: user,
        }
    }
    pub fn new_both(tx_buffer: &'a [u8], rx_buffer: &'a mut [u8], user: T) -> Self {
        Self {
            flags: 0,
            cmd: 0,
            addr: 0,
            length: tx_buffer.len()*8 as usize,
            rxlength: Some(rx_buffer.len()*8 as usize),
            tx_buffer: Some(tx_buffer),
            rx_buffer: Some(rx_buffer),
            user: user,
        }
    }

}

pub struct SpiDevice<TTransactionContext> {
    handle: idf::spi_device_handle_t,
    config: idf::spi_device_interface_config_t,
    pre_callback: Box<FnMut(&TTransactionContext)>,
    post_callback:  Box<FnMut(&TTransactionContext)>,

    last_word: u8,
}

struct SpiTransactionContext<'a, TTransactionContext> {
    device: &'a mut SpiDevice<TTransactionContext>,
    context: TTransactionContext,
}

impl<TTransactionContext> SpiDevice<TTransactionContext> {
    fn new<FPre, FPost>(handle: idf::spi_device_handle_t, config: idf::spi_device_interface_config_t, pre_callback: FPre, post_callback: FPost) -> Result<SpiDeviceBusLock<TTransactionContext>, SpiError> 
        where FPre : FnMut(&TTransactionContext) + 'static, FPost : FnMut(&TTransactionContext) + 'static {
        Ok( SpiDeviceBusLock::new(SpiDevice{handle: handle, config: config, pre_callback: Box::new(pre_callback), post_callback: Box::new(post_callback), last_word: 0}))
    }

    unsafe extern "C" fn pre_callback_handler(idf_transaction: *mut idf::spi_transaction_t) {
        let context_ptr = (*idf_transaction).user as *mut SpiTransactionContext<TTransactionContext>;
        let device = &mut (*context_ptr).device;
        let user_context = &(*context_ptr).context;
        (*device.pre_callback)(user_context);
    } 

    unsafe extern "C" fn post_callback_handler(idf_transaction: *mut idf::spi_transaction_t) {
        let context_ptr = (*idf_transaction).user as *mut SpiTransactionContext<TTransactionContext>;
        let device = &mut (*context_ptr).device;
        let user_context = &(*context_ptr).context;
        (*device.post_callback)(user_context);
    } 

    pub fn transfer<'t>(&mut self, transaction: SpiTransaction<'t, TTransactionContext>) -> Result<(), SpiError> {
        let mut context = SpiTransactionContext::<TTransactionContext> {
            device: self,
            context: transaction.user,
        };
        
        unsafe {
            let mut idf_transaction = idf::spi_transaction_t::default();
            idf_transaction.length   = transaction.length as u32;
            idf_transaction.rxlength = transaction.rxlength.map_or(0, |v| v as u32);
            idf_transaction.cmd      = transaction.cmd;
            idf_transaction.addr     = transaction.addr;

            let mut tx_buffer_ptr = idf_transaction.__bindgen_anon_1.tx_buffer.as_mut();
            *tx_buffer_ptr = transaction.tx_buffer.map_or(ptr::null(), |tx_buffer| tx_buffer.as_ptr() as *const c_void);
            
            let mut rx_buffer_ptr = idf_transaction.__bindgen_anon_2.rx_buffer.as_mut();
            *rx_buffer_ptr = transaction.rx_buffer.map_or(ptr::null_mut(), |rx_buffer| rx_buffer.as_ptr() as *mut c_void);
        
            idf_transaction.user = (&mut context as *mut SpiTransactionContext<TTransactionContext>) as *mut c_void;
            idf::spi_device_polling_transmit(self.handle, &mut idf_transaction).as_result()?;
            Ok(())
        }
    }
}

pub struct SpiDeviceBusLock<TTransactionContext> {
    device: UnsafeCell<SpiDevice<TTransactionContext>>,
}
unsafe impl<TTransactionContext> Sync for SpiDeviceBusLock<TTransactionContext> {}
impl<TTransactionContext> SpiDeviceBusLock<TTransactionContext> {
    fn new(device: SpiDevice<TTransactionContext>) -> Self {
        SpiDeviceBusLock {
            device: UnsafeCell::new(device),
        }
    }
    pub fn lock<'b>(&'b self) -> Result<SpiBusGuard<'b, TTransactionContext>, SpiError> {
        unsafe {
            idf::spi_device_acquire_bus((*self.device.get()).handle, idf::portMAX_DELAY).as_result()?;
            Ok(SpiBusGuard::<'b, TTransactionContext> { device: &self.device })
        }
    }
}

pub struct SpiBusGuard<'a, TTransactionContext> {
    device: &'a UnsafeCell<SpiDevice<TTransactionContext>>,
}

impl<'a, TTransactionContext> Drop for SpiBusGuard<'a, TTransactionContext> {
    fn drop(&mut self) {
        unsafe {
            idf::spi_device_release_bus((*self.device.get()).handle)
        }
    }
}
impl<'a, TTransactionContext> Deref for SpiBusGuard<'a, TTransactionContext> {
    type Target = SpiDevice<TTransactionContext>;

    fn deref<'b>(&'b self) -> &'b Self::Target {
        unsafe{ & *self.device.get() }
    }
}
impl<'a, TTransactionContext> DerefMut for SpiBusGuard<'a, TTransactionContext> {
    fn deref_mut<'b>(&'b mut self) -> &'b mut Self::Target {
        unsafe{ &mut *self.device.get() }
    }
}

impl Write<u8> for SpiDevice<()> 
{
    type Error = SpiError;
    fn write(&mut self, words: &[u8]) -> Result<(), SpiError> {
        let transaction = SpiTransaction::<()>::new_write(words, ());
        self.transfer(transaction)
    }
}

impl Write<u8> for SpiDeviceBusLock<()>
{
    type Error = SpiError;
    fn write(&mut self, words: &[u8]) -> Result<(), SpiError> {
        let transaction = SpiTransaction::<()>::new_write(words, ());
        let mut device = self.lock()?;
        device.transfer(transaction);
        Ok(())
    }
}

impl TransferDefault<u8> for SpiDeviceBusLock<()> {}
impl FullDuplex<u8> for SpiDeviceBusLock<()>
{
    type Error = SpiError;

    fn read(&mut self) -> nb::Result<u8, SpiError> {
        Ok(self.lock().unwrap().last_word)
    }
    fn send(&mut self, word: u8) -> nb::Result<(), SpiError> {
        let tx : [u8;1] = [word];
        let mut rx : [u8;1] = [0];
        let transaction = SpiTransaction::<()>::new_both(&tx, &mut rx, ());
        let mut device = self.lock().unwrap();
        match device.transfer(transaction) {
            Ok(_) => {
                device.last_word = rx[0];
                Ok(())
            },
            Err(err) => Err(nb::Error::Other(err)),
        }
    }
}
