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

// Stdout struct 
use core::fmt;
use core::fmt::Write;
struct Stdout;
impl fmt::Write for Stdout {
    // Implement write_str to write out a formatted string to stdout.
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe {
            let buffer = s.as_bytes();
            let mut offset = 0;
            loop {
                let bytes_written = write(0, buffer[offset..].as_ptr(), buffer.len() - offset);
                if bytes_written < 0 {
                    return Err(()).map_err(|_| fmt::Error);
                }
                offset += bytes_written;
                if( offset == buffer.len() ) {
                    return Ok(())
                }
            }
        }
    }
}
// print formatted arguments to the stdout.
fn print_fmt(args: fmt::Arguments) {
    let mut stdout = Stdout{};
    stdout.write_fmt(args).unwrap()
}

macro_rules! print {
    ($($arg:tt)*) => (print_fmt(format_args!($($arg)*)));
}

// rust main
#[no_mangle]
pub fn rust_main()
{
    print!("Hello from {}!\n", "Rust");
    print!("format number: {}\n", 1);
}
