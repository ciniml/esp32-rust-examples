#![no_std]
#![feature(alloc)] 

use core::iter::Iterator;
use idf;
use idf::IdfError;

use freertos_rs::*;
use peripheral::*;
use embedded_hal::digital::v2::*;
use embedded_graphics;
use embedded_graphics::drawable::Pixel;
use embedded_graphics::pixelcolor::{Rgb565, PixelColor};

#[derive(Debug)]
pub enum LcdError {
    Generic,
    IdfError(IdfError),
    SpiError(SpiError),
}

impl From<IdfError> for LcdError {
    fn from(error: IdfError) -> LcdError {
        LcdError::IdfError(error)
    }
}
impl From<SpiError> for LcdError {
    fn from(error: SpiError) -> LcdError {
        LcdError::SpiError(error)
    }
}

pub struct Lcd {
    spi: SpiDeviceBusLock<bool>,
    pin_dc: NormalGpio,
    pin_rst:NormalGpio,
    pin_bl: NormalGpio,
    pin_cs: NormalGpio,
    line_buffer: [u8; 640],
}

const TFT_NOP:u8 = 0x00;
const TFT_SWRST:u8 = 0x01;

const TFT_CASET:u8 = 0x2A;
const TFT_PASET:u8 = 0x2B;
const TFT_RAMWR:u8 = 0x2C;

const TFT_RAMRD:u8 = 0x2E;
const TFT_IDXRD:u8 = 0xDD; // ILI9341 only, indexed control register read

const TFT_MADCTL:u8 = 0x36;
const TFT_MAD_MY:u8 = 0x80;
const TFT_MAD_MX:u8 = 0x40;
const TFT_MAD_MV:u8 = 0x20;
const TFT_MAD_ML:u8 = 0x10;
const TFT_MAD_BGR:u8 = 0x08;
const TFT_MAD_MH:u8 = 0x04;
const TFT_MAD_RGB:u8 = 0x00;

const TFT_INVOFF:u8 = 0x20;
const TFT_INVON:u8 = 0x21;

const ILI9341_NOP:u8 = 0x00;
const ILI9341_SWRESET:u8 = 0x01;
const ILI9341_RDDID:u8 = 0x04;
const ILI9341_RDDST:u8 = 0x09;

const ILI9341_SLPIN:u8 = 0x10;
const ILI9341_SLPOUT:u8 = 0x11;
const ILI9341_PTLON:u8 = 0x12;
const ILI9341_NORON:u8 = 0x13;

const ILI9341_RDMODE:u8 = 0x0A;
const ILI9341_RDMADCTL:u8 = 0x0B;
const ILI9341_RDPIXFMT:u8 = 0x0C;
const ILI9341_RDIMGFMT:u8 = 0x0A;
const ILI9341_RDSELFDIAG:u8 = 0x0F;

const ILI9341_INVOFF:u8 = 0x20;
const ILI9341_INVON:u8 = 0x21;
const ILI9341_GAMMASET:u8 = 0x26;
const ILI9341_DISPOFF:u8 = 0x28;
const ILI9341_DISPON:u8 = 0x29;

const ILI9341_CASET:u8 = 0x2A;
const ILI9341_PASET:u8 = 0x2B;
const ILI9341_RAMWR:u8 = 0x2C;
const ILI9341_RAMRD:u8 = 0x2E;

const ILI9341_PTLAR:u8 = 0x30;
const ILI9341_VSCRDEF:u8 = 0x33;
const ILI9341_MADCTL:u8 = 0x36;
const ILI9341_VSCRSADD:u8 = 0x37;
const ILI9341_PIXFMT:u8 = 0x3A;

const ILI9341_WRDISBV:u8 = 0x51;
const ILI9341_RDDISBV:u8 = 0x52;
const ILI9341_WRCTRLD:u8 = 0x53;

const ILI9341_FRMCTR1:u8 = 0xB1;
const ILI9341_FRMCTR2:u8 = 0xB2;
const ILI9341_FRMCTR3:u8 = 0xB3;
const ILI9341_INVCTR:u8 = 0xB4;
const ILI9341_DFUNCTR:u8 = 0xB6;

const ILI9341_PWCTR1:u8 = 0xC0;
const ILI9341_PWCTR2:u8 = 0xC1;
const ILI9341_PWCTR3:u8 = 0xC2;
const ILI9341_PWCTR4:u8 = 0xC3;
const ILI9341_PWCTR5:u8 = 0xC4;
const ILI9341_VMCTR1:u8 = 0xC5;
const ILI9341_VMCTR2:u8 = 0xC7;

const ILI9341_RDID4:u8 = 0xD3;
const ILI9341_RDINDEX:u8 = 0xD9;
const ILI9341_RDID1:u8 = 0xDA;
const ILI9341_RDID2:u8 = 0xDB;
const ILI9341_RDID3:u8 = 0xDC;
const ILI9341_RDIDX:u8 = 0xDD; // TBC

const ILI9341_GMCTRP1:u8 = 0xE0;
const ILI9341_GMCTRN1:u8 = 0xE1;

const ILI9341_MADCTL_MY:u8 = 0x80;
const ILI9341_MADCTL_MX:u8 = 0x40;
const ILI9341_MADCTL_MV:u8 = 0x20;
const ILI9341_MADCTL_ML:u8 = 0x10;
const ILI9341_MADCTL_RGB:u8 = 0x00;
const ILI9341_MADCTL_BGR:u8 = 0x08;
const ILI9341_MADCTL_MH:u8 = 0x04;

const LCD_WIDTH:u16 = 320;
const LCD_HEIGHT:u16 = 240;

impl Lcd {
    pub fn new(bus: &mut SpiBus, pin_cs: GpioPin, pin_dc: GpioPin, pin_rst: GpioPin, pin_bl: GpioPin) -> Result<Lcd, LcdError> {
        let spi_device_config = SpiDeviceInterfaceConfig {
            cs_pin: Some(pin_cs),
            clock_speed_hz: 20000000,
            ..Default::default()
        };
        let mut dc = pin_dc.normal();
        let mut rst = pin_rst.normal();
        let mut bl = pin_bl.normal();
        let mut cs = pin_cs.normal();
        
        dc.configure(GpioConfig::output())?;
        rst.configure(GpioConfig::output())?;
        cs.configure(GpioConfig::output())?;
        bl.configure(GpioConfig::output())?;
        
        dc.set_high()?;
        rst.set_low()?;
        bl.set_low()?;
        cs.set_high()?;

        let device = bus.add_device(spi_device_config, 
            move |dc:&bool| {
                let mut dc_pin = pin_dc.normal();
                dc_pin.set_level(*dc);
            }, 
            |_| {}
        )?;
        let lcd = Lcd{spi: device, pin_dc: dc, pin_rst: rst, pin_bl: bl, pin_cs: cs, line_buffer: [0u8; 640]};
        Ok(lcd)
    }

    pub fn reset(&mut self) -> Result<(), LcdError> {
        self.pin_rst.set_low()?;
        TaskDelay::new().delay_until(Duration::ms(150));
        self.pin_rst.set_high()?;
        TaskDelay::new().delay_until(Duration::ms(150));

        self.write_cmd_data(0xef, &[0x03, 0x80, 0x02])?;
        self.write_cmd_data(0xcf, &[0x00, 0xc1, 0x30])?;
        self.write_cmd_data(0xed, &[0x64, 0x03, 0x12, 0x81])?;
        self.write_cmd_data(0xe8, &[0x85, 0x00, 0x78])?;
        self.write_cmd_data(0xcb, &[0x39, 0x2c, 0x00, 0x34, 0x02])?;
        self.write_cmd_data(0xf7, &[0x20])?;
        self.write_cmd_data(0xea, &[0x00, 0x00])?;
        self.write_cmd_data(ILI9341_PWCTR1, &[0x23])?;
        self.write_cmd_data(ILI9341_PWCTR2, &[0x10])?;
        self.write_cmd_data(ILI9341_VMCTR1, &[0x3e, 0x28])?;
        self.write_cmd_data(ILI9341_VMCTR2, &[0x86])?;
        self.write_cmd_data(ILI9341_MADCTL, &[0xa8])?;
        self.write_cmd_data(ILI9341_PIXFMT, &[0x55])?;
        self.write_cmd_data(ILI9341_FRMCTR1, &[0x00, 0x13])?;
        self.write_cmd_data(ILI9341_DFUNCTR, &[0x08, 0x82, 0x27])?;
        self.write_cmd_data(0xf2, &[0x00])?;
        self.write_cmd_data(ILI9341_GAMMASET, &[0x01])?;
        self.write_cmd_data(ILI9341_GMCTRP1, &[0x0F, 0x31, 0x2B, 0x0C, 0x0E, 0x08, 0x4E, 0xF1, 0x37, 0x07, 0x10, 0x03, 0x0E, 0x09, 0x00])?;
        self.write_cmd_data(ILI9341_GMCTRN1, &[0x00, 0x0E, 0x14, 0x03, 0x11, 0x07, 0x31, 0xC1, 0x48, 0x08, 0x0F, 0x0C, 0x31, 0x36, 0x0F])?;
        self.write_cmd(ILI9341_SLPOUT)?;
        
        TaskDelay::new().delay_until(Duration::ms(120));
        self.write_cmd(ILI9341_DISPON)?;
        self.write_cmd_data(TFT_MADCTL, &[TFT_MAD_BGR])?;
        
        self.pin_bl.set_level(true)?;
        Ok(())
    }

    pub fn read_id(&mut self) -> Result<[u8;3], LcdError> {
        let mut buffer = [0, 0, 0]; 
        self.write_cmd(0x04)
            .and_then(|_| self.read_data(&mut buffer))
            .and(Ok(buffer))
    }

    pub fn set_column_address(&mut self, start: u16, end: u16) -> Result<(), LcdError> {
        let buffer: [u8; 4] = [
            (start >> 8) as u8,
            (start & 0xff) as u8,
            (end >> 8) as u8,
            (end & 0xff) as u8,
        ];

        self.write_cmd_data(0x2a, &buffer)
    }
    pub fn set_page_address(&mut self, start: u16, end: u16) -> Result<(), LcdError> {
        let buffer: [u8; 4] = [
            (start >> 8) as u8,
            (start & 0xff) as u8,
            (end >> 8) as u8,
            (end & 0xff) as u8,
        ];
        self.write_cmd_data(0x2b, &buffer)
    }
    pub fn start_memory_write(&mut self) -> Result<(), LcdError> {
        self.write_cmd(0x2c)
    }

    pub fn write_cmd(&mut self, command: u8) -> Result<(), LcdError> {
        let buffer = [command];
        let transaction = SpiTransaction::new_write(&buffer, false);
        let mut device = self.spi.lock()?;
        device.transfer(transaction)?;
        Ok(())
    }
    
    pub fn write_data(&self, data: &[u8]) -> Result<(), LcdError> {
        let transaction = SpiTransaction::new_write(data, true);
        let mut device = self.spi.lock()?;
        device.transfer(transaction)?;
        Ok(())
    }

    pub fn write_cmd_data(&mut self, command: u8, values: &[u8]) -> Result<(), LcdError> {
        self.write_cmd(command)
            .and_then(|_| self.write_data(values))
    }

    pub fn read_data(&mut self, data: &mut [u8]) -> Result<(), LcdError> {
        let transaction = SpiTransaction::new_read(data, true);
        let mut device = self.spi.lock()?;
        device.transfer(transaction)?;
        Ok(())
    }

    pub fn fill(&mut self, x0: u16, y0: u16, x1: u16, y1: u16, color: Rgb565) -> Result<(), LcdError> {
        let width = x1 - x0;
        for x in 0..width {
            self.line_buffer[(x*2 + 0) as usize] = (color.0 >> 8) as u8;
            self.line_buffer[(x*2 + 1) as usize] = (color.0 & 0xffu16) as u8;
        }
        self.set_column_address(x0, x1)?;
        self.set_page_address(y0, y1)?;
        self.start_memory_write()?;
        
        for y in y0..y1 {
            self.write_data(&self.line_buffer[0..((width*2) as usize)])?;
        }
        Ok(())
    }

    fn inner_draw<T>(&mut self, item: T)
        where T: IntoIterator<Item = Pixel<Rgb565>>,
    {
        let mut buffer = [0u8; (LCD_WIDTH*2) as usize];
        let mut count:usize = 0;
        let mut last_x_opt:Option<u32> = None;
        let mut last_y_opt:Option<u32> = None;

        for Pixel(coord, color) in item {
            let x = coord[0];
            let y = coord[1];
            if let Some(last_y) = last_y_opt {
                if last_y != y {
                    // Flush the last line
                    if let Some(last_x) = last_x_opt {
                        if count > 0 {
                            self.set_column_address((last_x + 1 - (count as u32)) as u16, (last_x + 1) as u16);
                            self.set_page_address(last_y as u16, last_y as u16);
                            self.start_memory_write();
                            self.write_data(&buffer[..count*2]);
                            count = 0;
                        }
                    }
                    last_x_opt = None;
                }
            }
            match last_x_opt {
                None => {
                    // Nothing to do.
                },
                Some(last_x) => {
                    if last_x + 1 != x && count > 0 {
                        if let Some(last_y) = last_y_opt {
                            self.set_column_address((last_x + 1 - (count as u32)) as u16, (last_x + 1) as u16);
                            self.set_page_address(last_y as u16, last_y as u16);
                            self.start_memory_write();
                            self.write_data(&buffer[..count*2]);
                            last_x_opt = None;
                            count = 0;
                        }
                    }
                },
            }
            buffer[count*2 + 0] = (color.0 >> 8)   as u8;
            buffer[count*2 + 1] = (color.0 & 0xff) as u8;
            last_x_opt = Some(x);
            last_y_opt = Some(y);
            count += 1;
        }

        // Flush the last line
        if let Some(last_x) = last_x_opt {
            if let Some(last_y) = last_y_opt {
                if count > 0 {
                    self.set_column_address((last_x + 1 - (count as u32)) as u16, (last_x + 1) as u16);
                    self.set_page_address(last_y as u16, last_y as u16);
                    self.start_memory_write();
                    self.write_data(&buffer[..count*2]);
                }
            }
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawU1(u8);
impl PixelColor for RawU1 {}
impl From<u8> for RawU1 {
    fn from(data: u8) -> Self {
        Self(if data == 0 { 0u8 } else { 1u8 } )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Monochrome(u8);
impl PixelColor for Monochrome {}
impl From<u8> for Monochrome {
    fn from(data: u8) -> Self {
        Self(if data == 0 { 0u8 } else { 1u8 } )
    }
}
impl Into<Rgb565> for Monochrome {
    fn into(self) -> Rgb565 {
        Rgb565( if self.0 == 0 { 0 } else { 0xffffu16 } )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GrayScale8(u8);

impl PixelColor for GrayScale8 {}
impl From<u8> for GrayScale8 {
    fn from(data: u8) -> Self {
        Self(data)
    }
}
impl From<RawU1> for GrayScale8 {
    fn from(data: RawU1) -> Self {
        Self( if data == RawU1(0) { 0 } else { 0xffu8 } )
    }
}

impl Into<Rgb565> for GrayScale8 {
    fn into(self) -> Rgb565 {
        Rgb565( if self.0 == 0 { 0 } else { 0xffffu16 } )
    }
}

impl<TPixelColor> embedded_graphics::Drawing<TPixelColor> for Lcd 
    where TPixelColor : PixelColor + Into<Rgb565> 
{
    fn draw<T>(&mut self, item: T)
    where
        T: IntoIterator<Item = Pixel<TPixelColor>>,
    {
        self.inner_draw(item.into_iter().map(|v| Pixel::<Rgb565>(v.0, v.1.into())));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GrayScale16(u16);

impl PixelColor for GrayScale16 {}
impl From<u8> for GrayScale16 {
    fn from(data: u8) -> Self {
        Self( (data as u16) << 8 )
    }
}
impl From<u16> for GrayScale16 {
    fn from(data: u16) -> Self {
        Self(data)
    }
}
impl From<RawU1> for GrayScale16 {
    fn from(data: RawU1) -> Self {
        Self( if data == RawU1(0) { 0 } else { 0xffffu16 } )
    }
}