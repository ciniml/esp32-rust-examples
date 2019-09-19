#![no_std]
#![feature(lang_items, core_intrinsics, alloc_error_handler, alloc)]

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


// GlobalAllocator
extern "C" {
    fn malloc(size: usize) -> *mut u8;
    fn free(ptr: *mut u8);
}
use core::alloc::{GlobalAlloc, Layout};
struct LibcAllocator;
unsafe impl GlobalAlloc for LibcAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() > 8 {
            panic!("Unsupported alignment")
        }
        malloc(layout.size()) as *mut u8
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        free(ptr)
    }
}
#[global_allocator]
static A: LibcAllocator = LibcAllocator;
#[alloc_error_handler]
fn on_oom(_layout: Layout) -> ! {
    print!("OOM: {:?}", _layout);
    loop {}
}


// rust main
extern crate alloc;
use alloc::vec;

#[no_mangle]
pub fn rust_main()
{
    let v = vec![1, 2, 3, 4, 5];
    print!("v = {:?}\n", v);
    for i in &v {
        print!("{}\n", i);
    }
}
