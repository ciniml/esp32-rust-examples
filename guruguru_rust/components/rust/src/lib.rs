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
use alloc::sync::Arc;

use core::ptr;
use core::mem;
use core::str;
use core::ffi::*;
use rand::prelude::*;

use m5stack::*;
use embedded_graphics::coord::Coord;
use embedded_graphics::fonts::Font6x8;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, Line, Rectangle};
use embedded_graphics::image::{Image1BPP, Image8BPP, Image16BPP};
use embedded_graphics::pixelcolor::Rgb565;

use freertos_rs::*;

use idf::*;
use embedded_hal::blocking::spi::Write as spiWrite; 

use peripheral::*;

#[no_mangle]
pub extern fn rust_main() {
    #[repr(u8)]
    #[derive(Copy, Clone, Debug)]
    enum ButtonName {
        A,
        B,
        C,
    }

    #[derive(Copy, Clone, Debug)]
    struct ButtonEvent {
        button: ButtonName,
        pressed: bool,
    }

    let queue = Arc::new( Queue::<ButtonEvent>::new(32).unwrap() );
    let queueDrawTask = queue.clone();
    let _drawTask = Task::new().name("line task").stack_size(4096).core(1).start(move || {
        unsafe { esp_task_wdt_add(ptr::null_mut()); }

        let spi_bus_config = SpiBusConfig {
            mosi_pin: GpioPin23,
            miso_pin: GpioPin19,
            sclk_pin: GpioPin18,
            quadwp_pin: None,
            quadhd_pin: None,
            max_transfer_size: 320*4,
        };
        print!("SPI Config: {:#?}\n", spi_bus_config);
        let mut spi_bus = SpiBus::new(SpiHostDevice::Vspi, spi_bus_config, 1).unwrap();
        print!("Initializing LCD...\n");
        let mut display = Lcd::new(&mut spi_bus, GpioPin14, GpioPin27, GpioPin33, GpioPin32).unwrap();
        display.reset().unwrap();
        
        let images: [Image1BPP<Monochrome>; 9] = [
            Image1BPP::<Monochrome>::new(include_bytes!("../../../assets/rust-logo-0.raw"), 240, 240).translate(Coord::new(40, 0)),
            Image1BPP::<Monochrome>::new(include_bytes!("../../../assets/rust-logo-45.raw"), 240, 240).translate(Coord::new(40, 0)),
            Image1BPP::<Monochrome>::new(include_bytes!("../../../assets/rust-logo-90.raw"), 240, 240).translate(Coord::new(40, 0)),
            Image1BPP::<Monochrome>::new(include_bytes!("../../../assets/rust-logo-135.raw"), 240, 240).translate(Coord::new(40, 0)),
            Image1BPP::<Monochrome>::new(include_bytes!("../../../assets/rust-logo-180.raw"), 240, 240).translate(Coord::new(40, 0)),
            Image1BPP::<Monochrome>::new(include_bytes!("../../../assets/rust-logo-225.raw"), 240, 240).translate(Coord::new(40, 0)),
            Image1BPP::<Monochrome>::new(include_bytes!("../../../assets/rust-logo-270.raw"), 240, 240).translate(Coord::new(40, 0)),
            Image1BPP::<Monochrome>::new(include_bytes!("../../../assets/rust-logo-315.raw"), 240, 240).translate(Coord::new(40, 0)),
            Image1BPP::<Monochrome>::new(include_bytes!("../../../assets/fuga.raw"), 240, 240).translate(Coord::new(40, 0)),
        ];
        display.fill(0, 0, 320, 240, Rgb565(0xffffu16));
        let mut angle = 0;
        #[derive(Copy, Clone, Debug)]
        enum Mode {
            RustLogoManual,
            RustLogoAuto,
            TwitterIcon,
        }
        let mut mode = Mode::RustLogoManual;
        loop {
            display.draw(&images[angle]);
            match mode {
                Mode::RustLogoManual => {
                    if let Ok(event) = queueDrawTask.receive(Duration::infinite()) {
                        print!("event: {:?}, angle: {}\n", event, angle);

                        if event.pressed {
                            match event.button {
                                ButtonName::A => angle = if angle > 0 { angle - 1 } else { 7 },
                                ButtonName::C => angle = if angle == 7 { 0 } else { angle + 1 },
                                ButtonName::B => mode = Mode::RustLogoAuto,
                            }
                        }
                    }
                },
                Mode::RustLogoAuto => {
                    angle = if angle == 7 { 0 } else { angle + 1 };
                    if let Ok(event) = queueDrawTask.receive(Duration::ms(250)) {
                        if event.pressed {
                            match event.button {
                                ButtonName::B => mode = Mode::TwitterIcon,
                                _ => (),
                            }
                        }
                    }
                },
                Mode::TwitterIcon => {
                    angle = 8;  // Twitter Icon
                    if let Ok(event) = queueDrawTask.receive(Duration::infinite()) {
                        if event.pressed {
                            match event.button {
                                ButtonName::B => {angle = 0; mode = Mode::RustLogoManual},
                                _ => (),
                            }
                        }
                    }
                },
            }
        }
    }).unwrap();
    


    let mainTask = Task::current().unwrap();
    let queueRequestTask = queue.clone();
    let _inputTask = Task::new().name("input task").stack_size(4096).core(0).start(move || {
        unsafe { esp_task_wdt_add(ptr::null_mut()); }
        struct ButtonInput {
            gpio: NormalGpio,
            button: ButtonName,
            pressed: bool,
        }
        let mut buttons: [ButtonInput; 3] = [
            ButtonInput{ gpio: GpioPin39.normal(), button: ButtonName::A, pressed: false },
            ButtonInput{ gpio: GpioPin38.normal(), button: ButtonName::B, pressed: false },
            ButtonInput{ gpio: GpioPin37.normal(), button: ButtonName::C, pressed: false },
        ];
        for button in &mut buttons {
            button.gpio.configure(GpioConfig::inputPullUp());
        }
        
        loop {
            for button in &mut buttons {
                let pressed = !button.gpio.get_level().unwrap_or(true);
                if button.pressed != pressed {
                    print!("button changed: {:?}, {}\n", button.button, pressed);
                    queueRequestTask.send( ButtonEvent { button: button.button, pressed: pressed }, Duration::ms(1) );
                }
                button.pressed = pressed;
            }
            CurrentTask::delay(Duration::ms(10));
            unsafe {
                esp_task_wdt_reset();
            }
        }
    }).unwrap();
    
    let mainTask = Task::current().unwrap();
    mainTask.wait_for_notification(0x01, 0x01, Duration::infinite());
}   
