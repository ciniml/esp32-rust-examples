#![no_std]
#![feature(lang_items, core_intrinsics)]

// panic handler
use core::intrinsics;
use core::panic::PanicInfo;
#[lang = "panic_impl"]
extern fn rust_begin_panic(_info: &PanicInfo) -> ! {
    unsafe { intrinsics::abort() }
}

// write function in standard C library
extern "C" {
    fn write(fd: i32, data: *const u8, size: usize) -> usize;
}

// rust main
#[no_mangle]
pub fn rust_main()
{
    let buffer = "Hello, from Rust\n".as_bytes();
    unsafe {
        write(0, buffer.as_ptr(), buffer.len());
    }
}