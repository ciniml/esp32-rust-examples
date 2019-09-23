#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod std {
    pub use core::*;
    pub mod os {
        pub mod raw {
            pub enum c_void {}
            pub type c_uchar = u8;
            pub type c_schar = i8;
            pub type c_char = i8;
            pub type c_short = i16;
            pub type c_ushort = u16;
            pub type c_int = i32;
            pub type c_uint = u32;
            pub type c_long = i32;
            pub type c_ulong = u32;
            pub type c_longlong = i64;
            pub type c_ulonglong = u64;
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));


pub trait AsResult<T, E> {
    fn as_result(self) -> Result<T, E>;
}
impl AsResult<(), IdfError> for esp_err_t {
    fn as_result(self) -> Result<(), IdfError> {
        if self == 0 {
            Ok(())
        }
        else {
            Err(IdfError(self))
        }
    }
}

pub const portMAX_DELAY: TickType_t = 0xffffffff;

#[derive(Copy, Clone, Debug)]
pub struct IdfError(esp_err_t);

impl From<esp_err_t> for IdfError {
    fn from(err: esp_err_t) -> Self {
        IdfError(err)
    }
}

impl From<()> for IdfError {
    fn from(err: ()) -> Self {
        IdfError(1)
    }
}
impl Into<Result<(), IdfError>> for IdfError {
    fn into(self) -> Result<(), IdfError> {
        if self.0 == 0 { Ok(()) } else { Err(self) }
    }
}