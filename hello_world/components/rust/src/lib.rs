#![no_std]
#![feature(lang_items, alloc_error_handler)]

// panic handler
use core::panic::PanicInfo;
#[lang = "panic_impl"]
#[no_mangle]
pub extern fn rust_begin_panic(_info: &PanicInfo) -> ! {
    loop {}
}
use core::alloc::Layout;
#[alloc_error_handler]
fn on_oom(_layout: Layout) -> ! {
    loop {}
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