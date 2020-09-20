#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate cty;

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